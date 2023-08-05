use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

use auto_docker_compose::run;

#[derive(StructOpt, Debug)]
#[structopt(name = "adcompose")]
struct Cli {
    // Command
    // List: list filter or all the compose project
    // Start: start filter or all the compose project
    // Stop: stop filter or all the compose project
    /// The compose command, list, start, stop
    command: String,

    /// filter compose name
    #[structopt(default_value = "")]
    filter_name: String,

    /// The search compose path
    #[structopt(short, long, default_value = "./")]
    path: PathBuf,
}

fn main() {
    let args = Cli::from_args();

    if !args.path.exists() || !args.path.is_dir() {
        println!("{} is not a exists path or not is dir", args.path.display());
        process::exit(1);
    }

    let filter_name = if "" == args.filter_name {
        None
    } else {
        Some(args.filter_name)
    };

    match run(args.command, &args.path, filter_name) {
        Err(err) => {
            eprintln!("{}", err.to_string());
        }
        _ => {}
    };
}
