pub mod handlers;
pub mod client;

use std::io::Result as IOResult;
use clap::Parser;
use tokio::spawn;
use sheave_core::{
    cli::*,
    net::rtmp::RtmpStream
};
use self::client::Client;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct ClientOptions {
    #[arg(long, value_enum, default_value_t = Protocol::Rtmp)]
    protocol: Protocol,
    #[arg(short, long, default_value = "localhost")]
    host: String,
    #[arg(short, long, default_value_t = 1935)]
    port: u16
}

async fn run_as_rtmp(host: String, port: u16) -> IOResult<()> {
    let stream = RtmpStream::connect((host, port)).await?;
    let client = Client::new(stream);

    spawn(client).await?
}

#[tokio::main]
async fn main() -> IOResult<()> {
    use Protocol::*;

    let options = ClientOptions::parse();
    match options.protocol {
        Rtmp => if let Err(e) = run_as_rtmp(options.host, options.port).await {
            println!("{e}")
        }
    }

    Ok(())
}
