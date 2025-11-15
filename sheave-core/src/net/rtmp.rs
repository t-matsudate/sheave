use std::{
    fmt::{
        Debug,
        Formatter,
        Result as FormatResult
    },
    io::{
        Error as IOError,
        IoSlice,
        IoSliceMut,
        Result as IOResult
    },
    net::{
        SocketAddr,
        TcpStream as StdStream
    },
    pin::Pin,
    task::{
        Context as FutureContext,
        Poll
    },
    time::Duration
};
use bytes::buf::BufMut;
use pin_project_lite::pin_project;
use tokio::{
    io::{
        AsyncRead,
        AsyncWrite,
        Interest,
        ReadBuf,
        Ready
    },
    net::{
        TcpStream as TokioStream,
        ToSocketAddrs,
        tcp::{
            OwnedReadHalf,
            OwnedWriteHalf,
            ReadHalf,
            WriteHalf
        }
    }
};

pin_project! {
    /// A stream for RTMP that wrapped Tokio's `TcpStream`.
    ///
    /// If you constructs this struct from some address, use `RtmpStream::connect("aaa.bbb.ccc.ddd:1935")`.
    /// Or if you do it from already created std's TcpStream, use `RtmpStream::from_std(std_stream)`
    pub struct RtmpStream {
        #[pin]
        tokio_stream: TokioStream
    }
}

impl RtmpStream {
    fn new(tokio_stream: TokioStream) -> Self {
        Self { tokio_stream }
    }

    /// Opens a RTMP connection to a remote host.
    ///
    /// When connection succeeded, this wraps tokio's TcpStream into RtmpStream.
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.connect)
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> IOResult<Self> {
        TokioStream::connect(addr).await.map(Self::new)
    }

    /// Creates new RtmpStream from a `std::net::TcpStream`.
    ///
    /// When connection succeeded, this wraps tokio's TcpStream into RtmpStream.
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.from_std)
    pub fn from_std(std_stream: StdStream) -> IOResult<Self> {
        TokioStream::from_std(std_stream).map(Self::new)
    }

    /// Turns a `sheave_core::net::rtmp::RtmpStream into `std::net::TcpStream`.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.into_std)
    pub fn into_std(self) -> IOResult<StdStream> {
        self.tokio_stream.into_std()
    }

    /// Returns the local address that this stream is bound to.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.local_addr)
    pub fn local_addr(&self) -> IOResult<SocketAddr> {
        self.tokio_stream.local_addr()
    }

    /// Returns the value of the `SO_ERROR` option.
    pub fn take_error(&self) -> IOResult<Option<IOError>> {
        self.tokio_stream.take_error()
    }

    /// Returns the remote address that this stream is connected to.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.peer_addr)
    pub fn peer_addr(&self) -> IOResult<SocketAddr> {
        self.tokio_stream.peer_addr()
    }

    /// Attempts to receive data on the socket, without removing that data from the queue, registering the current task for wakeup if data is not yet available.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.poll_peek)
    pub fn poll_peek(&self, cx: &mut FutureContext<'_>, buf: &mut ReadBuf<'_>) -> Poll<IOResult<usize>> {
        self.tokio_stream.poll_peek(cx, buf)
    }

    /// Waits for any of the requested ready states.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.ready)
    pub async fn ready(&self, interest: Interest) -> IOResult<Ready> {
        self.tokio_stream.ready(interest).await
    }

    /// Waits for the socket to become readable.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.readable)
    pub async fn readable(&self) -> IOResult<()> {
        self.tokio_stream.readable().await
    }

    /// Polls for read readiness.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.poll_read_ready)
    pub fn poll_read_ready(&self, cx: &mut FutureContext<'_>) -> Poll<IOResult<()>> {
        self.tokio_stream.poll_read_ready(cx)
    }

    /// Tries to read data from the stream into the provided buffer, returning how many bytes were read.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.try_read)
    pub fn try_read(&self, buf: &mut [u8]) -> IOResult<usize> {
        self.tokio_stream.try_read(buf)
    }

    /// Tries to read data from the stream into the provided buffers, returning how many bytes were read.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.try_read_vectored)
    pub fn try_read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> IOResult<usize> {
        self.tokio_stream.try_read_vectored(bufs)
    }

    /// Tries to read data from the stream into the provided buffer, advancing the buffer’s internal cursor, returning how many bytes were read.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.try_read_buf)
    pub fn try_read_buf<B: BufMut>(&self, buf: &mut B) -> IOResult<usize> {
        self.tokio_stream.try_read_buf(buf)
    }

    /// Waits for the socket to become writable.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.writable)
    pub async fn writable(&self) -> IOResult<()> {
        self.tokio_stream.writable().await
    }

    /// Polls for write readiness.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.poll_write_ready)
    pub fn poll_write_ready(&self, cx: &mut FutureContext<'_>) -> Poll<IOResult<()>> {
        self.tokio_stream.poll_write_ready(cx)
    }

    /// Tries to write several buffers to the stream, returning how many bytes were written.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.try_write)
    pub fn try_write(&self, buf: &[u8]) -> IOResult<usize> {
        self.tokio_stream.try_write(buf)
    }

    /// Tries to write several buffers to the stream, returning how many bytes were written.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.try_write_vectored)
    pub fn try_write_vectored(&self, bufs: &[IoSlice<'_>]) -> IOResult<usize> {
        self.tokio_stream.try_write_vectored(bufs)
    }

    /// Tries to read or write from the socket using a user-provided IO operation.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.try_io)
    pub fn try_io<R>(&self, interest: Interest, f: impl FnOnce() -> IOResult<R>) -> IOResult<R> {
        self.tokio_stream.try_io(interest, f)
    }

    /// Reads or writes from the socket using a user-provided IO operation.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.async_io)
    pub async fn async_io<R>(&self, interest: Interest, f: impl FnMut() -> IOResult<R>) -> IOResult<R> {
        self.tokio_stream.async_io(interest, f).await
    }

    /// Receives data on the socket from the remote address to which it is connected, without removing that data from the queue.
    /// On success, returns the number of bytes peeked.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.peek)
    pub async fn peek(&self, buf: &mut [u8]) -> IOResult<usize> {
        self.tokio_stream.peek(buf).await
    }

    /// Gets the value of the TCP_NODELAY option on this socket.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.nodelay)
    pub fn nodelay(&self) -> IOResult<bool> {
        self.tokio_stream.nodelay()
    }

    /// Sets the value of the TCP_NODELAY option on this socket.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.set_nodelay)
    pub fn set_nodelay(&self, nodelay: bool) -> IOResult<()> {
        self.tokio_stream.set_nodelay(nodelay)
    }

    /// Reads the linger duration for this socket by getting the SO_LINGER option.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.linger)
    pub fn linger(&self) -> IOResult<Option<Duration>> {
        self.tokio_stream.linger()
    }

    /// Sets the linger duration of this socket by setting the SO_LINGER option.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.set_linger)
    pub fn set_linger(&self, dur: Option<Duration>) -> IOResult<()> {
        self.tokio_stream.set_linger(dur)
    }

    /// Gets the value of the IP_TTL option for this socket.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.ttl)
    pub fn ttl(&self) -> IOResult<u32> {
        self.tokio_stream.ttl()
    }

    /// Sets the value for the IP_TTL option on this socket.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.set_ttl)
    pub fn set_ttl(&self, ttl: u32) -> IOResult<()> {
        self.tokio_stream.set_ttl(ttl)
    }

    /// Splits a TcpStream into a read half and a write half, which can be used to read and write the stream concurrently.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.split)
    pub fn split<'a>(&'a mut self) -> (ReadHalf<'a>, WriteHalf<'a>) {
        self.tokio_stream.split()
    }

    /// Splits a TcpStream into a read half and a write half, which can be used to read and write the stream concurrently.
    ///
    /// [Read more](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.into_split)
    pub fn into_split(self) -> (OwnedReadHalf, OwnedWriteHalf) {
        self.tokio_stream.into_split()
    }
}

impl TryFrom<StdStream> for RtmpStream {
    type Error = IOError;

    fn try_from(std_stream: StdStream) -> IOResult<Self> {
        Self::from_std(std_stream)
    }
}

impl From<TokioStream> for RtmpStream {
    fn from(tokio_stream: TokioStream) -> Self {
        Self::new(tokio_stream)
    }
}

impl AsyncRead for RtmpStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, buf: &mut ReadBuf<'_>) -> Poll<IOResult<()>> {
        let this = self.project();
        this.tokio_stream.poll_read(cx, buf)
    }
}

impl AsyncWrite for RtmpStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut FutureContext<'_>, buf: &[u8]) -> Poll<IOResult<usize>> {
        let this = self.project();
        this.tokio_stream.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<IOResult<()>> {
        let this = self.project();
        this.tokio_stream.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<IOResult<()>> {
        let this = self.project();
        this.tokio_stream.poll_shutdown(cx)
    }
}

impl Debug for RtmpStream {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        self.tokio_stream.fmt(f)
    }
}

#[cfg(unix)]
mod sys {
    use std::os::unix::prelude::*;
    use super::RtmpStream;

    impl AsRawFd for RtmpStream {
        fn as_raw_fd(&self) -> RawFd {
            self.tokio_stream.as_raw_fd()
        }
    }

    impl AsFd for RtmpStream {
        fn as_fd(&self) -> BorrowedFd<'_> {
            self.tokio_stream.as_fd()
        }
    }
}

#[cfg(any(all(doc, docsrs), windows))]
#[cfg_attr(docsrs, doc(cfg(windows)))]
mod sys {
    use tokio::os::windows::io::{
        AsRawSocket,
        AsSocket,
        BorrowedSocket,
        Rawsocket
    };
    use super::RtmpStream;

    impl AsRawSocket for RtmpStream {
        fn as_raw_socket(&self) -> RawSocket {
            self.tokio_stream.as_raw_socket()
        }
    }

    impl AsSocket for RtmpStream {
        fn as_sokcet(&self) -> BorrowedSocket<'_> {
            self.tokio_stream.as_socket()
        }
    }
}

#[cfg(all(tokio_unstable, target_os = "wasi"))]
mod sys {
    use std::os::wasi::prelude::*;
    use super::RtmpStream;

    impl AsRawFd for RtmpStream {
        fn as_raw_fd(&self) -> RawFd {
            self.tokio_stream.as_raw_fd()
        }
    }

    impl AsFd for RtmpStream {
        fn as_fd(&self) -> BorrowedFd<'_> {
            self.tokio_stream.as_fd()
        }
    }
}
