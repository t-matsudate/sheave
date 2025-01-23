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
    #[arg(num_args(1..), long, value_name = "Address")]
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
    #[arg(long, value_enum, value_name = "LogLevel", default_value_t)]
    loglevel: LogLevel
    // TODO: Makes other options if they are required.
}

async fn run_as_rtmp(address: &str) -> IOResult<()> {
    let listener = RtmpListener::bind(address).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let server = Server::new(stream, RtmpContext::default(), PhantomData::<RtmpHandler<RtmpStream>>);
        return spawn(server).await?;
    }
}

#[tokio::main]
async fn main() -> IOResult<()> {
    let options = ServerOptions::parse();

    builder().filter_level(options.loglevel.into()).try_init().map_err(|e| IOError::other(e))?;

    if let Err(e) = run_as_rtmp(&options.listeners.rtmp[0]).await {
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
    fn err_missing_group() {
        let result = ServerOptions::command()
            .try_get_matches_from(vec!["sheave-server"]);
        assert!(result.is_err())
    }

    #[test]
    fn ok_presenting_hosts() {
        let result = ServerOptions::command()
            .try_get_matches_from(vec!["sheave-server", "--rtmp", "127.0.0.1:1935"]);
        assert!(result.is_ok());
        let result = ServerOptions::command()
            .try_get_matches_from(vec!["sheave-server", "--rtmp", "127.0.0.1:1935", "0.0.0.0:1935"]);
        assert!(result.is_ok())
    }
}
