pub mod net;
pub mod handlers;
pub mod server;

use std::io::Result as IOResult;
use clap::Parser;
use tokio::spawn;
use sheave_core::cli::*;
use self::{
    net::rtmp::RtmpListener,
    server::*
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct ServerOptions {
    #[arg(long, value_enum, default_value_t = Protocol::Rtmp)]
    protocol: Protocol,
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,
    #[arg(short, long, default_value_t = 1935)]
    port: u16
}

async fn run_as_rtmp(host: String, port: u16) -> IOResult<()> {
    let listener = RtmpListener::bind((host, port)).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let message_id = provide_message_id().await;
        let server = Server::new(stream, message_id);
        return spawn(server).await?;
    }
}

#[tokio::main]
async fn main() -> IOResult<()> {
    use Protocol::*;

    let options = ServerOptions::parse();
    match options.protocol {
        Rtmp => if let Err(e) = run_as_rtmp(options.address, options.port).await {
            println!("{e}")
        }
    }

    Ok(())
}
