use std::path::{Path, PathBuf};

pub fn to_uri(path: &Path) -> String {
    encode(&path.to_string_lossy(), cfg!(windows))
}

pub fn from_uri(uri: &str) -> Option<PathBuf> {
    decode(uri, cfg!(windows)).map(PathBuf::from)
}

/// `C:\a b\c.rs` on Windows becomes `file:///C:/a%20b/c.rs`.
fn encode(path: &str, windows: bool) -> String {
    let path = if windows { path.replace('\\', "/") } else { path.to_string() };
    let mut out = String::from(if path.starts_with('/') { "file://" } else { "file:///" });
    for b in path.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b':' if windows => out.push(':'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// `file:///c%3A/x.rs` on Windows becomes `C:/x.rs` (drive letter upper-cased, as Windows paths
/// compare it case-sensitively).
fn decode(uri: &str, windows: bool) -> Option<String> {
    let raw = uri.strip_prefix("file://")?.as_bytes();
    let mut bytes = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'%' && i + 2 < raw.len() {
            let hex = std::str::from_utf8(&raw[i + 1..i + 3]).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            bytes.push(raw[i]);
            i += 1;
        }
    }
    let path = String::from_utf8(bytes).ok()?;
    match path.as_bytes() {
        [b'/', drive, b':', ..] if windows && drive.is_ascii_alphabetic() => Some(format!("{}{}", drive.to_ascii_uppercase() as char, &path[2..])),
        _ => Some(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_paths_round_trip() {
        let uri = encode("/home/me/a b/ü.rs", false);
        assert_eq!(uri, "file:///home/me/a%20b/%C3%BC.rs");
        assert_eq!(decode(&uri, false).unwrap(), "/home/me/a b/ü.rs");
    }

    #[test]
    fn windows_paths_get_a_drive_uri() {
        assert_eq!(encode(r"C:\Users\me\a b.rs", true), "file:///C:/Users/me/a%20b.rs");
        assert_eq!(decode("file:///c%3A/Users/me/a%20b.rs", true).unwrap(), "C:/Users/me/a b.rs");
        assert_eq!(decode("file:///C:/x.rs", true).unwrap(), "C:/x.rs");
    }
}
