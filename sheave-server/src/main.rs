#![allow(unexpected_cfgs)]

pub mod net;
pub mod handlers;
mod server;

use std::{
    io::{
        Error as IOError,
        Result as IOResult
    },
    marker::PhantomData
};
use log::{
    LevelFilter,
    error,
    info
};
use env_logger::builder;
use clap::{
    Args,
    Parser,
    ValueEnum
};
use dotenvy::from_filename;
use tokio::spawn;
use sheave_core::{
    handlers::RtmpContext,
    net::rtmp::RtmpStream
};
use self::{
    handlers::RtmpHandler,
    net::rtmp::RtmpListener
};
pub use self::server::*;

#[derive(Debug, Clone, Args)]
#[group(required = true)]
struct Listeners {
    /// Listening addresses/ports via RTMP.
    /// Currently only a address/port is allowed and plural addresses/ports are available.
    /// Because of unimplemented the connection pool yet.
    #[arg(num_args(1..), long, value_name = "Address", value_delimiter = ',', env = "RTMP_LISTENERS")]
    rtmp: Vec<String>
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum LogLevel {
    #[default]
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace
}

impl From<LogLevel> for LevelFilter {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace
        }
    }
}

/// Command line options for the Sheave Server.
///
/// # Required Arguments
///
/// * Listening protocols and addresses/ports
///
/// `sheave-server --rtmp 127.0.0.1:1935`
#[derive(Debug, Parser)]
#[command(author, version)]
struct ServerOptions {
    #[command(flatten)]
    listeners: Listeners,
    /// Displays server status in detail by logger.
    /// Correspondence of parameters to log kinds are following:
    ///
    /// |Parameter|Log Kind|
    /// | :- | :- |
    /// |`off`|Logs nothing.|
    /// |`error`|<ul><li>Cause of server/connection stopping.</li></ul>|
    /// |`warn`|<ul><li>Limit excess.</li><li>Insufficient parameter.</li></ul>|
    /// |`info`|<ul><li>Current process</li></ul>|
    /// |`debug`|<ul><li>Detailed data for debugging.</li></ul>|
    /// |`trace`|<ul><li>Detailed process for tracing.</li></ul>|
    #[arg(long, value_enum, value_name = "LogLevel", default_value_t, env = "LOGLEVEL")]
    loglevel: LogLevel,
    /// The database storage URL to keep playpath/subscribepath to handle FLV data.
    /// This must start with One of database URL schemas. (e.g. mysql:, postgres:, sqlite:, etc.)
    ///
    /// When you store topics into SQLite storages, you can use in-memory storage URL (`:memory:`).
    #[arg(long, required = true, value_name = "URL", env = "TOPIC_STORAGE_URL")]
    topic_storage_url: String,
    // TODO: Makes other options if they are required.
}

async fn run_as_rtmp(address: &str, topic_storage_url: &str) -> IOResult<()> {
    let listener = RtmpListener::bind(address).await?;

    loop {
        let (stream, client_addr) = listener.accept().await?;
        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_topic_storage_url(topic_storage_url);
        rtmp_context.set_client_addr(client_addr);
        let server = Server::new(stream, rtmp_context, PhantomData::<RtmpHandler<RtmpStream>>);
        return spawn(server).await?;
    }
}

#[tokio::main]
async fn main() -> IOResult<()> {
    /* NOTE: Makes sure to set enviromnent variables into CLI options as defaults. (if dotenv file is specified) */
    if let Some(filename) = option_env!("DOTENV") {
        from_filename(filename).map_err(|e| IOError::other(e))?;
    }

    let options = ServerOptions::parse();

    builder().filter_level(options.loglevel.into()).try_init().map_err(|e| IOError::other(e))?;

    if let Err(e) = run_as_rtmp(&options.listeners.rtmp[0], &options.topic_storage_url).await {
        error!("Some error got occurred: {e}");
        return Err(e)
    }

    info!("RTMP communication got completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use super::*;

    #[test]
    fn err_no_parameter() {
        let result = ServerOptions::command()
            .try_get_matches_from(vec!["sheave-server"]);
        assert!(result.is_err())
    }

    #[test]
    fn err_missing_topic_storage_url() {
        let result = ServerOptions::command()
            .try_get_matches_from(vec!["sheave-server", "--rtmp", "127.0.0.1:1935"]);
        assert!(result.is_err())
    }

    #[test]
    fn ok_passing_required_parameters() {
        let result = ServerOptions::command()
            .try_get_matches_from(vec!["sheave-server", "--rtmp", "127.0.0.1:1935", "--topic-storage-url", "sqlite::memory:"]);
        assert!(result.is_ok());
        let result = ServerOptions::command()
            .try_get_matches_from(vec!["sheave-server", "--rtmp", "127.0.0.1:1935,0.0.0.0:1935", "--topic-storage-url", "sqlite::memory:"]);
        assert!(result.is_ok())
    }
}
