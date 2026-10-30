use super::{Finding, FindingId, FindingSource, FindingStatus, Session, Severity};
use crate::ai::RawFinding;
use crate::diff::Side;
use crate::domain::FilePath;
use crate::tools::ClippyDiagnostic;
use std::collections::HashSet;

fn severity(s: Option<&str>) -> Severity {
    match s.unwrap_or("warning") {
        "error" => Severity::Error,
        "info" | "note" | "help" => Severity::Info,
        _ => Severity::Warning,
    }
}

impl Session {
    fn changed(&self, path: &FilePath) -> bool {
        self.files.iter().any(|f| &f.path == path)
    }

    /// Replaces the open AI findings for `paths` with a new review's results.
    pub fn add_ai_findings(&mut self, raw: Vec<RawFinding>, paths: &HashSet<FilePath>) -> usize {
        self.state.findings.retain(|f| !(f.source == FindingSource::Ai && f.status == FindingStatus::Open && paths.contains(&f.path)));
        let mut n = 0;
        for r in raw {
            let Ok(path) = FilePath::new(r.path.trim_start_matches("./")) else { continue };
            if !self.changed(&path) {
                continue;
            }
            let side = if r.side.as_deref() == Some("old") { Side::Old } else { Side::New };
            let id = self.state.next_finding_id();
            self.state.findings.push(Finding {
                id,
                source: FindingSource::Ai,
                severity: severity(r.severity.as_deref()),
                path,
                side,
                line: r.line.max(1),
                end_line: r.end_line.unwrap_or(r.line).max(r.line.max(1)),
                title: r.title,
                detail: r.detail,
                status: crate::review::FindingStatus::Open,
            });
            n += 1;
        }
        self.touch();
        n
    }

    /// Replaces all clippy findings; only those in changed files are kept.
    pub fn set_clippy_findings(&mut self, diags: Vec<ClippyDiagnostic>) -> usize {
        self.state.findings.retain(|f| f.source != FindingSource::Clippy);
        let mut n = 0;
        let diags: Vec<ClippyDiagnostic> = diags.into_iter().filter(|d| self.changed(&d.path)).collect();
        for d in diags {
            let id = self.state.next_finding_id();
            self.state.findings.push(Finding {
                id,
                source: FindingSource::Clippy,
                severity: severity(Some(&d.level)),
                path: d.path,
                side: Side::New,
                line: d.line,
                end_line: d.end_line,
                title: d.title,
                detail: d.detail,
                status: FindingStatus::Open,
            });
            n += 1;
        }
        self.touch();
        n
    }

    pub fn set_finding_status(&mut self, ids: &[FindingId], status: FindingStatus) {
        for f in self.state.findings.iter_mut().filter(|f| ids.contains(&f.id)) {
            f.status = status;
        }
        self.touch();
    }

    pub fn findings_by_id(&self, ids: &[FindingId]) -> Vec<Finding> {
        self.state.findings.iter().filter(|f| ids.contains(&f.id)).cloned().collect()
    }
}
