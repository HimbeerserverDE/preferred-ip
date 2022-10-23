#![feature(ip)]

use std::fmt;
use std::net::{IpAddr, Ipv6Addr};
use std::net::{ToSocketAddrs, UdpSocket};

use socket2::{Domain, Socket, Type};

/// The errors that can occur when trying to get IP address information.
#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    WrongIpVer(String, IpAddr),
    NoGua(Ipv6Addr),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => {
                write!(fmt, "can't get ip address: io error: {}", e)
            }
            Self::WrongIpVer(want, got) => {
                write!(fmt, "wrong ip version: expected {}, got {}", want, got)
            }
            Self::NoGua(ip) => write!(fmt, "ipv6 address {} is not a gua", ip),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

/// An alias for `std::result::Result` that uses `Error` as its error variant.
pub type Result<T> = std::result::Result<T, Error>;

/// Get the preferred outgoing IPv6 GUA of the given interface.
pub fn get_ipv6_global(interface: &str) -> Result<Ipv6Addr> {
    let socket = Socket::new(Domain::IPV6, Type::DGRAM, None)?;
    let sock_addr = ("2000::", 0).to_socket_addrs()?.next().unwrap();

    socket.bind_device(Some(interface.as_bytes()))?;
    socket.connect(&sock_addr.into())?;

    let udp: UdpSocket = socket.into();
    let ip = udp.local_addr()?.ip();

    match ip {
        IpAddr::V4(_) => Err(Error::WrongIpVer("IPv6".into(), ip)),
        IpAddr::V6(ipv6) => {
            if ipv6.is_unicast_global() {
                Ok(ipv6)
            } else {
                Err(Error::NoGua(ipv6))
            }
        }
    }
}
