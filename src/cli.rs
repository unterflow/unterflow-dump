use clap::{App, Arg};

pub fn app() -> App<'static, 'static> {
    App::new("unterflow-dump")
        .version(crate_version!())
        .about("Dump unterflow protocol packages")
        .arg(
            Arg::with_name("v")
                .short("v")
                .help("Enable logging, use multiple `v`s to increase verbosity")
                .multiple(true),
        )
        .arg(
            Arg::with_name("interface")
                .short("i")
                .long("interface")
                .help(
                    "Interface to capture (if non is specified the first one found will be used)",
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Ports to capture")
                .multiple(true)
                .default_value("51015"),
        )
        .arg(
            Arg::with_name("list-interfaces")
                .short("l")
                .long("list-interfaces")
                .help("List all interfaces and exists"),
        )
}
