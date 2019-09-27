#![feature(type_alias_impl_trait)]

use std::iter::once;

use failure::Error;
use futures::prelude::*;

use buildkit_frontend::oci::*;
use buildkit_frontend::run_frontend;
use buildkit_frontend::{Bridge, Frontend, FrontendOutput, Options, OutputRef};

use buildkit_llb::prelude::*;

#[runtime::main(runtime_tokio::Tokio)]
async fn main() {
    if let Err(_) = run_frontend(ReverseFrontend).await {
        std::process::exit(1);
    }
}

struct ReverseFrontend;

impl Frontend for ReverseFrontend {
    type RunFuture = impl Future<Output = Result<FrontendOutput, Error>>;

    fn run(self, mut bridge: Bridge, options: Options) -> Self::RunFuture {
        async move {
            Ok(FrontendOutput::with_spec_and_ref(
                Self::image_spec(),
                Self::solve(&mut bridge, options.get("filename").unwrap()).await?,
            ))
        }
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
                entrypoint: Some(vec![String::from("/bin/cat")]),
                cmd: Some(vec![String::from("/reverse.dockerfile")]),
                env: None,
                user: None,
                working_dir: None,

                labels: None,
                volumes: None,
                exposed_ports: None,
                stop_signal: None,
            }),

            rootfs: None,
            history: None,
        }
    }

    async fn solve(bridge: &mut Bridge, dockerfile_path: &str) -> Result<OutputRef, Error> {
        let dockerfile = Source::local("dockerfile");
        let alpine = Source::image("alpine:latest");

        let dockerfile_layer = bridge.solve(Terminal::with(dockerfile.output())).await?;
        let dockerfile_contents = bridge
            .read_file(&dockerfile_layer, dockerfile_path, None)
            .await?;

        let transformed_dockerfile_contents: String = {
            String::from_utf8_lossy(&dockerfile_contents)
                .lines()
                .into_iter()
                .map(|line| line.chars().rev().chain(once('\n')).collect::<String>())
                .collect()
        };

        let final_llb = {
            FileSystem::mkfile(
                OutputIdx(0),
                LayerPath::Other(alpine.output(), "/reverse.dockerfile"),
            )
            .data(transformed_dockerfile_contents.into_bytes())
            .into_operation()
        };

        let final_layer = bridge.solve(Terminal::with(final_llb.output(0))).await?;

        Ok(final_layer)
    }
}
