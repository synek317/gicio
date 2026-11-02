//! Project tools run by the reviewer: formatters, linters and compilers.

mod cargo;
mod diagnostic;
mod dotnet;
mod msbuild;
mod tail;

pub use cargo::{cargo_clippy, cargo_clippy_fix, cargo_fmt};
pub use diagnostic::ToolDiagnostic;
pub use dotnet::{dotnet_build, dotnet_format};
pub(crate) use tail::tail;
