use super::{Finding, FindingId, FindingSource, FindingStatus, Session, Severity};
use crate::ai::RawFinding;
use crate::diff::Side;
use crate::domain::FilePath;
use crate::tools::ToolDiagnostic;
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
        let n = raw.iter().filter(|r| self.add_raw_finding(FindingSource::Ai, r)).count();
        self.touch();
        n
    }

    /// Adds what the AI reported, unless it is about a file outside the change.
    pub(super) fn add_raw_finding(&mut self, source: FindingSource, r: &RawFinding) -> bool {
        let Ok(path) = FilePath::new(r.path.trim_start_matches("./")) else { return false };
        if !self.changed(&path) {
            return false;
        }
        let side = if r.side.as_deref() == Some("old") { Side::Old } else { Side::New };
        let (line, end_line) = (r.line.max(1), r.end_line.unwrap_or(r.line).max(r.line.max(1)));
        let code = self.excerpt_of(&path, side, line, end_line);
        let id = self.state.next_finding_id();
        self.state.findings.push(Finding {
            id,
            source,
            severity: severity(r.severity.as_deref()),
            path,
            side,
            line,
            end_line,
            title: r.title.clone(),
            detail: r.detail.clone(),
            status: FindingStatus::Open,
            code,
            outdated: false,
        });
        true
    }

    /// Replaces all findings of a tool (clippy, dotnet build); only those in changed files are kept.
    pub fn set_tool_findings(&mut self, source: FindingSource, diags: Vec<ToolDiagnostic>) -> usize {
        self.state.findings.retain(|f| f.source != source);
        let mut n = 0;
        let diags: Vec<ToolDiagnostic> = diags.into_iter().filter(|d| self.changed(&d.path)).collect();
        for d in diags {
            let code = self.excerpt_of(&d.path, Side::New, d.line, d.end_line);
            let id = self.state.next_finding_id();
            self.state.findings.push(Finding {
                id,
                source,
                severity: severity(Some(&d.level)),
                path: d.path,
                side: Side::New,
                line: d.line,
                end_line: d.end_line,
                title: d.title,
                detail: d.detail,
                status: FindingStatus::Open,
                code,
                outdated: false,
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
