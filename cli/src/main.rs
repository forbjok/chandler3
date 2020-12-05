use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::{debug, error, info, LevelFilter};
use structopt::StructOpt;
use strum_macros::EnumString;

mod command;
mod config;
mod error;
mod ui;

use chandler::project;
use chandler::ui::*;

use crate::ui::*;

#[derive(Clone, Copy, Debug, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ProjectFormat {
    V2,
    V3,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Chandler", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
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
        #[structopt(help = "URL of threads to download")]
        url: String,
        #[structopt(flatten)]
        project_options: ProjectOptions,
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

    let cfg = match config::CliConfig::from_default_location() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{}", err);

            config::CliConfig::default()
        }
    };

    // Cancellation boolean.
    let cancel = Arc::new(AtomicBool::new(false));

    // Set break (Ctrl-C) handler.
    ctrlc::set_handler({
        let cancel = Arc::clone(&cancel);

        move || {
            info!("Cancellation requested by user.");
            cancel.store(true, Ordering::SeqCst);
        }
    })
    .unwrap_or_else(|err| error!("Error setting Ctrl-C handler: {}", err));

    // Create UI handler.
    let mut ui: Box<dyn ChandlerUiHandler> = {
        let cancel = Arc::clone(&cancel);

        let cancel_check = Box::new(move || {
            // If cancellation has been requested, break out immediately.
            if cancel.load(Ordering::SeqCst) {
                return true;
            }

            false
        });

        if cfg.progress.enable {
            let progress_chars = match cfg.progress.bar_style {
                config::CliProgressBarStyle::Dot => "●●·",
                config::CliProgressBarStyle::Hash => "##·",
                config::CliProgressBarStyle::Arrow => "=> ",
            }
            .to_owned();

            Box::new(IndicatifUiHandler::new(progress_chars, cancel_check))
        } else {
            Box::new(StderrUiHandler::new(cancel_check))
        }
    };

    let cmd_result = match opt.command {
        Command::Grab { url, project_options } => command::grab(&url, &project_options, ui.as_mut()),
        Command::Rebuild { path } => command::rebuild(&path, ui.as_mut()),
        Command::Watch {
            url,
            interval,
            project_options,
        } => command::watch(&url, interval, &project_options, ui.as_mut()),
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
