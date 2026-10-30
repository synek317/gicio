use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! string_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(n: u64) -> Self {
                Self(format!("{}{}", $prefix, n))
            }
            pub fn from_raw(raw: impl Into<String>) -> Self {
                Self(raw.into())
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_id!(ChunkId, "c");
string_id!(ThreadId, "t");

/// Content-derived id of a hunk: stays the same as long as the hunk's text doesn't change.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HunkId(String);

impl HunkId {
    pub fn from_content(path: &str, body: &str, occurrence: usize) -> Self {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in path.bytes().chain([0u8]).chain(body.bytes()) {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        Self(format!("h{:012x}{}", h & 0xffff_ffff_ffff, if occurrence > 0 { format!("-{occurrence}") } else { String::new() }))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
