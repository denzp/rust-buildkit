use std::io::stdout;

use buildkit_llb::prelude::*;

fn main() {
    let builder_image =
        Source::image("library/alpine:latest").custom_name("Using alpine:latest as a builder");

    let command = {
        Command::run("/bin/sh")
            .args(&["-c", "echo 'test string 5' > /out/file0"])
            .custom_name("create a dummy file")
            .mount(Mount::ReadOnlyLayer(builder_image.output(), "/"))
            .mount(Mount::Scratch(OutputIdx(0), "/out"))
    };

    let fs = {
        FileSystem::sequence()
            .custom_name("do multiple file system manipulations")
            .append(
                FileSystem::copy()
                    .from(LayerPath::Other(command.output(0), "/file0"))
                    .to(OutputIdx(0), LayerPath::Other(command.output(0), "/file1")),
            )
            .append(
                FileSystem::copy()
                    .from(LayerPath::Own(OwnOutputIdx(0), "/file0"))
                    .to(OutputIdx(1), LayerPath::Own(OwnOutputIdx(0), "/file2")),
            )
    };

    Terminal::with(fs.output(1))
        .write_definition(stdout())
        .unwrap()
}
