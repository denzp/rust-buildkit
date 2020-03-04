use std::io::{self, stdin, stdout};
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project::pin_project;
use tokio::io::*;
use tonic::transport::Uri;

#[pin_project]
pub struct StdioSocket {
    #[pin]
    reader: PollEvented<async_stdio::EventedStdin>,

    #[pin]
    writer: PollEvented<async_stdio::EventedStdout>,
}

pub async fn stdio_connector(_: Uri) -> io::Result<StdioSocket> {
    StdioSocket::try_new()
}

impl StdioSocket {
    pub fn try_new() -> io::Result<Self> {
        Ok(StdioSocket {
            reader: PollEvented::new(async_stdio::EventedStdin::try_new(stdin())?)?,
            writer: PollEvented::new(async_stdio::EventedStdout::try_new(stdout())?)?,
        })
    }
}

impl AsyncRead for StdioSocket {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        self.project().reader.poll_read(cx, buf)
    }
}

impl AsyncWrite for StdioSocket {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        self.project().writer.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().writer.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().writer.poll_shutdown(cx)
    }
}

mod async_stdio {
    use std::io::{self, Read, Stdin, Stdout, Write};
    use std::os::unix::io::AsRawFd;

    use mio::event::Evented;
    use mio::unix::EventedFd;
    use mio::{Poll, PollOpt, Ready, Token};

    use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

    pub struct EventedStdin(Stdin);
    pub struct EventedStdout(Stdout);

    impl EventedStdin {
        pub fn try_new(stdin: Stdin) -> io::Result<Self> {
            set_non_blocking_flag(&stdin)?;

            Ok(EventedStdin(stdin))
        }
    }

    impl EventedStdout {
        pub fn try_new(stdout: Stdout) -> io::Result<Self> {
            set_non_blocking_flag(&stdout)?;

            Ok(EventedStdout(stdout))
        }
    }

    impl Evented for EventedStdin {
        fn register(
            &self,
            poll: &Poll,
            token: Token,
            interest: Ready,
            opts: PollOpt,
        ) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).register(poll, token, interest, opts)
        }

        fn reregister(
            &self,
            poll: &Poll,
            token: Token,
            interest: Ready,
            opts: PollOpt,
        ) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).reregister(poll, token, interest, opts)
        }

        fn deregister(&self, poll: &Poll) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).deregister(poll)
        }
    }

    impl Read for EventedStdin {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl Evented for EventedStdout {
        fn register(
            &self,
            poll: &Poll,
            token: Token,
            interest: Ready,
            opts: PollOpt,
        ) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).register(poll, token, interest, opts)
        }

        fn reregister(
            &self,
            poll: &Poll,
            token: Token,
            interest: Ready,
            opts: PollOpt,
        ) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).reregister(poll, token, interest, opts)
        }

        fn deregister(&self, poll: &Poll) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).deregister(poll)
        }
    }

    impl Write for EventedStdout {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0.flush()
        }
    }

    fn set_non_blocking_flag<T: AsRawFd>(stream: &T) -> io::Result<()> {
        let flags = unsafe { fcntl(stream.as_raw_fd(), F_GETFL, 0) };

        if flags < 0 {
            return Err(std::io::Error::last_os_error());
        }

        if unsafe { fcntl(stream.as_raw_fd(), F_SETFL, flags | O_NONBLOCK) } != 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }
}
