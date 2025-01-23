pub mod handlers;
mod client;
mod invalid_uri;

use std::{
    io::{
        Error as IOError,
        Result as IOResult
    },
    marker::PhantomData,
    time::Duration
};
use log::{
    LevelFilter,
    error,
    info
};
use env_logger::builder;
use clap::{
    ArgAction,
    Parser,
    ValueEnum
};
use tokio::spawn;
use sheave_core::{
    flv::*,
    handlers::RtmpContext,
    net::rtmp::RtmpStream
};
use self::handlers::RtmpHandler;
pub use self::{
    client::Client,
    invalid_uri::*
};

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum FileFormat {
    #[default]
    Flv
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

/// Command line options for the Sheave Client.
///
/// # Required Arguments
///
/// * Input files and file formats (currently only `flv` is available)
/// * Output URI (currently only any remote host by RTMP is available)
///
/// `sheave-client -i test.flv -f flv rtmp://127.0.0.1`
/// `sheave-client --input test.flv --format flv rtmp://127.0.0.1`
///
/// (Truth be told, file format arguments would be put before input file names, but input files must be put before them. This is a specification of the clap.)
///
/// # Optional Arguments
///
/// * Timeout duration (in milliseconds)
///
/// `sheave-client -i test.flv -f flv -t 1000 rtmp://127.0.0.1`
/// `sheave-client --input test.flv --format flv --timeout 1000 rtmp://127.0.0.1`
#[derive(Debug, Parser)]
#[command(author, version)]
struct ClientOptions {
    /// Formats of input files. (required with input files)
    /// Following values are available.
    ///
    /// * flv
    #[arg(requires("input"), long, short, value_name = "Format", value_enum, action = ArgAction::Append)]
    format: Vec<FileFormat>,
    /// Input files. (required)
    /// Currently only A FLV file is allowed and plural files are ignored.
    /// Because of unimplemented muxing yet.
    #[arg(required(true), num_args(1..), long, short, value_name = "Input", action = ArgAction::Append)]
    input: Vec<String>,
    /// An await duration. (in milliseconds, default = 1000)
    /// While stream publication, the client may receive some message from the server, but not always.
    #[arg(long, short, value_name = "Duration", default_value_t = 1000)]
    await_duration: u64,
    /// The URI of destination.
    /// Currently only remote host via the RTMP stream is allowed.
    /// e.g. `rtmp://127.0.0.1`
    #[arg(value_name="URI")]
    uri: String,
    /// Displays client status in detail by logger.
    /// Correspondence of parameters to log kinds are following:
    ///
    /// |Parameter|Log Kind|
    /// | :- | :- |
    /// |`off`|Logs nothing.|
    /// |`error`|<ul><li>Cause of client/connection stopping.</li></ul>|
    /// |`warn`|<ul><li>Limit excess.</li><li>Insufficient parameter.</li></ul>|
    /// |`info`|<ul><li>Current process</li></ul>|
    /// |`debug`|<ul><li>Detailed data for debugging.</li></ul>|
    /// |`trace`|<ul><li>Detailed process for tracing.</li></ul>|
    #[arg(long, value_enum, value_name = "LogLevel", default_value_t)]
    loglevel: LogLevel
    // TODO: Makes other options if they are required.
}

fn split_uri<'a>(uri: &'a str) -> IOResult<(&'a str, &'a str, &'a str, &'a str)> {
    let protocol_len = match uri.find(':') {
        Some(protocol_len) => protocol_len,
        None => {
            error!("This isn't the URI: {uri}");
            return Err(invalid_uri(format!("This isn't the URI: {uri}")));
        }
    };
    let (protocol, rest) = uri.split_at(protocol_len);

    if !protocol.starts_with("rtmp") {
        error!("URI isn't started with protocol scheme: {protocol}");
        return Err(invalid_uri("URI isn't started with protocol scheme.".to_string()))
    }

    let (addr, rest) = if rest.starts_with("://") && rest.len() > 3 {
        let addr_len = rest[3..].find('/').unwrap_or(rest.len() - 3);
        rest[3..].split_at(addr_len)
    } else {
        error!("URI didn't contain the destination address: {rest}");
        return Err(invalid_uri("URI didn't contain the destination address.".to_string()))
    };

    let (app, rest) = if rest.len() > 1 {
        if let Some(app_len) = rest[1..].rfind('/') {
            rest[1..].split_at(app_len)
        } else {
            (&rest[1..], <&str>::default())
        }
    } else {
        (<&str>::default(), <&str>::default())
    };

    if app.split('/').count() > 2 {
        error!("The app part is exceeded two elemeents: {app}");
        return Err(invalid_uri("The app part is exceeded two elements.".to_string()))
    }

    let playpath = if rest.len() > 1 {
        &rest[1..]
    } else {
        <&str>::default()
    };

    Ok((protocol, addr, app, playpath))
}

async fn run_as_rtmp(input: Flv, addr: &str, app: &str, playpath: &str, tc_url: &str, await_duration: u64) -> IOResult<()> {
    let stream = RtmpStream::connect(addr).await?;

    let mut rtmp_context = RtmpContext::default();
    rtmp_context.set_app(app.into());
    rtmp_context.set_playpath(playpath.into());
    rtmp_context.set_tc_url(tc_url.into());
    rtmp_context.set_input(input);
    rtmp_context.set_await_duration(Duration::from_millis(await_duration));

    let client = Client::new(stream, rtmp_context, PhantomData::<RtmpHandler<RtmpStream>>);

    spawn(client).await?
}

// TODO: Muxing input files.
#[tokio::main]
async fn main() -> IOResult<()> {
    let options = ClientOptions::parse();

    builder().filter_level(options.loglevel.into()).try_init().map_err(|e| IOError::other(e))?;

    let input = match options.format[0] {
        FileFormat::Flv => Flv::open(&options.input[0])?,
    };

    let (protocol, addr, app, playpath) = split_uri(&options.uri)?;

    match protocol {
        "rtmp" => if let Err(e) = run_as_rtmp(input, addr, app, playpath, &options.uri, options.await_duration).await {
            error!("Some error got occurred: {e}");
            return Err(e)
        },
        _ => unimplemented!("Protocol: Currently RTMP only.")
    }

    info!("RTMP communication got completed.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use super::*;

    #[test]
    fn ok_scheme_localhost() {
        let result = split_uri("rtmp://localhost");
        assert!(result.is_ok());
        assert_eq!(("rtmp", "localhost", "", ""), result.unwrap())
    }

    #[test]
    fn ok_scheme_localhost_port() {
        let result = split_uri("rtmp://localhost:1935");
        assert!(result.is_ok());
        assert_eq!(("rtmp", "localhost:1935", "", ""), result.unwrap())
    }

    #[test]
    fn ok_scheme_localhost_port_app() {
        let result = split_uri("rtmp://localhost:1935/live");
        assert!(result.is_ok());
        assert_eq!(("rtmp", "localhost:1935", "live", ""), result.unwrap())
    }

    #[test]
    fn ok_scheme_localhost_port_app_playpath() {
        let result = split_uri("rtmp://localhost:1935/live/stream1");
        assert!(result.is_ok());
        assert_eq!(("rtmp", "localhost:1935", "live", "stream1"), result.unwrap())
    }

    #[test]
    fn err_not_uri() {
        let result = split_uri("/path/to/stream1");
        assert!(result.is_err())
    }

    #[test]
    fn err_protocol_only() {
        let result = split_uri("rtmp://");
        assert!(result.is_err())
    }

    #[test]
    fn err_missing_input_file() {
        let result = ClientOptions::command()
            .try_get_matches_from(vec!["sheave-client", "rtmp://localhost"]);
        assert!(result.is_err())
    }

    #[test]
    fn err_missing_input_format() {
        let result = ClientOptions::command()
            .try_get_matches_from(vec!["sheave-client", "-i", "test.flv", "rtmp://localhost"]);
        assert!(result.is_err());
        let result = ClientOptions::command()
            .try_get_matches_from(vec!["sheave-client", "-i", "test1.flv", "-f", "flv", "-i", "test2.flv", "rtmp://localhost"]);
        assert!(result.is_err())
    }

    #[test]
    fn ok_presenting_inputs() {
        let result = ClientOptions::command()
            .try_get_matches_from(vec!["sheave-client", "-i", "test.flv", "-f", "flv", "rtmp://localhost"]);
        assert!(result.is_ok());
        let result = ClientOptions::command()
            .try_get_matches_from(vec!["sheave-client", "-i", "test1.flv", "-f", "flv", "-i", "test2.flv", "-f", "flv", "rtmp://localhost"]);
        assert!(result.is_ok())
    }

    #[test]
    fn err_missing_output() {
        let result = ClientOptions::command()
            .try_get_matches_from(vec!["sheave-client", "-i", "test.flv", "-f", "flv"]);
        assert!(result.is_err())
    }
}
