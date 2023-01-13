// Unless explicitly stated otherwise all files in this repository are licensed
// under the MIT/Apache-2.0 License, at your convenience
//
// This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2020 Datadog, Inc.
//
use super::datagram::GlommioDatagram;
use nix::sys::socket::{InetAddr, SockAddr};
use socket2::{Domain, Protocol, Socket, Type};
use std::{
    io,
    net::{self, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs},
    os::unix::io::{AsRawFd, FromRawFd, RawFd},
    time::Duration,
};

type Result<T> = crate::Result<T, ()>;

#[derive(Debug)]
/// A raw Socket.
pub struct RawSocket {
    socket: GlommioDatagram<Socket>,
}

impl From<socket2::Socket> for RawSocket {
    fn from(socket: socket2::Socket) -> RawSocket {
        Self {
            socket: GlommioDatagram::<Socket>::from(socket),
        }
    }
}

impl AsRawFd for RawSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.socket.as_raw_fd()
    }
}

impl FromRawFd for RawSocket {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        let socket = socket2::Socket::from_raw_fd(fd);
        RawSocket::from(socket)
    }
}

impl RawSocket {
    pub fn new<A: ToSocketAddrs>(
        addr: A,
        domain: Domain,
        ty: Type,
        proto: Option<Protocol>,
    ) -> Result<RawSocket> {
        let addr = addr
            .to_socket_addrs()
            .unwrap()
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "empty address"))?;
        let sk = Socket::new(domain, ty, proto)?;
        sk.set_nonblocking(true)?;
        let addr = socket2::SockAddr::from(addr);
        sk.bind(&addr)?;
        Ok(Self {
            socket: GlommioDatagram::from(sk),
        })
    }

    /// Sets the buffer size used on the receive path
    pub fn set_buffer_size(&mut self, buffer_size: usize) {
        self.socket.rx_buf_size = buffer_size;
    }

    /// gets the buffer size used
    pub fn buffer_size(&mut self) -> usize {
        self.socket.rx_buf_size
    }

    pub async fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> Result<usize> {
        let addr = addr
            .to_socket_addrs()
            .unwrap()
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "empty address"))?;

        let inet = nix::sys::socket::InetAddr::from_std(&addr);
        let sockaddr = nix::sys::socket::SockAddr::new_inet(inet);
        self.socket.send_to(buf, sockaddr).await.map_err(Into::into)
    }
}
