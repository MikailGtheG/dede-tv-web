//! Remote access: WireGuard devices that may reach this console from outside.
//!
//! The root helper (dede-admin-apply) keeps the keys and writes the public
//! state to `/etc/dede-tv/remote.conf`:
//!
//! ```text
//! WG_ENDPOINT=haus.example.org:51820
//! WG_DEVICE=10.77.0.2 <public key> Handy Mikail
//! ```
use std::fs;
use std::path::Path;

/// UDP port of the TV's WireGuard interface; the router forwards to it.
pub const WG_PORT: u16 = 51820;
pub const NAME_MAX: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub address: String,
    pub name: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct State {
    pub endpoint: Option<String>,
    pub devices: Vec<Device>,
}

pub fn read(path: &Path) -> State {
    let mut state = State::default();
    let Ok(text) = fs::read_to_string(path) else {
        return state;
    };
    for line in text.lines() {
        if let Some(endpoint) = line.strip_prefix("WG_ENDPOINT=") {
            state.endpoint = valid_endpoint(endpoint).then(|| endpoint.to_string());
        } else if let Some(device) = line.strip_prefix("WG_DEVICE=") {
            let mut parts = device.splitn(3, ' ');
            if let (Some(address), Some(_key), Some(name)) =
                (parts.next(), parts.next(), parts.next())
            {
                if valid_address(address) && valid_name(name) {
                    state.devices.push(Device {
                        address: address.into(),
                        name: name.into(),
                    });
                }
            }
        }
    }
    state
}

/// Device addresses handed out by the helper: 10.77.0.2 to 10.77.0.254.
pub fn valid_address(address: &str) -> bool {
    address.strip_prefix("10.77.0.").is_some_and(|host| {
        host.parse::<u8>()
            .is_ok_and(|n| (2..=254).contains(&n) && n.to_string() == host)
    })
}

/// Letters (any script), digits, blanks inside, `-`, `_` and `.`.
pub fn valid_name(name: &str) -> bool {
    let count = name.chars().count();
    (1..=NAME_MAX).contains(&count)
        && name.trim() == name
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'))
}

/// `host` or `host:port`: a DNS name, an IPv4 address or `[IPv6]`.
pub fn valid_endpoint(endpoint: &str) -> bool {
    let (host, port) = if let Some(rest) = endpoint.strip_prefix('[') {
        let Some((v6, tail)) = rest.split_once(']') else {
            return false;
        };
        if v6.parse::<std::net::Ipv6Addr>().is_err() {
            return false;
        }
        match tail {
            "" => return true,
            _ => match tail.strip_prefix(':') {
                Some(port) => (None, Some(port)),
                None => return false,
            },
        }
    } else {
        match endpoint.rsplit_once(':') {
            Some((host, port)) => (Some(host), Some(port)),
            None => (Some(endpoint), None),
        }
    };
    if let Some(port) = port {
        let digits = port.bytes().all(|b| b.is_ascii_digit());
        if !digits || !port.parse::<u16>().is_ok_and(|p| p > 0) {
            return false;
        }
    }
    let Some(host) = host else {
        return true;
    };
    if host.parse::<std::net::Ipv4Addr>().is_ok() {
        return true;
    }
    host.len() <= 253
        && host.split('.').all(|label| {
            (1..=63).contains(&label.len())
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                && !label.starts_with('-')
                && !label.ends_with('-')
        })
        && host.bytes().any(|b| b.is_ascii_alphabetic())
}

/// Tailscale as last seen by the root helper (`/etc/dede-tv/tailscale.conf`).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Tailscale {
    pub signed_in: bool,
    pub waiting_for_login: bool,
    /// The TV's Tailscale IPv4 address while signed in.
    pub address: Option<String>,
}

pub fn read_tailscale(path: &Path) -> Tailscale {
    let mut tailscale = Tailscale::default();
    let Ok(text) = fs::read_to_string(path) else {
        return tailscale;
    };
    for line in text.lines() {
        match line.split_once('=') {
            Some(("TS_STATE", "Running")) => tailscale.signed_in = true,
            Some(("TS_STATE", "NeedsLogin" | "NeedsMachineAuth")) => {
                tailscale.waiting_for_login = true
            }
            Some(("TS_ADDRESS", address)) => {
                tailscale.address = address
                    .parse::<std::net::Ipv4Addr>()
                    .ok()
                    .filter(|ip| ip.octets()[0] == 100 && (64..128).contains(&ip.octets()[1]))
                    .map(|ip| ip.to_string());
            }
            _ => {}
        }
    }
    if !tailscale.signed_in {
        tailscale.address = None;
    }
    tailscale
}

/// QR code as inline SVG (black modules on white, with a quiet zone).
pub fn qr_svg(text: &str) -> Option<String> {
    let code = qrcode::QrCode::with_error_correction_level(text, qrcode::EcLevel::M).ok()?;
    let width = code.width();
    let quiet = 4;
    let size = width + 2 * quiet;
    let mut path = String::new();
    for (i, colour) in code.to_colors().iter().enumerate() {
        if *colour == qrcode::Color::Dark {
            let (x, y) = (i % width + quiet, i / width + quiet);
            path.push_str(&format!("M{x} {y}h1v1h-1z"));
        }
    }
    Some(format!(
        r##"<svg class="qr" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" shape-rendering="crispEdges" role="img"><rect width="{size}" height="{size}" fill="#fff"/><path d="{path}" fill="#000"/></svg>"##
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_endpoint_and_devices() {
        let dir = std::env::temp_dir().join(format!("dede-remote-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("remote.conf");
        assert_eq!(read(&file), State::default());
        fs::write(
            &file,
            "# test\nWG_ENDPOINT=haus.example.org:51820\n\
             WG_DEVICE=10.77.0.2 abc= Handy Mikail\n\
             WG_DEVICE=10.77.0.3 def= Tablet Ayşe\n\
             WG_DEVICE=10.0.0.9 ghi= Fremd\n\
             WG_DEVICE=10.77.0.4 broken\n",
        )
        .unwrap();
        let state = read(&file);
        assert_eq!(state.endpoint.as_deref(), Some("haus.example.org:51820"));
        let names: Vec<_> = state.devices.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["Handy Mikail", "Tablet Ayşe"]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn checks_names_addresses_and_endpoints() {
        assert!(valid_name("Handy Mikail") && valid_name("Tablet Ayşe"));
        for bad in ["", " Handy", "Handy ", "a<b", "x\ny", &"a".repeat(33)] {
            assert!(!valid_name(bad), "{bad:?}");
        }
        assert!(valid_address("10.77.0.2") && valid_address("10.77.0.254"));
        for bad in [
            "10.77.0.1",
            "10.77.0.255",
            "10.77.0.02",
            "10.77.1.2",
            "10.77.0.",
        ] {
            assert!(!valid_address(bad), "{bad}");
        }
        for good in [
            "haus.example.org",
            "haus.example.org:51820",
            "203.0.113.7",
            "203.0.113.7:4500",
            "[2001:db8::1]:51820",
            "[2001:db8::1]",
            "127.0.0.1:18082",
        ] {
            assert!(valid_endpoint(good), "{good}");
        }
        for bad in [
            "",
            "haus..org",
            "-haus.org",
            "haus.org:0",
            "haus.org:70000",
            "haus.org:",
            "http://haus.org",
            "haus org",
            "[2001:db8::1",
            "2001:db8::1",
            "1234",
        ] {
            assert!(!valid_endpoint(bad), "{bad}");
        }
    }

    #[test]
    fn reads_tailscale_state() {
        let dir = std::env::temp_dir().join(format!("dede-ts-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("tailscale.conf");
        assert_eq!(read_tailscale(&file), Tailscale::default());
        fs::write(&file, "TS_STATE=Running\nTS_ADDRESS=100.101.102.103\n").unwrap();
        let ts = read_tailscale(&file);
        assert!(ts.signed_in && ts.address.as_deref() == Some("100.101.102.103"));
        fs::write(&file, "TS_STATE=NeedsLogin\nTS_ADDRESS=100.101.102.103\n").unwrap();
        let ts = read_tailscale(&file);
        assert!(ts.waiting_for_login && !ts.signed_in && ts.address.is_none());
        fs::write(&file, "TS_STATE=Running\nTS_ADDRESS=8.8.8.8\n").unwrap();
        assert_eq!(read_tailscale(&file).address, None);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn qr_is_svg() {
        let svg = qr_svg("[Interface]\nPrivateKey = x\n").unwrap();
        assert!(svg.starts_with("<svg") && svg.contains("<path d=\"M"));
    }
}
