use crate::domain::{FilePath, RepoRoot};
use crate::project::DotnetProject;
use serde_json::{json, Value};
use std::ffi::OsString;

/// The language servers gicio can run, one per repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerKind {
    RustAnalyzer,
    CSharpLs,
}

impl ServerKind {
    /// The server for the repository's project: Cargo first, then .NET.
    pub fn detect(root: &RepoRoot) -> Option<Self> {
        if root.path().join("Cargo.toml").exists() {
            Some(Self::RustAnalyzer)
        } else if DotnetProject::find(root).is_some() {
            Some(Self::CSharpLs)
        } else {
            None
        }
    }

    /// The program name, also how the UI calls the server.
    pub fn name(self) -> &'static str {
        match self {
            Self::RustAnalyzer => "rust-analyzer",
            Self::CSharpLs => "csharp-ls",
        }
    }

    /// The program that installs the server.
    pub fn installer(self) -> &'static str {
        match self {
            Self::RustAnalyzer => "rustup",
            Self::CSharpLs => "dotnet",
        }
    }

    /// Arguments for the installer; `dotnet tool update` also installs a tool that is missing.
    pub fn install_args(self) -> &'static [&'static str] {
        match self {
            Self::RustAnalyzer => &["component", "add", "rust-analyzer"],
            Self::CSharpLs => &["tool", "update", "--global", "csharp-ls"],
        }
    }

    /// The whole install command, to show.
    pub fn install_command(self) -> String {
        format!("{} {}", self.installer(), self.install_args().join(" "))
    }

    /// What to install first when the installer itself is missing.
    pub fn prerequisite(self) -> &'static str {
        match self {
            Self::RustAnalyzer => "rustup (https://rustup.rs)",
            Self::CSharpLs => "the .NET SDK (https://dot.net)",
        }
    }

    /// File extensions the server understands.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::RustAnalyzer => &["rs"],
            Self::CSharpLs => &["cs"],
        }
    }

    pub fn handles(self, path: &FilePath) -> bool {
        path.extension().is_some_and(|e| self.extensions().contains(&e))
    }

    pub fn language_id(self) -> &'static str {
        match self {
            Self::RustAnalyzer => "rust",
            Self::CSharpLs => "csharp",
        }
    }

    /// Command-line arguments: csharp-ls is pointed at the solution, which may sit in a subdirectory.
    pub fn args(self, root: &RepoRoot) -> Vec<OsString> {
        match self {
            Self::RustAnalyzer => Vec::new(),
            Self::CSharpLs => DotnetProject::find(root).filter(DotnetProject::is_solution).map(|p| vec!["--solution".into(), p.path().into()]).unwrap_or_default(),
        }
    }

    pub fn initialization_options(self) -> Value {
        match self {
            Self::RustAnalyzer => json!({"checkOnSave": false, "cachePriming": {"enable": true}}),
            Self::CSharpLs => Value::Null,
        }
    }

    /// Whether the server says when the whole workspace is loaded (rust-analyzer's
    /// `experimental/serverStatus`). Others count as loaded once initialized.
    pub fn reports_quiescence(self) -> bool {
        self == Self::RustAnalyzer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_only_its_own_files() {
        let cs = FilePath::new("src/App/Program.cs").unwrap();
        let rs = FilePath::new("src/main.rs").unwrap();
        assert!(ServerKind::CSharpLs.handles(&cs));
        assert!(!ServerKind::CSharpLs.handles(&rs));
        assert!(ServerKind::RustAnalyzer.handles(&rs));
        assert!(!ServerKind::RustAnalyzer.handles(&FilePath::new("App.csproj").unwrap()));
    }
}
