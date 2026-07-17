//! Reverse DNS (`gethostbyaddr`) over the **system resolver** via `getnameinfo`,
//! mirroring ext/standard/dns.c `php_gethostbyaddr`: `inet_pton(AF_INET6)` is
//! tried first, then `AF_INET`, and the lookup uses `NI_NAMEREQD` so an address
//! without a PTR record reports "no name" rather than echoing the numeric form.
//! Going through the same libc as the oracle keeps results identical on the
//! same machine (/etc/hosts, mDNS, resolver order).

use std::mem::size_of;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::os::raw::c_char;

use libc::{getnameinfo, sockaddr, sockaddr_in, sockaddr_in6, socklen_t, NI_NAMEREQD};

/// Outcome of a PTR lookup, one arm per PHP-visible behaviour.
pub enum Reverse {
    /// Not a valid IPv4/IPv6 literal: PHP warns and returns `false`.
    Malformed,
    /// Valid address, no PTR record (`NI_NAMEREQD` failed): PHP returns the
    /// input string unchanged.
    NoName,
    /// The canonical host name.
    Name(Vec<u8>),
}

pub fn reverse_lookup(ip: &str) -> Reverse {
    // dns.c order: IPv6 first (an IPv4-mapped literal like `::ffff:1.2.3.4`
    // resolves through the v6 branch), then dotted-quad IPv4. Rust's parsers
    // accept/reject the same literals as inet_pton (no leading zeros, no
    // scope ids), so they stand in for it.
    if let Ok(v6) = ip.parse::<Ipv6Addr>() {
        // SAFETY: zeroed sockaddr_in6 == dns.c's memset; only family + addr set.
        let mut sa: sockaddr_in6 = unsafe { std::mem::zeroed() };
        sa.sin6_family = libc::AF_INET6 as _;
        sa.sin6_addr.s6_addr = v6.octets();
        return name_of(&sa as *const _ as *const sockaddr, size_of::<sockaddr_in6>());
    }
    if let Ok(v4) = ip.parse::<Ipv4Addr>() {
        let mut sa: sockaddr_in = unsafe { std::mem::zeroed() };
        sa.sin_family = libc::AF_INET as _;
        sa.sin_addr.s_addr = u32::from(v4).to_be();
        return name_of(&sa as *const _ as *const sockaddr, size_of::<sockaddr_in>());
    }
    Reverse::Malformed
}

fn name_of(sa: *const sockaddr, salen: usize) -> Reverse {
    // NI_MAXHOST (1025) — not exported by the libc crate on every target.
    let mut host = [0 as c_char; 1025];
    // SAFETY: sa points at a live sockaddr of salen bytes; host is writable.
    let rc = unsafe {
        getnameinfo(
            sa,
            salen as socklen_t,
            host.as_mut_ptr(),
            host.len() as _,
            std::ptr::null_mut(),
            0,
            NI_NAMEREQD,
        )
    };
    if rc != 0 {
        return Reverse::NoName;
    }
    let bytes: Vec<u8> =
        host.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect();
    Reverse::Name(bytes)
}
