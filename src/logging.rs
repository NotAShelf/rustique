use std::io;
use std::io::Write;
use tracing::{Event, Level, Subscriber};
use tracing::field::Field;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::field::Visit;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;


pub enum VerboseLevel {
    Verbose,
    Debug,
    Default,
}

pub fn init_logging(verbose: VerboseLevel) {
    let filter = match verbose {
        VerboseLevel::Verbose => {
            EnvFilter::new("info,Rustique=info,ureq=info")
        }
        VerboseLevel::Debug => {
            EnvFilter::new("info,Rustique=debug,ureq=info")
        }
        _ => {
            EnvFilter::new("warn,Rustique=warn,ureq=warn")
        }
    };

    tracing_subscriber::registry()
        .with(fmt::layer()
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_target(false))
        .with(filter)
        .init();
}