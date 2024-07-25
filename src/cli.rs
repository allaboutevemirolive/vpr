use clap::Arg;
use clap::ArgAction;
use clap::Command;

pub mod options {
    //
    pub mod miscellaneous {
        pub static VERSION: &str = "version";
    }
}

pub fn vrp_app() -> Command {
    Command::new("vrp")
        .infer_long_args(true)
        .args_override_self(true)
        .arg(
            Arg::new(options::miscellaneous::VERSION)
                .long("version")
                .short('V')
                .help("Print current version of VRP.")
                .action(ArgAction::SetTrue),
        )
}
