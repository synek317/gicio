/// One `file(line,col): warning CS0168: message [project]` line of MSBuild output.
#[derive(Debug, PartialEq, Eq)]
pub struct MsbuildMessage {
    /// As MSBuild printed it: absolute, or relative to the project's directory.
    pub file: String,
    pub line: u32,
    pub end_line: u32,
    pub level: String,
    pub code: String,
    pub text: String,
}

impl MsbuildMessage {
    /// Reads a warning or error with a source location; `None` for any other line.
    pub fn parse(line: &str) -> Option<Self> {
        let (location, rest) = line.trim().split_once("): ")?;
        let (file, position) = location.rsplit_once('(')?;
        let numbers: Vec<u32> = position.split(',').map(|n| n.trim().parse().ok()).collect::<Option<_>>()?;
        let (start, end) = match numbers[..] {
            [l, _] => (l, l),
            [l, _, e, _] => (l, e.max(l)),
            _ => return None,
        };
        let (level, rest) = rest.split_once(' ')?;
        if !matches!(level, "warning" | "error") {
            return None;
        }
        let (code, text) = rest.split_once(": ")?;
        // The project MSBuild appends in brackets says nothing about the problem.
        let text = match text.rsplit_once(" [") {
            Some((t, project)) if project.ends_with(']') => t,
            _ => text,
        };
        Some(Self { file: file.trim().to_string(), line: start, end_line: end, level: level.to_string(), code: code.to_string(), text: text.to_string() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_compiler_warning() {
        let m = MsbuildMessage::parse("/repo/src/App/Program.cs(12,17): warning CS0168: The variable 'x' is declared but never used [/repo/src/App/App.csproj]").unwrap();
        assert_eq!(
            m,
            MsbuildMessage { file: "/repo/src/App/Program.cs".into(), line: 12, end_line: 12, level: "warning".into(), code: "CS0168".into(), text: "The variable 'x' is declared but never used".into() }
        );
    }

    #[test]
    fn reads_ranges_and_windows_paths() {
        let m = MsbuildMessage::parse(r"  C:\repo\A (1)\B.cs(3,1,5,2): error CA1822: Member 'Run' does not access instance data [C:\repo\A.csproj]").unwrap();
        assert_eq!((m.file.as_str(), m.line, m.end_line, m.level.as_str(), m.code.as_str()), (r"C:\repo\A (1)\B.cs", 3, 5, "error", "CA1822"));
        assert_eq!(m.text, "Member 'Run' does not access instance data");
    }

    #[test]
    fn skips_lines_without_a_location() {
        assert_eq!(MsbuildMessage::parse("CSC : error CS2001: Source file 'x.cs' could not be found."), None);
        assert_eq!(MsbuildMessage::parse("Build succeeded."), None);
        assert_eq!(MsbuildMessage::parse("  App -> /repo/bin/Debug/net9.0/App.dll"), None);
    }
}
