#![allow(unexpected_cfgs)]

pub mod net;
pub mod handlers;
mod server;
mod invalid_uri;

use std::{
    env::temp_dir,
    fs::exists,
    io::{
        Error as IOError,
        Result as IOResult
    },
    marker::PhantomData,
    path::Path
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
use dotenvy::dotenv;
use tokio::spawn;

#[cfg(feature = "sqlite")]
use sqlx::SqliteConnection as Connector;
#[cfg(feature = "mysql")]
use sqlx::MySqlConnection as Connector;
#[cfg(feature = "postgres")]
use sqlx::PgConnection as Connector;

use sqlx::{
    Connection,
    migrate::Migrator
};

use sheave_core::{
    handlers::RtmpContext,
    net::rtmp::RtmpStream
};
use self::{
    handlers::RtmpHandler,
    net::rtmp::RtmpListener
};
pub use self::{
    server::Server,
    invalid_uri::*
};

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
/// * The topic database URL
///
/// `sheave-server --listeners rtmp://127.0.0.1 --database-url sqlite::memory:`
/// `sheave-server --listeners rtmp://127.0.0.1:1935` --database-url sqlite::memory:
/// `sheave-server --listeners rtmp://127.0.0.1:1935/live --database-url sqlite::memory:`
#[derive(Debug, Parser)]
#[command(author, version)]
struct ServerOptions {
    /// Displays server status in detail by logger.
    ///
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
    #[arg(long, value_enum, value_name = "LogLevel", env = "LOGLEVEL", default_value_t)]
    loglevel: LogLevel,

    /// Specifies the path of migration files.
    ///
    /// If present, the Sheave Server runs migration files in specified path.
    /// Currnetly this is used for testing with the topic database.
    #[arg(long, value_name = "Path", env = "MIGRATIONS_PATH")]
    migrations_path: Option<String>,

    #[arg(long, value_name = "Fixtures", value_delimiter = ',', action = ArgAction::Append, requires = "migrations_path", env = "FIXTURES")]
    fixtures: Vec<String>,

    /// Listening URIs which starts with protocol schemas of the RTMP.
    ///
    /// Currently only `rtmp` schema is available.
    /// Following URI format is required.
    ///
    /// `rtmp://{address}[:port]/[app_name]`
    ///
    /// For example:
    ///
    /// * `rtmp://127.0.0.1`
    /// * `rtmp://127.0.0.1:1935`
    /// * `rtmp://127.0.0.1/live`
    /// * `rtmp://127.0.0.1:1935/live`
    ///
    /// Note that URIs are appended the port of `1935` as defaults if they are without ports.
    #[arg(long, value_name = "URIs", num_args = 1.., value_delimiter = ',', action = ArgAction::Append, required = true, env = "LISTENERS")]
    listeners: Vec<String>,

    /// The database URL to keep the topic path to handle topics.
    ///
    /// This must start with one of database URL schemas. (e.g. mysql:, postgres:, sqlite:, etc.)
    ///
    /// When you store topics into SQLite database, you can use in-memory storage URL (`:memory:`).
    #[arg(long, value_name = "URL", env = "DATABASE_URL", required = true)]
    database_url: String,

    /// The path to the base directory for storing topics.
    ///
    /// If this isn't present, the server set this to TEMP(windows)/TMPDIR(linux) environment variable.
    #[arg(long, value_name = "Path", env = "STORAGE_PATH")]
    storage_path: Option<String>,
    // TODO: Makes other options if they are required.
}

fn split_uri(uri: &str) -> IOResult<(&str, &str, Option<&str>)> {
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

    let (server_addr, rest) = if rest.starts_with("://") && rest.len() > 3 {
        let addr_len = rest[3..].find('/').unwrap_or(rest.len() - 3);
        rest[3..].split_at(addr_len)
    } else {
        error!("URI didn't contain the destination address: {rest}");
        return Err(invalid_uri(format!("URI didn't contain the destination address: {rest}")))
    };

    let app = if rest.len() > 1 {
        if rest[1..].split('/').count() > 2 {
            error!("The app part is exceeded two elements: {rest}");
            return Err(invalid_uri(format!("The app part is exceeded two elements: {rest}")))
        } else {
            Some(&rest[1..])
        }
    } else {
        None
    };

    Ok((protocol, server_addr, app))
}

async fn run_as_rtmp(server_addr: &str, app: Option<&str>, options: ServerOptions) -> IOResult<()> {
    /* NOTE: Addresses can be specified without ports. */
    let server_addr = if let Some(_) = server_addr.rfind(':') {
        server_addr.to_string()
    } else {
        format!("{server_addr}:1935")
    };
    let listener = RtmpListener::bind(&server_addr).await?;

    loop {
        let (stream, client_addr) = listener.accept().await?;
        let mut rtmp_context = RtmpContext::default();
        rtmp_context.set_database_url(&options.database_url);
        let temp_dir = temp_dir().to_str().map(String::from);
        rtmp_context.set_storage_path(&options.storage_path.or(temp_dir).unwrap());
        rtmp_context.set_app(app.unwrap_or_default());
        rtmp_context.set_client_addr(client_addr);

        let server = Server::new(stream, rtmp_context, PhantomData::<RtmpHandler<RtmpStream>>);
        return spawn(server).await?;
    }
}

#[tokio::main]
async fn main() -> IOResult<()> {
    /* NOTE: Makes sure to set enviromnent variables into CLI options as defaults. (if dotenv file is present) */
    if exists(".env")? {
        dotenv().map_err(IOError::other)?;
    }

    let options = ServerOptions::parse();

    builder().filter_level(options.loglevel.into()).try_init().map_err(|e| IOError::other(e))?;

    if let Some(migrations_path) = options.migrations_path.clone() {
        let mut connection = Connector::connect(&options.database_url).await.map_err(IOError::other)?;
        let migrator = Migrator::new(Path::new(&migrations_path)).await.map_err(IOError::other)?;
        migrator.run(&mut connection).await.map_err(IOError::other)?;
        info!("Migrations in {migrations_path} got ran.");
    }

    let listener = options.listeners[0].clone();
    let (protocol, server_addr, app) = split_uri(&listener)?;

    match protocol {
        "rtmp" => if let Err(e) = run_as_rtmp(server_addr, app, options).await {
            error!("Some error got occurred: {e}");
            return Err(e)
        },
        _ => unimplemented!("Other protocols.")
    }

    info!("RTMP communication got completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env::remove_var;
    use clap::CommandFactory;
    use super::*;

    #[test]
    fn err_no_parameter() {
        unsafe {
            remove_var("LISTENERS");
            remove_var("DATABASE_URL");
        }

        let result = ServerOptions::command()
            .try_get_matches_from(vec!["sheave-server"]);
        assert!(result.is_err())
    }

    #[test]
    fn err_missing_database_url() {
        unsafe {
            remove_var("DATABASE_URL");
        }

        let result = ServerOptions::command()
            .try_get_matches_from(vec!["sheave-server", "--listeners", "rtmp://127.0.0.1:1935"]);
        assert!(result.is_err())
    }

    #[test]
    fn ok_passing_required_parameters() {
        let single_listener = ServerOptions::command()
            .try_get_matches_from(vec!["sheave-server", "--listeners", "rtmp://127.0.0.1:1935", "--database-url", "sqlite::memory:"]);
        assert!(single_listener.is_ok());
        let plural_listeners = ServerOptions::command()
            .try_get_matches_from(vec!["sheave-server", "--listeners", "rtmp://127.0.0.1:1935,rtmp://0.0.0.0:1935", "--database-url", "sqlite::memory:"]);
        assert!(plural_listeners.is_ok())
    }
}
