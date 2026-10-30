use super::uri::{from_uri, to_uri};
use super::{LspClient, LspStatus};
use crate::ai::{login_path, resolve_program};
use crate::domain::{FilePath, RepoRoot};
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use tokio::sync::Mutex;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    /// Absolute path (may be outside the repo, e.g. std or a dependency).
    pub abs_path: String,
    /// Repo-relative path when the target is inside the repository.
    pub repo_path: Option<FilePath>,
    pub line: u32,
    pub character: u32,
    /// The target line's text, for listing references.
    pub preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub name: String,
    /// LSP SymbolKind (12 = function, 6 = method, 5 = class, 23 = struct, …).
    pub kind: u32,
    pub start: u32,
    pub end: u32,
}

pub struct RustAnalyzer {
    client: LspClient,
    root: RepoRoot,
    open_docs: Mutex<HashMap<String, (i32, String)>>,
}

impl RustAnalyzer {
    pub async fn start(root: RepoRoot) -> Result<Self> {
        let program = resolve_program("rust-analyzer").context("rust-analyzer not found (rustup component add rust-analyzer)")?;
        let client = LspClient::spawn(&program, login_path(), root.path())?;
        let params = json!({
            "processId": std::process::id(),
            "rootUri": to_uri(root.path()),
            "workspaceFolders": [{"uri": to_uri(root.path()), "name": "repo"}],
            "capabilities": {
                "window": {"workDoneProgress": true},
                "textDocument": {
                    "hover": {"contentFormat": ["markdown", "plaintext"]},
                    "definition": {"linkSupport": true},
                    "documentSymbol": {"hierarchicalDocumentSymbolSupport": true},
                    "synchronization": {"didSave": false}
                },
                "general": {"positionEncodings": ["utf-16"]}
            },
            "initializationOptions": {"checkOnSave": false, "cachePriming": {"enable": true}}
        });
        client.request("initialize", params, Duration::from_secs(60)).await?;
        client.notify("initialized", json!({})).await?;
        client.set_status(LspStatus::Indexing { detail: "loading workspace".into() });
        Ok(Self { client, root, open_docs: Mutex::new(HashMap::new()) })
    }

    pub fn status(&self) -> LspStatus {
        self.client.status()
    }

    /// Makes sure the server sees the file as it is on disk right now.
    async fn sync(&self, path: &FilePath) -> Result<String> {
        let abs = self.root.join(path);
        let uri = to_uri(&abs);
        let text = std::fs::read_to_string(&abs).with_context(|| format!("cannot read {path}"))?;
        let mut docs = self.open_docs.lock().await;
        match docs.get_mut(&uri) {
            None => {
                self.client.notify("textDocument/didOpen", json!({"textDocument": {"uri": uri, "languageId": "rust", "version": 1, "text": text}})).await?;
                docs.insert(uri.clone(), (1, text));
            }
            Some((version, old)) if *old != text => {
                *version += 1;
                self.client.notify("textDocument/didChange", json!({"textDocument": {"uri": uri, "version": *version}, "contentChanges": [{"text": text}]})).await?;
                *old = text;
            }
            Some(_) => {}
        }
        Ok(uri)
    }

    fn position(uri: &str, line: u32, character: u32) -> Value {
        json!({"textDocument": {"uri": uri}, "position": {"line": line, "character": character}})
    }

    pub async fn hover(&self, path: &FilePath, line: u32, character: u32) -> Result<Option<String>> {
        let uri = self.sync(path).await?;
        let res = self.client.request("textDocument/hover", Self::position(&uri, line, character), REQUEST_TIMEOUT).await?;
        let contents = res.get("contents").cloned().unwrap_or(Value::Null);
        let text = match &contents {
            Value::String(s) => s.clone(),
            Value::Object(o) => o.get("value").and_then(Value::as_str).unwrap_or_default().to_string(),
            Value::Array(a) => a.iter().map(|v| v.as_str().map(str::to_string).or_else(|| v.get("value").and_then(Value::as_str).map(str::to_string)).unwrap_or_default()).collect::<Vec<_>>().join("\n\n"),
            _ => String::new(),
        };
        Ok(Some(text).filter(|t| !t.trim().is_empty()))
    }

    pub async fn definition(&self, path: &FilePath, line: u32, character: u32) -> Result<Vec<SourceLocation>> {
        let uri = self.sync(path).await?;
        let res = self.client.request("textDocument/definition", Self::position(&uri, line, character), REQUEST_TIMEOUT).await?;
        Ok(self.locations(res))
    }

    pub async fn references(&self, path: &FilePath, line: u32, character: u32) -> Result<Vec<SourceLocation>> {
        let uri = self.sync(path).await?;
        let mut params = Self::position(&uri, line, character);
        params["context"] = json!({"includeDeclaration": true});
        let res = self.client.request("textDocument/references", params, REQUEST_TIMEOUT).await?;
        Ok(self.locations(res))
    }

    /// Functions, methods, impls etc. of a file as flat line ranges (1-based, inclusive).
    pub async fn symbols(&self, path: &FilePath) -> Result<Vec<Symbol>> {
        let uri = self.sync(path).await?;
        let res = self.client.request("textDocument/documentSymbol", json!({"textDocument": {"uri": uri}}), REQUEST_TIMEOUT).await?;
        let mut out = Vec::new();
        fn walk(items: &[Value], out: &mut Vec<Symbol>) {
            for it in items {
                let range = it.get("range").or_else(|| it.pointer("/location/range"));
                if let Some(r) = range {
                    out.push(Symbol {
                        name: it.get("name").and_then(Value::as_str).unwrap_or_default().to_string(),
                        kind: it.get("kind").and_then(Value::as_u64).unwrap_or(0) as u32,
                        start: r.pointer("/start/line").and_then(Value::as_u64).unwrap_or(0) as u32 + 1,
                        end: r.pointer("/end/line").and_then(Value::as_u64).unwrap_or(0) as u32 + 1,
                    });
                }
                if let Some(children) = it.get("children").and_then(Value::as_array) {
                    walk(children, out);
                }
            }
        }
        if let Value::Array(items) = res {
            walk(&items, &mut out);
        }
        Ok(out)
    }

    /// Normalises Location | Location[] | LocationLink[] and attaches line previews.
    fn locations(&self, res: Value) -> Vec<SourceLocation> {
        let items = match res {
            Value::Array(a) => a,
            Value::Null => Vec::new(),
            single => vec![single],
        };
        let mut files: HashMap<String, Vec<String>> = HashMap::new();
        items
            .iter()
            .filter_map(|item| {
                let uri = item.get("targetUri").or_else(|| item.get("uri"))?.as_str()?;
                let range = item.get("targetSelectionRange").or_else(|| item.get("range"))?;
                let abs = from_uri(uri)?;
                let abs_str = abs.to_string_lossy().into_owned();
                let line = range.pointer("/start/line")?.as_u64()? as u32;
                let lines = files.entry(abs_str.clone()).or_insert_with(|| std::fs::read_to_string(&abs).map(|t| t.lines().map(str::to_string).collect()).unwrap_or_default());
                Some(SourceLocation {
                    repo_path: self.root.relative(Path::new(&abs)),
                    abs_path: abs_str,
                    line,
                    character: range.pointer("/start/character")?.as_u64()? as u32,
                    preview: lines.get(line as usize).map(|l| l.trim().to_string()),
                })
            })
            .collect()
    }
}
