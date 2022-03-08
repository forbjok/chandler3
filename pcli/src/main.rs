use std::path::PathBuf;

use clap::Parser;
use strum_macros::EnumString;

mod command;
mod result;
mod ui;

use chandler::project;
use tracing::debug;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Clone, Copy, Debug, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ProjectFormat {
    V2,
    V3,
}

#[derive(Debug, Parser)]
#[clap(name = "Chandler Programmatic CLI", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opt {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
pub struct ProjectOptions {
    #[clap(long = "format", default_value = "v3", help = "Project format to create (v2|v3)")]
    format: ProjectFormat,
}

#[derive(Debug, Parser)]
enum Command {
    #[clap(name = "grab", about = "Download thread")]
    Grab {
        #[clap(help = "URL of thread to download")]
        url: String,
        #[clap(help = "Destination path to download to")]
        destination: PathBuf,
        #[clap(flatten)]
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
    let opt = Opt::parse();

    // Initialize logging
    initialize_logging();

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

fn initialize_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default tracing subscriber failed!");
}
