/// `content` with CRLF line endings when `like` uses them and `content` has bare LFs only, else as is.
pub fn with_line_endings_of(content: &[u8], like: &[u8]) -> Vec<u8> {
    let crlf = |b: &[u8]| b.windows(2).any(|w| w == b"\r\n");
    if !crlf(like) || crlf(content) {
        return content.to_vec();
    }
    let mut out = Vec::with_capacity(content.len() + content.len() / 32);
    for &b in content {
        if b == b'\n' {
            out.push(b'\r');
        }
        out.push(b);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_a_crlf_file() {
        assert_eq!(with_line_endings_of(b"a\nb\n", b"x\r\ny\r\n"), b"a\r\nb\r\n");
    }

    #[test]
    fn leaves_lf_and_existing_crlf_alone() {
        assert_eq!(with_line_endings_of(b"a\nb\n", b"x\ny\n"), b"a\nb\n");
        assert_eq!(with_line_endings_of(b"a\r\nb\n", b"x\r\n"), b"a\r\nb\n");
        assert_eq!(with_line_endings_of(b"a\nb", b""), b"a\nb");
    }
}
