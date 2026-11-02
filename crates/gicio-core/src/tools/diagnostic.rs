use crate::domain::FilePath;

/// A warning or error a linter or compiler reported inside the repository.
pub struct ToolDiagnostic {
    pub path: FilePath,
    pub line: u32,
    pub end_line: u32,
    pub level: String,
    pub title: String,
    pub detail: String,
}
