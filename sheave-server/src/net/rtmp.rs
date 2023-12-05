use std::{
    io::{
        Error as IOError,
        Result as IOResult
    },
    net::{
        SocketAddr,
        TcpListener as StdListener
    },
    task::{
        Context,
        Poll
    }
};
use futures::ready;
use tokio::net::{
    TcpListener as TokioListener,
    ToSocketAddrs
};
use sheave_core::net::rtmp::*;

#[derive(Debug)]
pub struct RtmpListener {
    tokio_listener: TokioListener
}

impl RtmpListener {
    fn new(tokio_listener: TokioListener) -> Self {
        Self { tokio_listener }
    }

    pub async fn bind<A: ToSocketAddrs>(addr: A) -> IOResult<Self> {
        TokioListener::bind(addr).await.map(Self::new)
    }

    pub async fn accept(&self) -> IOResult<(RtmpStream, SocketAddr)> {
        let (tokio_stream, addr) = self.tokio_listener.accept().await?;
        Ok((RtmpStream::from(tokio_stream), addr))
    }

    pub fn poll_accept(&self, cx: &mut Context<'_>) -> Poll<IOResult<(RtmpStream, SocketAddr)>> {
        let (tokio_stream, addr) = ready!(self.tokio_listener.poll_accept(cx))?;
        Poll::Ready(Ok((RtmpStream::from(tokio_stream), addr)))
    }

    pub fn from_std(std_listener: StdListener) -> IOResult<Self> {
        TokioListener::from_std(std_listener).map(Self::new)
    }

    pub fn into_std(self) -> IOResult<StdListener> {
        self.tokio_listener.into_std()
    }

    pub fn local_addr(&self) -> IOResult<SocketAddr> {
        self.tokio_listener.local_addr()
    }

    pub fn ttl(&self) -> IOResult<u32> {
        self.tokio_listener.ttl()
    }

    pub fn set_ttl(&self, ttl: u32) -> IOResult<()> {
        self.tokio_listener.set_ttl(ttl)
    }
}

impl TryFrom<StdListener> for RtmpListener {
    type Error = IOError;

    fn try_from(std_listener: StdListener) -> IOResult<Self> {
        Self::from_std(std_listener)
    }
}

#[cfg(unix)]
mod sys {
    use std::os::unix::prelude::*;
    use super::RtmpListener;

    impl AsRawFd for RtmpListener {
        fn as_raw_fd(&self) -> RawFd {
            self.tokio_listener.as_raw_fd()
        }
    }

    impl AsFd for RtmpListener {
        fn as_fd(&self) -> BorrowedFd<'_> {
            self.tokio_listener.as_fd()
        }
    }
}

#[cfg(any(all(doc, docsrs), windows))]
#[cdg_attr(docsrs, doc(cfg(windows)))]
mod sys {
    use tokio::os::windows::io::{
        AsRawSocket,
        AsSocket,
        BorrowedSocket,
        Rawsocket
    };
    use super::RtmpListener;

    impl AsRawSocket for RtmpListener {
        fn as_raw_socket(&self) -> RawSocket {
            self.tokio_listener.as_raw_socket()
        }
    }

    impl AsSocket for RtmpListener {
        fn as_socket(&self) -> BorrowedFd<'_> {
            self.tokio_listener.as_socket()
        }
    }
}

#[cfg(all(tokio_unstable, target_os = "wasi"))]
#[cfg_attr(docsrs, doc(cfg(tokio_unstable)))]
mod sys {
    use std::os::wasi::prelude::*;
    use super::RtmpListener;

    impl AsRawFd for RtmpListener {
        fn as_raw_fd(&self) -> RawFd {
            self.tokio_listener.as_raw_fd()
        }
    }

    impl AsFd for RtmpListener {
        fn as_fd(&self) -> BorrowedFd<'_> {
            self.tokio_listener.as_fd()
        }
    }
}
