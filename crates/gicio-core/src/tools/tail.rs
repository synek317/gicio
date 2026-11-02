/// The last 2000 characters of a tool's output, for error messages.
pub fn tail(bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes);
    s.trim().chars().rev().take(2000).collect::<Vec<_>>().into_iter().rev().collect()
}
