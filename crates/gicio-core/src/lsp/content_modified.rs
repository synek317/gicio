use std::fmt;

/// LSP error code a server answers with when the document changed while it was computing.
pub const CODE: i64 = -32801;

/// The request was cancelled because the content changed (e.g. the workspace finished
/// loading); asking again gives an up-to-date answer.
#[derive(Debug)]
pub struct ContentModified;

impl fmt::Display for ContentModified {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LSP error: content modified")
    }
}

impl std::error::Error for ContentModified {}
