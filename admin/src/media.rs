//! Films in a channel folder: listing, deleting and streaming uploads.
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// Same list as dede-virtual-sync (dede-tv-os).
pub const EXTENSIONS: &[&str] = &["mp4", "mkv", "mov", "m4v", "webm", "avi", "ts"];
pub const MAX_UPLOAD_BYTES: u64 = 32 * 1024 * 1024 * 1024;
const MAX_PART_HEADER: usize = 4096;
const MAX_FIELD: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Film {
    pub name: String,
    pub bytes: u64,
}

/// A plain file name with a video extension; no paths, no hidden files.
pub fn valid_film_name(name: &str) -> bool {
    let ext_ok = name.rsplit_once('.').is_some_and(|(stem, ext)| {
        !stem.is_empty() && EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
    });
    ext_ok
        && name.len() <= 200
        && !name.starts_with('.')
        && !name.contains(['/', '\\'])
        && !name.chars().any(|c| (c as u32) < 0x20 || c == '\u{7f}')
}

/// Folder of a channel; refuses anything but a plain, existing directory.
pub fn channel_dir(root: &Path, folder: &str) -> io::Result<PathBuf> {
    if folder.is_empty() || folder.contains('/') || folder.starts_with('.') {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "folder"));
    }
    let dir = root.join(folder);
    let meta = fs::symlink_metadata(&dir)?;
    if !meta.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a directory",
        ));
    }
    Ok(dir)
}

pub fn list(dir: &Path) -> io::Result<Vec<Film>> {
    let mut films = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if meta.is_file() && valid_film_name(&name) {
            films.push(Film {
                name,
                bytes: meta.len(),
            });
        }
    }
    films.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(films)
}

pub fn delete(dir: &Path, name: &str) -> io::Result<()> {
    if !valid_film_name(name) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "name"));
    }
    let path = dir.join(name);
    if !fs::symlink_metadata(&path)?.is_file() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "not a file"));
    }
    fs::remove_file(path)
}

#[derive(Debug, PartialEq, Eq)]
pub enum UploadError {
    BadRequest,
    Forbidden,
    BadName,
    Exists,
    TooLarge,
    Io,
}

/// Boundary from `multipart/form-data; boundary=...`.
pub fn boundary(content_type: &str) -> Option<String> {
    let (kind, rest) = content_type.split_once(';')?;
    if !kind.trim().eq_ignore_ascii_case("multipart/form-data") {
        return None;
    }
    rest.split(';').find_map(|p| {
        let (k, v) = p.trim().split_once('=')?;
        let v = v.trim().trim_matches('"');
        (k.eq_ignore_ascii_case("boundary") && !v.is_empty() && v.len() <= 70)
            .then(|| v.to_string())
    })
}

struct Stream<R> {
    inner: R,
    buf: Vec<u8>,
    read_total: u64,
}

impl<R: Read> Stream<R> {
    fn fill(&mut self) -> io::Result<bool> {
        let mut chunk = [0u8; 64 * 1024];
        let n = self.inner.read(&mut chunk)?;
        self.read_total += n as u64;
        self.buf.extend_from_slice(&chunk[..n]);
        Ok(n > 0)
    }

    /// Bytes up to `needle` (consumed), at most `limit`.
    fn until(&mut self, needle: &[u8], limit: usize) -> Result<Vec<u8>, UploadError> {
        loop {
            if let Some(i) = find(&self.buf, needle) {
                let out = self.buf[..i].to_vec();
                self.buf.drain(..i + needle.len());
                return Ok(out);
            }
            if self.buf.len() > limit + needle.len() {
                return Err(UploadError::BadRequest);
            }
            if !self.fill().map_err(|_| UploadError::Io)? {
                return Err(UploadError::BadRequest);
            }
        }
    }

    /// Copy bytes up to `needle` into `out`, without holding them in memory.
    fn copy_until(
        &mut self,
        needle: &[u8],
        out: &mut impl Write,
        max: u64,
    ) -> Result<u64, UploadError> {
        let mut written = 0u64;
        loop {
            if let Some(i) = find(&self.buf, needle) {
                written += i as u64;
                if written > max {
                    return Err(UploadError::TooLarge);
                }
                out.write_all(&self.buf[..i]).map_err(|_| UploadError::Io)?;
                self.buf.drain(..i + needle.len());
                return Ok(written);
            }
            let keep = needle.len();
            if self.buf.len() > keep {
                let n = self.buf.len() - keep;
                written += n as u64;
                if written > max {
                    return Err(UploadError::TooLarge);
                }
                out.write_all(&self.buf[..n]).map_err(|_| UploadError::Io)?;
                self.buf.drain(..n);
            }
            if !self.fill().map_err(|_| UploadError::Io)? {
                return Err(UploadError::BadRequest);
            }
        }
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn disposition(headers: &str) -> Option<(String, Option<String>)> {
    let line = headers
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("content-disposition:"))?;
    let mut name = None;
    let mut filename = None;
    for part in line.split(';').skip(1) {
        let (k, v) = part.trim().split_once('=')?;
        let v = v.trim().trim_matches('"').to_string();
        match k {
            "name" => name = Some(v),
            "filename" => filename = Some(v),
            _ => {}
        }
    }
    Some((name?, filename))
}

/// Stream a multipart upload into `dir`. The `csrf` field must come before
/// any file and match `csrf`. Returns the stored file names.
pub fn receive(
    body: impl Read,
    boundary: &str,
    csrf: &str,
    dir: &Path,
) -> Result<Vec<String>, UploadError> {
    let mut stream = Stream {
        inner: body,
        buf: Vec::new(),
        read_total: 0,
    };
    let first = format!("--{boundary}");
    let delimiter = format!("\r\n--{boundary}");
    stream.until(first.as_bytes(), 1024)?;
    let mut csrf_ok = false;
    let mut stored = Vec::new();
    loop {
        let after = stream.until(b"\r\n", 2)?;
        if after == b"--" {
            break; // closing delimiter: "--boundary--\r\n"
        }
        if !after.is_empty() {
            return Err(UploadError::BadRequest);
        }
        let headers = stream.until(b"\r\n\r\n", MAX_PART_HEADER)?;
        let headers = String::from_utf8(headers).map_err(|_| UploadError::BadRequest)?;
        let (name, filename) = disposition(&headers).ok_or(UploadError::BadRequest)?;
        match (name.as_str(), filename) {
            ("csrf", None) => {
                let value = stream.until(delimiter.as_bytes(), MAX_FIELD)?;
                csrf_ok = crate::auth::constant_time_eq(&value, csrf.as_bytes());
                if !csrf_ok {
                    return Err(UploadError::Forbidden);
                }
            }
            ("film", Some(filename)) => {
                if !csrf_ok {
                    return Err(UploadError::Forbidden);
                }
                let base = filename
                    .rsplit(['/', '\\'])
                    .next()
                    .unwrap_or("")
                    .to_string();
                if base.is_empty() && stored.is_empty() {
                    // No file chosen: browsers still send an empty part.
                    stream.copy_until(delimiter.as_bytes(), &mut io::sink(), 0)?;
                    continue;
                }
                if !valid_film_name(&base) {
                    return Err(UploadError::BadName);
                }
                let target = dir.join(&base);
                if fs::symlink_metadata(&target).is_ok() {
                    return Err(UploadError::Exists);
                }
                let temp = dir.join(format!(".upload-{}.part", crate::auth::random_token()));
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o640)
                    .open(&temp)
                    .map_err(|_| UploadError::Io)?;
                let copied = stream
                    .copy_until(delimiter.as_bytes(), &mut file, MAX_UPLOAD_BYTES)
                    .and_then(|n| file.sync_all().map(|_| n).map_err(|_| UploadError::Io));
                let result = copied.and_then(|_| {
                    // No overwrite: a file with that name appeared meanwhile.
                    fs::hard_link(&temp, &target).map_err(|e| {
                        if e.kind() == io::ErrorKind::AlreadyExists {
                            UploadError::Exists
                        } else {
                            UploadError::Io
                        }
                    })
                });
                let _ = fs::remove_file(&temp);
                result?;
                stored.push(base);
            }
            _ => {
                stream.until(delimiter.as_bytes(), MAX_FIELD)?;
            }
        }
    }
    if !csrf_ok {
        return Err(UploadError::Forbidden);
    }
    Ok(stored)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dede-media-{}", crate::auth::random_token()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn body(parts: &[(&str, Option<&str>, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        for (name, filename, value) in parts {
            out.extend_from_slice(b"--XyZ\r\n");
            match filename {
                Some(f) => out.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"{f}\"\r\nContent-Type: video/mp4\r\n\r\n").as_bytes(),
                ),
                None => out.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
                ),
            }
            out.extend_from_slice(value);
            out.extend_from_slice(b"\r\n");
        }
        out.extend_from_slice(b"--XyZ--\r\n");
        out
    }

    /// Reader returning tiny chunks, so delimiters straddle reads.
    struct Trickle(Vec<u8>, usize);
    impl Read for Trickle {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let n = buf.len().min(7).min(self.0.len() - self.1);
            buf[..n].copy_from_slice(&self.0[self.1..self.1 + n]);
            self.1 += n;
            Ok(n)
        }
    }

    #[test]
    fn film_names() {
        assert!(valid_film_name("Film 1.mp4"));
        assert!(valid_film_name("Çizgi.MKV"));
        for bad in [
            ".mp4",
            ".hidden.mp4",
            "a/b.mp4",
            "a\\b.mp4",
            "film.txt",
            "film",
            "a\nb.mp4",
        ] {
            assert!(!valid_film_name(bad), "{bad:?}");
        }
    }

    #[test]
    fn boundary_parsing() {
        assert_eq!(
            boundary("multipart/form-data; boundary=----abc").as_deref(),
            Some("----abc")
        );
        assert_eq!(
            boundary("multipart/form-data; boundary=\"q\"").as_deref(),
            Some("q")
        );
        assert_eq!(boundary("application/x-www-form-urlencoded"), None);
    }

    #[test]
    fn upload_streams_files_with_binary_content() {
        let dir = tempdir();
        // CR LF and partial delimiters (split across reads) must survive intact.
        let film: Vec<u8> = b"\r\n--XyQ\x00\xff\r\n--Xy"
            .iter()
            .copied()
            .cycle()
            .take(200_000)
            .collect();
        let data = body(&[
            ("csrf", None, b"tok"),
            ("film", Some("a.mp4"), &film),
            ("film", Some("C:\\x\\b.mkv"), b"bb"),
        ]);
        let stored = receive(Trickle(data, 0), "XyZ", "tok", &dir).unwrap();
        assert_eq!(stored, vec!["a.mp4", "b.mkv"]);
        assert_eq!(fs::read(dir.join("a.mp4")).unwrap(), film);
        assert_eq!(
            list(&dir)
                .unwrap()
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>(),
            ["a.mp4", "b.mkv"]
        );
        assert!(fs::read_dir(&dir).unwrap().all(|e| !e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".upload-")));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn upload_refuses_missing_csrf_bad_names_and_overwrites() {
        let dir = tempdir();
        let no_csrf = body(&[("film", Some("a.mp4"), b"x")]);
        assert_eq!(
            receive(&no_csrf[..], "XyZ", "tok", &dir),
            Err(UploadError::Forbidden)
        );
        let wrong = body(&[("csrf", None, b"nope"), ("film", Some("a.mp4"), b"x")]);
        assert_eq!(
            receive(&wrong[..], "XyZ", "tok", &dir),
            Err(UploadError::Forbidden)
        );
        let bad = body(&[
            ("csrf", None, b"tok"),
            ("film", Some("../../evil.mp4.sh"), b"x"),
        ]);
        assert_eq!(
            receive(&bad[..], "XyZ", "tok", &dir),
            Err(UploadError::BadName)
        );
        fs::write(dir.join("a.mp4"), b"old").unwrap();
        let dup = body(&[("csrf", None, b"tok"), ("film", Some("a.mp4"), b"new")]);
        assert_eq!(
            receive(&dup[..], "XyZ", "tok", &dir),
            Err(UploadError::Exists)
        );
        assert_eq!(fs::read(dir.join("a.mp4")).unwrap(), b"old");
        let truncated = &body(&[("csrf", None, b"tok"), ("film", Some("c.mp4"), b"data")])[..60];
        assert!(receive(truncated, "XyZ", "tok", &dir).is_err());
        assert!(!dir.join("c.mp4").exists());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn empty_file_field_is_ignored() {
        let dir = tempdir();
        let data = body(&[("csrf", None, b"tok"), ("film", Some(""), b"")]);
        assert_eq!(receive(&data[..], "XyZ", "tok", &dir), Ok(vec![]));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn delete_only_plain_films() {
        let dir = tempdir();
        fs::write(dir.join("a.mp4"), b"x").unwrap();
        assert!(delete(&dir, "../a.mp4").is_err());
        delete(&dir, "a.mp4").unwrap();
        assert!(!dir.join("a.mp4").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
