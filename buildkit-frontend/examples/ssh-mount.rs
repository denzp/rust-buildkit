use async_trait::async_trait;
use failure::{bail, Error};

use buildkit_frontend::oci::*;
use buildkit_frontend::run_frontend;
use buildkit_frontend::{Bridge, Frontend, FrontendOutput, Options, OutputRef};

use buildkit_llb::prelude::*;

#[tokio::main(basic_scheduler)]
async fn main() {
    env_logger::init();

    if let Err(_) = run_frontend(ReverseFrontend).await {
        std::process::exit(1);
    }
}

struct ReverseFrontend;

const OUTPUT_FILENAME: &str = "/test.out";
const PATH: &str =
    "/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

#[async_trait]
impl Frontend for ReverseFrontend {
    async fn run(self, bridge: Bridge, options: Options) -> Result<FrontendOutput, Error> {
        Ok(FrontendOutput::with_spec_and_ref(
            Self::image_spec(),
            Self::solve(&bridge, options.get("filename").unwrap()).await?,
        ))
    }
}

impl ReverseFrontend {
    fn image_spec() -> ImageSpecification {
        ImageSpecification {
            created: None,
            author: None,

            architecture: Architecture::Amd64,
            os: OperatingSystem::Linux,

            config: Some(ImageConfig {
                entrypoint: None,
                cmd: Some(vec!["/bin/cat".into(), OUTPUT_FILENAME.into()]),
                env: None,
                user: None,
                working_dir: Some("/output".into()),

                labels: None,
                volumes: None,
                exposed_ports: None,
                stop_signal: None,
            }),

            rootfs: None,
            history: None,
        }
    }

    async fn solve(bridge: &Bridge, dockerfile_path: &str) -> Result<OutputRef, Error> {
        let dockerfile_source = Source::local("dockerfile");
        let dockerfile_layer = bridge
            .solve(Terminal::with(dockerfile_source.output()))
            .await?;

        let dockerfile_contents = bridge
            .read_file(&dockerfile_layer, dockerfile_path, None)
            .await?;

        let dockerfile_contents = String::from_utf8_lossy(&dockerfile_contents);

        let mut repo = None;
        let mut tag = None;
        let mut test = None;

        for line in dockerfile_contents.lines() {
            if line.starts_with("REPO:") {
                repo = Some(line[5..].trim());
            }

            if line.starts_with("TAG:") {
                tag = Some(line[4..].trim());
            }

            if line.starts_with("TEST:") {
                test = Some(line[5..].trim());
            }
        }

        let rootfs = Source::image("rust:latest");
        let install_command = match (repo, tag) {
            (Some(repo), Some(tag)) => Command::run("cargo")
                .args(&["install", "--git", repo, "--tag", tag])
                .mount(Mount::Layer(OutputIdx(0), rootfs.output(), "/"))
                .mount(Mount::OptionalSshAgent("/tmp/ssh_agent.0"))
                .env("PATH", PATH)
                .env("RUSTUP_HOME", "/usr/local/rustup")
                .env("CARGO_HOME", "/usr/local/cargo")
                .env("RUST_VERSION", "1.40.0")
                .env("SSH_AUTH_SOCK", "/tmp/ssh_agent.0"),

            _ => {
                bail!("Missing REPO or TAG directives!");
            }
        };

        let test_command = if let Some(test) = test {
            Command::run("/bin/sh")
                .args(&["-c", &format!("{} > {}", test, OUTPUT_FILENAME)])
                .mount(Mount::Layer(OutputIdx(0), install_command.output(0), "/"))
                .env("PATH", PATH)
        } else {
            bail!("Missing TEST directive!");
        };

        bridge.solve(Terminal::with(test_command.output(0))).await
    }
}
