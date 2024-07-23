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
    #[arg(long, value_name = "Address", default_values_t=[String::from("127.0.0.1:1935")])]
    rtmp: Vec<String>
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
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

#[tokio::main]
async fn main() -> IOResult<()> {
    let options = ServerOptions::parse();
    if let Err(e) = run_as_rtmp(&options.listeners.rtmp[0]).await {
        println!("{e}");
    }

    Ok(())
}
