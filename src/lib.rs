#![feature(ip)]

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::net::{ToSocketAddrs, UdpSocket};

use socket2::{Domain, Socket, Type};

/// The errors that can occur when trying to get IP address information.
#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    WrongIpVer(String, IpAddr),
    NoLinkLocal(Ipv6Addr),
    NoUla(Ipv6Addr),
    NoGua(Ipv6Addr),
    NoV4LL(Ipv4Addr),
    NoPrivate(Ipv4Addr, Ipv4Addr, Ipv4Addr),
    NoGlobal(Ipv4Addr),
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
            Self::NoLinkLocal(ip) => {
                write!(fmt, "ipv6 address {} is not a link-local address", ip)
            }
            Self::NoUla(ip) => write!(fmt, "ipv6 address {} is not a ula", ip),
            Self::NoGua(ip) => write!(fmt, "ipv6 address {} is not a gua", ip),
            Self::NoV4LL(ip) => {
                write!(fmt, "ipv4 address {} is not a link-local address", ip)
            }
            Self::NoPrivate(a, b, c) => {
                write!(fmt, "none of {}, {} and {} are private ipv4", a, b, c)
            }
            Self::NoGlobal(ip) => {
                write!(fmt, "ipv4 address {} is not a global address", ip)
            }
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

fn get_ipv6(interface: &str, network: &str) -> Result<Ipv6Addr> {
    let socket = Socket::new(Domain::IPV6, Type::DGRAM, None)?;
    let sock_addr = (network, 0).to_socket_addrs()?.next().unwrap();

    socket.bind_device(Some(interface.as_bytes()))?;
    socket.connect(&sock_addr.into())?;

    let udp: UdpSocket = socket.into();
    let ip = udp.local_addr()?.ip();

    match ip {
        IpAddr::V4(_) => Err(Error::WrongIpVer("IPv6".into(), ip)),
        IpAddr::V6(ipv6) => Ok(ipv6),
    }
}

/// Get the (preferred outgoing) IPv6 link-local address
/// of the given interface.
pub fn get_ipv6_unicast_link_local(interface: &str) -> Result<Ipv6Addr> {
    let ipv6 = get_ipv6(interface, "fe80::")?;

    if ipv6.is_unicast_link_local() {
        Ok(ipv6)
    } else {
        Err(Error::NoLinkLocal(ipv6))
    }
}

/// Get the preferred outgoing IPv6 ULA of the given interface.
pub fn get_ipv6_unique_local(interface: &str) -> Result<Ipv6Addr> {
    let ipv6 = get_ipv6(interface, "fc00::")?;

    if ipv6.is_unique_local() {
        Ok(ipv6)
    } else {
        Err(Error::NoUla(ipv6))
    }
}

/// Get the preferred outgoing IPv6 GUA of the given interface.
pub fn get_ipv6_unicast_global(interface: &str) -> Result<Ipv6Addr> {
    let ipv6 = get_ipv6(interface, "2000::")?;

    if ipv6.is_unicast_global() {
        Ok(ipv6)
    } else {
        Err(Error::NoGua(ipv6))
    }
}

fn get_ipv4(interface: &str, network: &str) -> Result<Ipv4Addr> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    let sock_addr = (network, 0).to_socket_addrs()?.next().unwrap();

    socket.bind_device(Some(interface.as_bytes()))?;
    socket.connect(&sock_addr.into())?;

    let udp: UdpSocket = socket.into();
    let ip = udp.local_addr()?.ip();

    match ip {
        IpAddr::V4(ipv4) => Ok(ipv4),
        IpAddr::V6(_) => Err(Error::WrongIpVer("IPv4".into(), ip)),
    }
}

/// Get the (preferred outgoing) IPv4 link-local address
/// of the given interface.
pub fn get_ipv4_link_local(interface: &str) -> Result<Ipv4Addr> {
    let ipv4 = get_ipv4(interface, "169.254.0.0")?;

    if ipv4.is_link_local() {
        Ok(ipv4)
    } else {
        Err(Error::NoV4LL(ipv4))
    }
}

/// Get the preferred outgoing IPv4 private address
/// of the given interface.
pub fn get_ipv4_private(interface: &str) -> Result<Ipv4Addr> {
    let a = get_ipv4(interface, "10.0.0.0")?;
    let b = get_ipv4(interface, "172.16.0.0")?;
    let c = get_ipv4(interface, "192.168.0.0")?;

    if c.is_private() {
        Ok(c)
    } else if b.is_private() {
        Ok(b)
    } else if a.is_private() {
        Ok(a)
    } else {
        Err(Error::NoPrivate(a, b, c))
    }
}

/// Get the preferred outgoing IPv4 global address
/// of the given interface.
pub fn get_ipv4_global(interface: &str) -> Result<Ipv4Addr> {
    let ipv4 = get_ipv4(interface, "0.0.0.0")?;

    if ipv4.is_global() {
        Ok(ipv4)
    } else {
        Err(Error::NoGlobal(ipv4))
    }
}
