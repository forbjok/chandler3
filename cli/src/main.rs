use std::path::PathBuf;

use log::{debug, LevelFilter};
use structopt::StructOpt;

mod command;
mod config;
mod misc;
mod progress;

#[derive(StructOpt, Debug)]
#[structopt(name = "Chandler", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opt {
    #[structopt(short = "v", parse(from_occurrences), help = "Verbosity")]
    verbosity: u8,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "grab", about = "Download thread")]
    Grab {
        #[structopt(help = "URL of threads to download")]
        url: String,
    },
    #[structopt(name = "rebuild", about = "Rebuild thread from original HTML files")]
    Rebuild {
        #[structopt(help = "Path to project to rebuild")]
        path: PathBuf,
    },
    #[structopt(name = "watch", about = "Watch thread")]
    Watch {
        #[structopt(help = "URL of thread to watch")]
        url: String,
        #[structopt(short = "i", long = "interval", help = "Interval (seconds)", default_value = "600")]
        interval: i64,
    },
}

fn main() {
    let opt = Opt::from_args();

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    let log_level = match opt.verbosity {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 | _ => LevelFilter::Trace,
    };

    // Initialize logging
    initialize_logging(log_level);

    debug!("Debug logging enabled.");

    let cmd_result = match opt.command {
        Command::Grab { url } => command::grab(&url),
        Command::Rebuild { path } => command::rebuild(&path),
        Command::Watch { url, interval } => command::watch(&url, interval),
    };

    match cmd_result {
        Ok(_) => {}
        Err(err) => {
            // Print error description to stderr
            eprintln!("{}", err.description);

            // Return the exit code that corresponds to the error kind
            std::process::exit(err.kind.exit_code());
        }
    };
}

fn initialize_logging(our_level_filter: LevelFilter) {
    use chrono::Utc;

    const BIN_MODULE: &str = env!("CARGO_CRATE_NAME");
    const LIB_MODULE: &str = "chandler";

    fern::Dispatch::new()
        .level(LevelFilter::Error)
        .level_for(BIN_MODULE, our_level_filter)
        .level_for(LIB_MODULE, our_level_filter)
        .chain(std::io::stderr())
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} | {} | {} | {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.target(),
                record.level(),
                message
            ))
        })
        .apply()
        .unwrap();
}
