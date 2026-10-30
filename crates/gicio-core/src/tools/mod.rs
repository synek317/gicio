//! Project tools run by the reviewer: formatter and linter.

mod cargo;

pub use cargo::{cargo_clippy, cargo_clippy_fix, cargo_fmt, ClippyDiagnostic};
