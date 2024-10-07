#![allow(unexpected_cfgs)]

pub mod net;
pub mod handlers;
mod server;

use std::{
    io::Result as IOResult,
    marker::PhantomData
};
use clap::{
    Args,
    Parser
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

// TODO: Runs server actually.
#[tokio::main]
async fn main() -> IOResult<()> {
    let options = ServerOptions::parse();
    if let Err(e) = run_as_rtmp(&options.listeners.rtmp[0]).await {
        println!("{e}");
    }

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
