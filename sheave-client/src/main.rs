pub mod handlers;
mod client;
mod invalid_uri;

use std::{
    io::Result as IOResult,
    marker::PhantomData
};
use clap::Parser;
use tokio::spawn;
use sheave_core::{
    handlers::RtmpContext,
    net::rtmp::RtmpStream
};
use self::handlers::RtmpHandler;
pub use self::{
    client::*,
    invalid_uri::*
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct ClientOptions {
    #[arg(value_name="URL")]
    uri: String,
    // TODO: Makes other options if they are required.
}

fn split_uri<'a>(uri: &'a str) -> IOResult<(&'a str, &'a str, &'a str, &'a str)> {
    let protocol_len = uri.find(':').ok_or(invalid_uri(format!("This isn't the URI: {uri}")))?;
    let (protocol, rest) = uri.split_at(protocol_len);

    if !protocol.starts_with("rtmp") {
        return Err(invalid_uri("URI isn't started with protocol scheme.".to_string()))
    }

    let (addr, rest) = if rest.starts_with("://") {
        let addr_len = rest[3..].find('/').unwrap_or(rest.len());
        rest.split_at(addr_len)
    } else {
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
        return Err(invalid_uri("the app part is exceeded two elements.".to_string()))
    }

    let playpath = if rest.len() > 1 {
        &rest[1..]
    } else {
        <&str>::default()
    };

    Ok((protocol, addr, app, playpath))
}

async fn run_as_rtmp(addr: &str, app: &str, playpath: &str, tc_url: &str) -> IOResult<()> {
    let stream = RtmpStream::connect(addr).await?;
    let mut rtmp_context = RtmpContext::default();
    rtmp_context.set_app(app.into());
    rtmp_context.set_playpath(playpath.into());
    rtmp_context.set_tc_url(tc_url.into());
    let client = Client::new(stream, rtmp_context, PhantomData::<RtmpHandler<RtmpStream>>);
    spawn(client).await?
}

#[tokio::main]
async fn main() -> IOResult<()> {
    let options = ClientOptions::parse();
    let (protocol, addr, app, playpath) = split_uri(&options.uri)?;

    match protocol {
        "rtmp" => run_as_rtmp(addr, app, playpath, &options.uri).await?,
        _ => unimplemented!("Protocol: Currently RTMP only.")
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_scheme_localhost() {
        let result = split_uri("rtmp://localhost");
        assert!(result.is_ok())
    }

    #[test]
    fn ok_scheme_localhost_port() {
        let result = split_uri("rtmp://localhost:1935");
        assert!(result.is_ok())
    }

    #[test]
    fn ok_scheme_localhost_port_app() {
        let result = split_uri("rtmp://localhost:1935/live");
        assert!(result.is_ok())
    }

    #[test]
    fn ok_scheme_localhost_port_app_playpath() {
        let result = split_uri("rtmp://localhost:1935/live/stream1");
        assert!(result.is_ok())
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
}
