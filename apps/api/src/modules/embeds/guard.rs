use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use url::{Host, Url};

/// How many redirects an unfurl follows.
pub const MAX_REDIRECTS: usize = 3;

fn public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    let reserved = a == 0
        || a >= 240
        || (a == 100 && (64..128).contains(&b))
        || (a == 192 && b == 0 && c == 0)
        || (a == 198 && (18..20).contains(&b));
    !(reserved
        || ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_unspecified())
}

fn public_v6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return public_v4(v4);
    }
    let first = ip.segments()[0];
    let special = (first & 0xfe00) == 0xfc00
        || (first & 0xffc0) == 0xfe80
        || (first & 0xffc0) == 0xfec0
        || first == 0x2002
        || (first == 0x2001 && ip.segments()[1] < 0x0200)
        || (first == 0x2001 && ip.segments()[1] == 0x0db8)
        || (first == 0x0064 && ip.segments()[1] == 0xff9b);
    !(special || ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() || first == 0)
}

/// Whether `ip` is a public unicast address: not loopback, private, link-local, CGNAT
/// (100.64/10), multicast, documentation, benchmarking, reserved, unique-local, NAT64, 6to4,
/// Teredo or an IPv4-mapped form of any of those.
pub fn public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => public_v4(ip),
        IpAddr::V6(ip) => public_v6(ip),
    }
}

/// Whether `url` may be fetched: `https`, a host, no credentials, and an IP literal only when
/// it is public. A name is checked again when it resolves.
pub fn allowed(url: &Url) -> bool {
    let literal_ok = match url.host() {
        Some(Host::Domain(name)) => !name.is_empty(),
        Some(Host::Ipv4(ip)) => public_v4(ip),
        Some(Host::Ipv6(ip)) => public_v6(ip),
        None => false,
    };
    url.scheme() == "https" && literal_ok && url.username().is_empty() && url.password().is_none()
}

/// A resolver that answers public addresses only.
pub(super) struct PublicOnly;

impl Resolve for PublicOnly {
    fn resolve(&self, name: Name) -> Resolving {
        Box::pin(async move {
            let found = tokio::net::lookup_host((name.as_str(), 0)).await?;
            let addrs: Vec<SocketAddr> = found.filter(|addr| public(addr.ip())).collect();
            if addrs.is_empty() {
                return Err("the host resolves to no public address".into());
            }
            Ok(Box::new(addrs.into_iter()) as Addrs)
        })
    }
}

fn redirect(attempt: reqwest::redirect::Attempt<'_>) -> reqwest::redirect::Action {
    match attempt.previous().len() > MAX_REDIRECTS || !allowed(attempt.url()) {
        true => attempt.stop(),
        false => attempt.follow(),
    }
}

/// The client for unfurls: every name resolves to public addresses only (so every redirect is
/// checked again), at most three `https` redirects, 5 s in all, no proxy from the environment.
pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .dns_resolver(Arc::new(PublicOnly))
        .redirect(reqwest::redirect::Policy::custom(redirect))
        .timeout(Duration::from_secs(5))
        .no_proxy()
        .user_agent("Docs-Unfurl/1 (+https://docs.facile.studio)")
        .build()
        .unwrap_or_default()
}
