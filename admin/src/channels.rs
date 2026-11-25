//! Own (virtual) channels: configuration file, validation, folder names.
//!
//! Mirrors dede-virtual-sync's rules (dede-tv-os); the root apply helper
//! validates again before anything reaches the TV.
use std::fs;
use std::path::Path;

pub const MAX_CHANNELS: usize = 50;
pub const MAX_CONFIG: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    pub name: String,
    pub number: u32,
    pub folder: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Problem {
    Name,
    Number,
    NumberTaken,
    TooMany,
}

fn invisible(c: char) -> bool {
    let n = c as u32;
    n < 0x20
        || n == 0x7f
        || (0x202a..0x202f).contains(&n)
        || (0x2066..0x206a).contains(&n)
        || matches!(
            n,
            0x200e | 0x200f | 0x061c | 0x00ad | 0x200b | 0x200c | 0x200d | 0x2060 | 0xfeff
        )
}

/// Same rules as dede-virtual-sync's valid_name, plus INI-safe brackets
/// and no surrounding blanks.
pub fn valid_name(name: &str) -> bool {
    !name.trim().is_empty()
        && name.trim() == name
        && name.len() <= 92
        && !name.chars().any(invisible)
        && !name.starts_with("pvr:")
        && !name.starts_with("virtual:")
        && !name.contains([',', '"', '[', ']'])
}

pub fn parse_number(raw: &str) -> Option<u32> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 4 || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    raw.parse().ok().filter(|n| (1..=9999).contains(n))
}

/// One folder name below the media root.
pub fn valid_folder(folder: &str) -> bool {
    !folder.is_empty()
        && folder.len() <= 64
        && !folder.starts_with('.')
        && folder
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// ASCII folder name derived from a channel name ("Dede Çizgi" → "dede-cizgi").
pub fn folder_slug(name: &str) -> String {
    let mut slug = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        let mapped = match c {
            'a'..='z' | '0'..='9' => c.to_string(),
            'ç' => "c".into(),
            'ğ' => "g".into(),
            'ı' | 'î' => "i".into(),
            'ö' => "oe".into(),
            'ş' => "s".into(),
            'ü' => "ue".into(),
            'ä' => "ae".into(),
            'ß' => "ss".into(),
            'â' | 'à' | 'á' => "a".into(),
            'é' | 'è' | 'ê' => "e".into(),
            _ => "-".into(),
        };
        slug.push_str(&mapped);
    }
    let mut out = String::new();
    for part in slug.split('-').filter(|p| !p.is_empty()) {
        if !out.is_empty() {
            out.push('-');
        }
        out.push_str(part);
    }
    out.truncate(48);
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() {
        "sender".into()
    } else {
        out
    }
}

/// A folder name not yet used below `root` nor by another channel.
pub fn unique_folder(name: &str, root: &Path, channels: &[Channel]) -> String {
    let base = folder_slug(name);
    let taken = |f: &str| root.join(f).exists() || channels.iter().any(|c| c.folder == f);
    if !taken(&base) {
        return base;
    }
    (2..1000)
        .map(|i| format!("{base}-{i}"))
        .find(|f| !taken(f))
        .unwrap_or_else(|| format!("{base}-{}", &crate::auth::random_token()[..8]))
}

pub fn suggest_number(channels: &[Channel]) -> u32 {
    let max = channels.iter().map(|c| c.number).max().unwrap_or(4);
    if max < 9999 {
        return max + 1;
    }
    (1..=9999)
        .find(|n| !channels.iter().any(|c| c.number == *n))
        .unwrap_or(1)
}

/// Read the configuration (INI subset written by `render`). Missing → empty.
pub fn read(path: &Path) -> Result<Vec<Channel>, String> {
    let text = match fs::read(path) {
        Ok(bytes) if bytes.len() > MAX_CONFIG => return Err("config too large".into()),
        Ok(bytes) => String::from_utf8(bytes).map_err(|_| "config is not UTF-8".to_string())?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("cannot read config: {e}")),
    };
    parse(&text)
}

pub fn parse(text: &str) -> Result<Vec<Channel>, String> {
    let mut channels: Vec<Channel> = Vec::new();
    let mut current: Option<(String, Option<u32>, Option<String>)> = None;
    let finish = |entry: Option<(String, Option<u32>, Option<String>)>,
                  channels: &mut Vec<Channel>|
     -> Result<(), String> {
        if let Some((name, number, folder)) = entry {
            let number = number.ok_or(format!("{name}: number missing"))?;
            let folder = folder.ok_or(format!("{name}: folder missing"))?;
            channels.push(Channel {
                name,
                number,
                folder,
            });
        }
        Ok(())
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(header) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            finish(current.take(), &mut channels)?;
            current = Some((header.to_string(), None, None));
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .or_else(|| line.split_once(':'))
            .ok_or("unreadable line")?;
        let entry = current.as_mut().ok_or("setting outside a channel")?;
        match key.trim() {
            "number" => entry.1 = Some(parse_number(value).ok_or("invalid number")?),
            "folder" => entry.2 = Some(value.trim().to_string()),
            _ => return Err("unknown setting".into()),
        }
    }
    finish(current, &mut channels)?;
    Ok(channels)
}

pub fn render(channels: &[Channel]) -> String {
    let mut sorted: Vec<&Channel> = channels.iter().collect();
    sorted.sort_by_key(|c| c.number);
    let mut out = String::from("# Eigene Sender, verwaltet von der Dede TV Verwaltung.\n");
    for c in sorted {
        out.push_str(&format!(
            "\n[{}]\nnumber = {}\nfolder = {}\n",
            c.name, c.number, c.folder
        ));
    }
    out
}

/// Validate a new channel against the existing ones.
pub fn check_new(channels: &[Channel], name: &str, number: Option<u32>) -> Result<u32, Problem> {
    if channels.len() >= MAX_CHANNELS {
        return Err(Problem::TooMany);
    }
    if !valid_name(name) || channels.iter().any(|c| c.name == name) {
        return Err(Problem::Name);
    }
    let number = number.ok_or(Problem::Number)?;
    if channels.iter().any(|c| c.number == number) {
        return Err(Problem::NumberTaken);
    }
    Ok(number)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(name: &str, number: u32, folder: &str) -> Channel {
        Channel {
            name: name.into(),
            number,
            folder: folder.into(),
        }
    }

    #[test]
    fn names_follow_the_device_rules() {
        assert!(valid_name("Dede Film"));
        assert!(valid_name("Çizgi Film Kanalı"));
        for bad in [
            "",
            "  ",
            " Film",
            "pvr:1",
            "virtual:x",
            "A,B",
            "A\"B",
            "A[1]",
            "A\u{202e}B",
            "A\nB",
            &"x".repeat(93),
        ] {
            assert!(!valid_name(bad), "{bad:?}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(parse_number("7"), Some(7));
        assert_eq!(parse_number(" 05 "), Some(5));
        for bad in ["", "0", "10000", "-1", "+3", "1.5", "abc"] {
            assert_eq!(parse_number(bad), None, "{bad}");
        }
    }

    #[test]
    fn folder_slugs_are_ascii_and_safe() {
        assert_eq!(folder_slug("Dede Film"), "dede-film");
        assert_eq!(folder_slug("Çocuk Şarkıları"), "cocuk-sarkilari");
        assert_eq!(folder_slug("Grüße aus Köln!"), "gruesse-aus-koeln");
        assert_eq!(folder_slug("../../etc"), "etc");
        assert_eq!(folder_slug("!!!"), "sender");
        for name in [
            "Dede Film",
            "Çocuk Şarkıları",
            "../../etc",
            "!!!",
            &"ü".repeat(90),
        ] {
            assert!(valid_folder(&folder_slug(name)), "{name}");
        }
    }

    #[test]
    fn unique_folder_avoids_existing_ones() {
        let dir = std::env::temp_dir().join(format!("dede-ch-{}", crate::auth::random_token()));
        fs::create_dir_all(dir.join("film")).unwrap();
        let channels = vec![ch("X", 1, "film-2")];
        assert_eq!(unique_folder("Film", &dir, &channels), "film-3");
        assert_eq!(unique_folder("Neu", &dir, &channels), "neu");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn render_parse_roundtrip_matches_the_device_format() {
        let channels = vec![ch("Dede Film", 5, "film"), ch("Çizgi = Spaß", 7, "cizgi")];
        let text = render(&channels);
        assert!(text.contains("[Dede Film]\nnumber = 5\nfolder = film\n"));
        assert_eq!(parse(&text).unwrap(), channels);
        // Hand-written file in the documented format.
        let manual = "[Dede Film]\nnumber = 5\nfolder = film\n\n[Dede Serien]\nnumber = 7\nfolder = serien\n";
        assert_eq!(parse(manual).unwrap().len(), 2);
        assert!(parse("[A]\nnumber = 5\n").is_err());
        assert!(parse("[A]\nnumber = 5\nfolder = a\nextra = 1\n").is_err());
        assert!(parse("number = 5\n").is_err());
    }

    #[test]
    fn new_channel_checks() {
        let channels = vec![ch("Dede Film", 5, "film")];
        assert_eq!(check_new(&channels, "Neu", Some(6)), Ok(6));
        assert_eq!(
            check_new(&channels, "Dede Film", Some(6)),
            Err(Problem::Name)
        );
        assert_eq!(
            check_new(&channels, "Neu", Some(5)),
            Err(Problem::NumberTaken)
        );
        assert_eq!(check_new(&channels, "Neu", None), Err(Problem::Number));
        assert_eq!(suggest_number(&channels), 6);
        assert_eq!(suggest_number(&[]), 5);
    }
}
