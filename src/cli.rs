use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// This struct contains all the arguments captured from the command line.
#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about=None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    /// Run the simulation with the partitioned graph
    GraphParts {
        /// Path of the JSON file for the Graph
        input_file: PathBuf,

        /// Mininum Vehicle Speed in m/s
        #[arg(long, default_value = "8.5")]
        min_speed: f64,

        /// Maximum Vehicle Speed in m/s
        #[arg(long, default_value = "13.8")]
        max_speed: f64,

        /// Whether to run it sequential or parallel
        #[arg(short, long, default_value_t=Parallelism::SingleThreaded, value_enum)]
        parallelism: Parallelism,

        /// Number of Vehicles
        #[arg(short, long, default_value = "1")]
        num_vehicles: usize,

        /// Logging level
        #[arg(short, long, default_value_t=LoggingLevel::Info, value_enum)]
        logging_level: LoggingLevel,

        /// Whether to use Tokio or Rust Threads
        #[arg(short, long, default_value_t=ThreadRuntime::RustThreads, value_enum)]
        thread_runtime: ThreadRuntime,

        /// Use MPI. Requires MPI to be installed
        #[arg(short, long, default_value = "false")]
        mpi: bool,

        /// Error rate
        #[arg(short, long, default_value = "0.0")]
        error_rate: f64,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LoggingLevel {
    /// Debug logging
    Debug,
    /// Info logging
    Info,
    /// Warning logging
    Warn,
    /// Error logging
    Error,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Parallelism {
    /// Run in a single threaded
    SingleThreaded,
    /// Run in multiple threads on a single node
    MultiThreaded,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ThreadRuntime {
    /// Run with Tokio
    Tokio,
    /// Run with Rust Threads
    RustThreads,
}
