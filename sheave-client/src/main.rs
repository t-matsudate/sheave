pub mod handlers;
mod client;
mod invalid_uri;

use std::{
    fmt::{
        Display,
        Formatter,
        Result as FormatResult
    },
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
    handlers::{
        ClientType as CoreClientType,
        RtmpContext
    },
    messages::amf::v0::AmfString,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ClientType {
    Publisher,
    Subscriber
}

impl From<ClientType> for CoreClientType {
    fn from(client_type: ClientType) -> Self {
        match client_type {
            ClientType::Publisher => CoreClientType::Publisher,
            ClientType::Subscriber => CoreClientType::Subscriber
        }
    }
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum PublishingType {
    #[default]
    Live,
    Record,
    Append
}

impl Display for PublishingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        use PublishingType::*;

        match *self {
            Live => write!(f, "live"),
            Record => write!(f, "record"),
            Append => write!(f, "append")
        }
    }
}

/// Command line options for the Sheave Client.
///
/// # Required Arguments
///
/// * client type (either `publisher` or `subscriber`)
/// * publishing type (when client type is `publisher`)
/// * start time (when client type is `subscriber`)
/// * File formats
/// * input files (when client type is `publisher`)
/// * output file (when client type is `subscriber`)
/// * URI
///
/// `sheave-client --client-type publisher --publishing-type live -f flv -i test.flv -f flv rtmp://127.0.0.1/app/path`
/// `sheave-client --client-type publisher --publishing-type live --formatf flv --input test.flv --format flv rtmp://127.0.0.1/app/path`
/// `sheave-client --client-type subscriber --start-time -2 -f flv -o test.flv -f flv rtmp://127.0.0.1/app/path`
/// `sheave-client --client-type subscriber --start-time -2 --format flv --output test.flv --format flv rtmp://127.0.0.1/app/path`
///
/// # Optional Arguments
///
/// * loglevel
/// * awaiting duration
///
/// `sheave-client --client-type publisher --publishing-type live -f flv -i test.flv --await-duration 1000 --loglevel error rtmp://127.0.0.1/app/path`
/// `sheave-client --client-type publisher --publishing-type live --formatf flv --input test.flv --await-duration 1000 --loglevel error rtmp://127.0.0.1/app/path`
#[derive(Debug, Parser)]
#[command(author, version)]
struct ClientOptions {
    /// Displays client status in detail by logger.
    ///
    /// Correspondence of parameters to log kinds are following:
    ///
    /// |Parameter|Log Level|
    /// | :- | :- |
    /// |`off`|Logs nothing.|
    /// |`error`|<ul><li>Cause of client/connection stopping.</li></ul>|
    /// |`warn`|<ul><li>Limit excess.</li><li>Insufficient parameter.</li></ul>|
    /// |`info`|<ul><li>Current process</li></ul>|
    /// |`debug`|<ul><li>Detailed data for debugging.</li></ul>|
    /// |`trace`|<ul><li>Detailed process for tracing.</li></ul>|
    ///
    /// The default is `off`.
    #[arg(long, value_enum, value_name = "LogLevel", default_value_t)]
    loglevel: LogLevel,

    /// An awaiting duration (in milliseconds).
    ///
    /// While stream publication, the client may receive some message from the server, but not always.
    ///
    /// The default is `1000`.
    #[arg(long, value_name = "Duration", default_value_t = 1000)]
    await_duration: u64,

    /// Indicates whether this client requires to perform handshake with HMAC(SHA-256).
    #[arg(long)]
    signed: Option<bool>,

    /// Indicates whether this client performs either as a publisher or as a subscriber, to the server.
    ///
    /// * publisher: the client sends audio/video data to the server.
    /// * subscriber: the client received audio/video data from the server.
    #[arg(long, value_enum, value_name = "publisher / subscriber", required = true)]
    client_type: ClientType,

    /// Specifies of the publishing data lifetime, to the server (required if client_type is `publisher`).
    ///
    /// * live: Makes audio/video data treated as a live stream. It isn't stored into the server.
    /// * record: Makes audio/video data treated as a recorded file. If file is already stored, it is rewritten.
    /// * append: Makes audio/video data treated as a recorded file. If file is already stored, data is appended into the bottom of its file.
    ///
    /// The default is `live`.
    #[arg(long, value_enum, value_name = "live / record / append", required_if_eq("client_type", "publisher"), default_value_t)]
    publishing_type: PublishingType,

    /// Specifies an offset duration (in seconds, **signed**)
    ///
    /// If the value is and above 0, this client requests audio/video data as a recorded file after offsetting as much as specified value, to the server.
    /// Otherwise the value has following correspondence.
    ///
    /// |Value|Behavior|
    /// | :- | :- |
    /// |`-2`|The server sends it as a recorded file, if data aren't on live streams.|
    /// |`-1`|The server sends it as a live stream.|
    ///
    /// In either case, the server may send an error when audio/video data exists.
    /// The default is `-2`.
    #[arg(long, value_name = "Duration", required_if_eq("client_type", "subscriber"), allow_negative_numbers = true, default_value_t = -2)]
    start_time: i64,

    /// Formats of input files (required with input/output files).
    ///
    /// Following values are available.
    ///
    /// * flv
    #[arg(long, short, value_enum, action = ArgAction::Append, value_name = "File Format")]
    format: Vec<FileFormat>,

    /// Input files (required if client_type is `publisher`).
    ///
    /// Currently only a FLV file is allowed and plural files are ignored.
    /// Because of unimplemented muxing yet.
    #[arg(long, short, num_args = 1.., action = ArgAction::Append, value_name = "File", required_if_eq("client_type", "publisher"), requires = "format")]
    input: Vec<String>,

    /// Output files (required if client_type is `subscriber`).
    ///
    /// Currnetly only a FLV file is allowed and plural files are ignored.
    /// Because of unimplemented muxing yet.
    #[arg(long, short, num_args = 1.., action = ArgAction::Append, value_name = "File", required_if_eq("client_type", "subscriber"), requires = "format")]
    output: Vec<String>,

    /// The URI of destination.
    ///
    /// Currently only remote host via the RTMP stream is allowed.
    /// e.g. `rtmp://127.0.0.1/app/path`
    #[arg(value_name = "URI", requires = "format")]
    uri: String,
    // TODO: Makes other options if they are required.
}

fn split_uri(uri: &str) -> IOResult<(&str, &str, &str, &str)> {
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
        return Err(invalid_uri(format!("URI isn't started with protocol scheme: {protocol}")))
    }

    let (addr, rest) = if rest.starts_with("://") && rest.len() > 3 {
        let addr_len = rest[3..].find('/').unwrap_or(rest.len() - 3);
        rest[3..].split_at(addr_len)
    } else {
        error!("URI didn't contain the destination address: {rest}");
        return Err(invalid_uri(format!("URI didn't contain the destination address: {rest}")))
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
        error!("The app part is exceeded two elements: {app}");
        return Err(invalid_uri(format!("The app part is exceeded two elements: {app}")))
    }

    let topic_id = if rest.len() > 1 {
        &rest[1..]
    } else {
        <&str>::default()
    };

    Ok((protocol, addr, app, topic_id))
}

async fn run_as_rtmp(addr: &str, app: &str, topic_id: &str, options: ClientOptions) -> IOResult<()> {
    let stream = RtmpStream::connect(addr).await?;

    let mut rtmp_context = RtmpContext::default();
    rtmp_context.set_signed(options.signed.unwrap_or_default());
    rtmp_context.set_app(app);
    rtmp_context.set_topic_id(topic_id.into());
    rtmp_context.set_tc_url(&options.uri);

    let client_type: CoreClientType = options.client_type.into();
    match client_type {
        CoreClientType::Publisher => match options.format[0] {
            FileFormat::Flv => {
                let topic = Flv::open(&options.input[0])?;
                rtmp_context.set_topic(topic);

                rtmp_context.set_await_duration(Duration::from_millis(options.await_duration));

                rtmp_context.set_publishing_name(topic_id.into());
                rtmp_context.set_publishing_type(AmfString::new(options.publishing_type.to_string()));
            }
        },
        CoreClientType::Subscriber => match options.format[0] {
            FileFormat::Flv => {
                let topic = Flv::create(&options.output[0])?;
                rtmp_context.set_topic(topic);

                rtmp_context.set_stream_name(topic_id.into());
                if options.start_time >= 0 {
                    rtmp_context.set_start_time(Some(Duration::from_secs(options.start_time as u64)));
                }
                rtmp_context.set_play_mode(options.start_time.into());
            }
        }
    };
    rtmp_context.set_client_type(client_type);

    let client = Client::new(stream, rtmp_context, PhantomData::<RtmpHandler<RtmpStream>>);
    spawn(client).await?
}

// TODO: Muxing input files.
#[tokio::main]
async fn main() -> IOResult<()> {
    let options = ClientOptions::parse();

    builder().filter_level(options.loglevel.into()).try_init().map_err(|e| IOError::other(e))?;

    let uri = options.uri.clone();
    let (protocol, addr, app, topic_id) = split_uri(&uri)?;

    match protocol {
        "rtmp" => if let Err(e) = run_as_rtmp(addr, app, topic_id, options).await {
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
    fn ok_scheme_localhost_port_app_topic_id() {
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
    fn err_missing_client_type() {
        let result = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "-f", "flv",
                    "-i", "test.flv",
                    "rtmp://localhost"
                ]
            );
        assert!(result.is_err())
    }

    #[test]
    fn err_missing_publishing_type() {
        let result = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "publisher",
                    "-f", "flv",
                    "-i", "test.flv",
                    "rtmp://localhost"
                ]
            );
        assert!(result.is_err())
    }

    #[test]
    fn err_missing_input_format() {
        let result = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "publisher",
                    "--publishing-type", "live"
                ]
            );
        assert!(result.is_err())
    }

    #[test]
    fn err_missing_input_file() {
        let result = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "publisher",
                    "--publishing-type", "live",
                    "-f", "flv"
                ]
            );
        assert!(result.is_err())
    }

    #[test]
    fn err_missing_uri_in_publisher() {
        let result = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "publisher",
                    "--publishing-type", "live",
                    "-f", "flv",
                    "-i", "test.flv",
                    "-f", "flv"
                ]
            );
        assert!(result.is_err());
    }

    #[test]
    fn err_missing_start_time() {
        let result = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "subscriber"
                ]
            );
        assert!(result.is_err())
    }

    #[test]
    fn err_missing_output_format() {
        let result = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "subscriber",
                    "--start-time", "\"-2\""
                ]
            );
        assert!(result.is_err())
    }

    #[test]
    fn err_missing_output_file() {
        let result = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "subscriber",
                    "--start-time", "\"-2\"",
                    "-f", "flv"
                ]
            );
        assert!(result.is_err())
    }

    #[test]
    fn err_missing_uri_in_subscriber() {
        let result = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "subscriber",
                    "--start-time", "\"-2\"",
                    "-f", "flv",
                    "-o", "tesst.flv",
                    "-f", "flv"
                ]
            );
        assert!(result.is_err());
    }

    #[test]
    fn ok_presenting_inputs() {
        let single_file_as_publisher = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "publisher",
                    "--publishing-type", "live",
                    "-f", "flv",
                    "-i", "test.flv",
                    "-f", "flv",
                    "rtmp://localhost"
                ]
            );
        assert!(single_file_as_publisher.is_ok());

        let single_file_as_subscriber = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "subscriber",
                    "--start-time", "-2",
                    "-f", "flv",
                    "-o", "test.flv",
                    "-f", "flv",
                    "rtmp://localhost"
                ]
            );
        assert!(single_file_as_subscriber.is_ok(), "{}", single_file_as_subscriber.err().unwrap());

        let plural_files_as_publisher = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "publisher",
                    "--publishing-type", "live",
                    "-f", "flv",
                    "-i", "test1.flv",
                    "-f", "flv",
                    "-i", "test2.flv",
                    "-f", "flv",
                    "rtmp://localhost"
                ]
            );
        assert!(plural_files_as_publisher.is_ok());

        let plural_files_as_subscriber = ClientOptions::command()
            .try_get_matches_from(
                vec![
                    "sheave-client",
                    "--client-type", "subscriber",
                    "--start-time", "-2",
                    "-f", "flv",
                    "-o", "test1.flv",
                    "-f", "flv",
                    "-o", "test2.flv",
                    "-f", "flv",
                    "rtmp://localhost"
                ]
            );
        assert!(plural_files_as_subscriber.is_ok())
    }
}
