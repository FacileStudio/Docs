use std::net::IpAddr;

use reqwest::dns::{Name, Resolve};
use url::Url;

use super::guard::{PublicOnly, allowed, public};

fn ip(raw: &str) -> IpAddr {
    raw.parse().unwrap()
}

#[test]
fn only_public_addresses_pass() {
    for raw in ["127.0.0.1", "10.1.2.3", "172.16.0.1", "192.168.1.1", "169.254.169.254", "100.64.0.1"] {
        assert!(!public(ip(raw)), "{raw}");
    }
    for raw in ["0.0.0.0", "224.0.0.1", "255.255.255.255", "198.18.0.1", "240.0.0.1", "192.0.0.8"] {
        assert!(!public(ip(raw)), "{raw}");
    }
    for raw in ["::1", "::", "fc00::1", "fd12::1", "fe80::1", "::ffff:127.0.0.1", "::ffff:10.0.0.1"] {
        assert!(!public(ip(raw)), "{raw}");
    }
    for raw in ["64:ff9b::7f00:1", "2002:7f00:1::", "2001::1", "2001:db8::1", "ff02::1"] {
        assert!(!public(ip(raw)), "{raw}");
    }
    for raw in ["1.1.1.1", "100.128.0.1", "93.184.216.34", "2606:4700::1111", "::ffff:8.8.8.8"] {
        assert!(public(ip(raw)), "{raw}");
    }
}

#[test]
fn only_https_urls_without_private_literals_or_credentials_pass() {
    let url = |raw: &str| Url::parse(raw).unwrap();
    assert!(allowed(&url("https://example.com/a")));
    assert!(allowed(&url("https://1.1.1.1/")));
    for raw in ["http://example.com", "https://127.0.0.1/", "https://[::1]/", "https://u:p@example.com/"] {
        assert!(!allowed(&url(raw)), "{raw}");
    }
    assert!(!allowed(&url("https://0x7f.1/")), "a hex IPv4 parses to 127.0.0.1");
    assert!(!allowed(&url("file:///etc/passwd")));
}

#[tokio::test]
async fn a_name_that_resolves_to_loopback_is_refused() {
    let answer = PublicOnly.resolve("localhost".parse::<Name>().unwrap()).await;
    assert!(answer.is_err());
}
