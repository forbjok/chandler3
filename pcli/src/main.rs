use std::path::PathBuf;

use log::{debug, LevelFilter};
use structopt::StructOpt;
use strum_macros::EnumString;

mod command;
mod result;
mod ui;

use chandler::project;

#[derive(Clone, Copy, Debug, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ProjectFormat {
    V2,
    V3,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Chandler Programmatic CLI", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opt {
    #[structopt(short = "v", parse(from_occurrences), help = "Verbosity")]
    verbosity: u8,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
pub struct ProjectOptions {
    #[structopt(long = "format", default_value = "v3", help = "Project format to create (v2|v3)")]
    format: ProjectFormat,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "grab", about = "Download thread")]
    Grab {
        #[structopt(help = "URL of thread to download")]
        url: String,
        #[structopt(help = "Destination path to download to")]
        destination: PathBuf,
        #[structopt(flatten)]
        project_options: ProjectOptions,
    },
}

impl From<ProjectFormat> for project::ProjectFormat {
    fn from(v: ProjectFormat) -> Self {
        match v {
            ProjectFormat::V2 => project::ProjectFormat::V2,
            ProjectFormat::V3 => project::ProjectFormat::V3,
        }
    }
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
        _ => LevelFilter::Trace,
    };

    // Initialize logging
    initialize_logging(log_level);

    debug!("Debug logging enabled.");

    let cmd_result = match opt.command {
        Command::Grab {
            url,
            destination,
            project_options,
        } => command::grab(&url, &destination, &project_options),
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
