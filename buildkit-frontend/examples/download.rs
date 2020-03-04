use std::path::PathBuf;

use async_trait::async_trait;
use failure::Error;
use regex::Regex;
use serde::Deserialize;
use url::Url;

use buildkit_frontend::oci::*;
use buildkit_frontend::options::common::CacheOptionsEntry;
use buildkit_frontend::run_frontend;
use buildkit_frontend::{Bridge, Frontend, FrontendOutput, OutputRef};

use buildkit_llb::prelude::*;

#[tokio::main(threaded_scheduler)]
async fn main() {
    env_logger::init();

    if let Err(_) = run_frontend(DownloadFrontend).await {
        std::process::exit(1);
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct DownloadOptions {
    filename: PathBuf,

    /// New approach to specify cache imports.
    #[serde(default)]
    cache_imports: Vec<CacheOptionsEntry>,

    /// Legacy convention to specify cache imports.
    #[serde(default)]
    #[serde(deserialize_with = "CacheOptionsEntry::from_legacy_list")]
    cache_from: Vec<CacheOptionsEntry>,
}

struct DownloadFrontend;

#[async_trait]
impl Frontend<DownloadOptions> for DownloadFrontend {
    async fn run(self, bridge: Bridge, options: DownloadOptions) -> Result<FrontendOutput, Error> {
        Ok(FrontendOutput::with_spec_and_ref(
            Self::image_spec(),
            Self::solve(&bridge, options).await?,
        ))
    }
}

const OUTPUT_DIR: &str = "/opt";

impl DownloadFrontend {
    fn image_spec() -> ImageSpecification {
        ImageSpecification {
            created: None,
            author: None,

            architecture: Architecture::Amd64,
            os: OperatingSystem::Linux,

            config: Some(ImageConfig {
                entrypoint: Some(vec!["/bin/sh".into()]),
                cmd: Some(vec!["-c".into(), "/usr/bin/sha256sum *".into()]),
                env: None,
                user: None,
                working_dir: Some(OUTPUT_DIR.into()),

                labels: None,
                volumes: None,
                exposed_ports: None,
                stop_signal: None,
            }),

            rootfs: None,
            history: None,
        }
    }

    async fn solve(bridge: &Bridge, options: DownloadOptions) -> Result<OutputRef, Error> {
        let dockerfile_source = Source::local("dockerfile");
        let dockerfile_layer = bridge
            .solve(Terminal::with(dockerfile_source.output()))
            .await?;

        let dockerfile_contents = String::from_utf8(
            bridge
                .read_file(&dockerfile_layer, &options.filename, None)
                .await?,
        )?;

        bridge
            .solve_with_cache(
                Terminal::with(Self::construct_llb(dockerfile_contents)?),
                options.cache_entries(),
            )
            .await
    }

    fn construct_llb(dockerfile: String) -> Result<OperationOutput<'static>, Error> {
        let alpine = Source::image("alpine:latest").ref_counted();

        let builder_rootfs = Command::run("apk")
            .args(&["add", "curl"])
            .custom_name("Installing curl")
            .mount(Mount::Layer(OutputIdx(0), alpine.output(), "/"))
            .ref_counted();

        Self::extract_files(&dockerfile)
            .map(move |result| {
                let (url, relative_path) = result?;
                let full_path = PathBuf::from(OUTPUT_DIR).join(&relative_path);

                let op = Command::run("curl")
                    .args(&[&url.to_string(), "-o", &full_path.to_string_lossy()])
                    .mount(Mount::ReadOnlyLayer(builder_rootfs.output(0), "/"))
                    .mount(Mount::Scratch(OutputIdx(0), OUTPUT_DIR))
                    .custom_name(format!("Downloading '{}'", relative_path.display()))
                    .ref_counted()
                    .output(0);

                Ok((op, relative_path, full_path))
            })
            .try_fold(
                FileSystem::sequence().custom_name("Copying assets into output directory"),
                |output, result: Result<_, Error>| {
                    let (op, relative_path, full_path) = result?;

                    let (out_index, out_layer) = match output.last_output_index() {
                        Some(last) => (last + 1, LayerPath::Own(OwnOutputIdx(last), &full_path)),
                        None => (0, LayerPath::Other(alpine.output(), &full_path)),
                    };

                    Ok(output.append(
                        FileSystem::copy()
                            .from(LayerPath::Other(op, &relative_path))
                            .to(OutputIdx(out_index), out_layer)
                            .create_path(true),
                    ))
                },
            )
            .map(|llb| llb.ref_counted().last_output().unwrap())
    }

    fn extract_files(
        dockerfile: &str,
    ) -> impl Iterator<Item = Result<(Url, PathBuf), url::ParseError>> + '_ {
        let cmd_regex = Regex::new(r#"Download\s+"(.+)"\s+as\s+"(.+)""#).unwrap();

        dockerfile.lines().filter_map(move |line| {
            let captures = cmd_regex.captures(&line)?;
            Some(Url::parse(&captures[1]).map(|url| (url, captures[2].into())))
        })
    }
}

impl DownloadOptions {
    pub fn cache_entries(&self) -> &[CacheOptionsEntry] {
        if !self.cache_imports.is_empty() {
            return &self.cache_imports;
        }

        &self.cache_from
    }
}
