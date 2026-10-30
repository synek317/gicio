//! Dev helper: `cargo run -p gicio-core --example cli -- <repo> <base> [plan|ask <question>]`
use gicio_core::ai::{self, AccessMode, AiBackend};
use gicio_core::domain::GitRef;
use gicio_core::review::{Anchor, Author, Message, MessageKind, Session};
use gicio_core::diff::Side;
use gicio_core::domain::FilePath;

fn side_new() -> Side {
    Side::New
}

fn anchor_at(path: FilePath, side: Side, line: u32, excerpt: String) -> Anchor {
    Anchor { path, side, start: line, end: line, excerpt }
}
use std::path::PathBuf;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let backend = AiBackend::default();
    let mut s = Session::open(&PathBuf::from(&args[0]), GitRef::new(&args[1])?, backend.clone())?;
    match args.get(2).map(String::as_str) {
        Some("plan") => {
            let req = ai::PlanRequest {
                backend,
                repo: s.state.repo.clone(),
                base_ref: s.state.base_ref.clone(),
                base_commit: s.state.base_commit.clone(),
                hunks: s.files.iter().flat_map(|f| f.hunks.clone()).collect(),
                kept_titles: vec![],
                extra: Default::default(),
            };
            let planned = ai::plan(req, Box::new(|l| eprintln!("  | {l}")), None).await?;
            s.apply_plan(vec![], planned, true);
        }
        Some("ask") => {
            let out = ai::run(&backend, &s.state.repo, &args[3], AccessMode::ReadOnly).await?;
            println!("{out}");
        }
        Some("thread") => {
            // Mimics the app's send_message: ask on the first chunk containing store.rs, allow edits.
            let chunk = s.state.chunks.iter().find(|c| c.hunks.iter().any(|h| h.path.as_str() == "src/store.rs")).unwrap().id.clone();
            let t = s.create_thread(Some(chunk.clone()), None, true, false);
            s.push_message(&t, gicio_core::review::Message::new(gicio_core::review::Author::Human, gicio_core::review::MessageKind::Text, args[3].clone()))?;
            let prompt = ai::build_ask_prompt(&s, &t, false, &Default::default())?;
            let answer = ai::run(&backend, &s.state.repo, &prompt, AccessMode::Edit).await?;
            println!("AI: {answer}\n");
            let report = s.refresh(Some((Some(chunk), "Follow-up".into(), Some(t))))?;
            println!("report: {report:?}\n");
        }
        Some("demo") => {
            // Runs the real AI through planning, one question, a summary and a chat turn, then
            // dumps everything (with activity logs) as the browser fixture used for screenshots.
            let logs = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
            let sink = |logs: &std::sync::Arc<std::sync::Mutex<Vec<String>>>| -> ai::LogSink {
                let l = logs.clone();
                Box::new(move |line| l.lock().unwrap().push(line))
            };
            let req = ai::PlanRequest {
                backend: backend.clone(),
                repo: s.state.repo.clone(),
                base_ref: s.state.base_ref.clone(),
                base_commit: s.state.base_commit.clone(),
                hunks: s.files.iter().flat_map(|f| f.hunks.clone()).collect(),
                kept_titles: vec![],
                extra: Default::default(),
            };
            let planned = ai::plan(req, sink(&logs), None).await?;
            s.apply_plan(vec![], planned, true);
            let plan_log = std::mem::take(&mut *logs.lock().unwrap());
            eprintln!("planned {} chunks", s.state.chunks.len());

            let chunk = s.state.chunks.iter().find(|c| !c.notes.is_empty()).or(s.state.chunks.get(1)).unwrap().clone();
            let anchor = chunk.notes.first().map(|n| (n.path.clone(), n.side, n.line)).unwrap_or_else(|| {
                let h = s.hunk(&chunk.hunks[0].id).unwrap();
                (h.path.clone(), side_new(), h.new_start)
            });
            let excerpt = s.file(&anchor.0).and_then(|f| f.hunks.iter().flat_map(|h| h.lines.iter()).find(|l| l.new_no == Some(anchor.2)).map(|l| l.text.clone())).unwrap_or_default();
            let t = s.create_thread(Some(chunk.id.clone()), Some(anchor_at(anchor.0.clone(), anchor.1, anchor.2, excerpt)), false, false);
            let q = "Why is it done this way? Is there a simpler option?";
            s.push_message(&t, Message::new(Author::Human, MessageKind::Text, q))?;
            let prompt = ai::build_ask_prompt(&s, &t, false, &Default::default())?;
            let answer = ai::run_streaming(&backend, &s.state.repo, &prompt, AccessMode::ReadOnly, sink(&logs), None).await?;
            s.push_message(&t, Message::new(Author::Ai { backend: backend.label() }, MessageKind::Text, answer.trim()))?;
            let ask_log = std::mem::take(&mut *logs.lock().unwrap());
            eprintln!("answered");

            let prompt = ai::build_summary_prompt(&s, &Default::default());
            let summary = ai::run_streaming(&backend, &s.state.repo, &prompt, AccessMode::ReadOnly, sink(&logs), None).await?;
            s.state.summary = Some(Message::new(Author::Ai { backend: backend.label() }, MessageKind::Text, summary.trim()));
            logs.lock().unwrap().clear();
            eprintln!("summarized");

            s.state.chat.push(Message::new(Author::Human, MessageKind::Text, "Which part would you review first, and why?"));
            let prompt = ai::build_chat_prompt(&s, &Default::default(), false);
            let reply = ai::run_streaming(&backend, &s.state.repo, &prompt, AccessMode::ReadOnly, sink(&logs), None).await?;
            s.state.chat.push(Message::new(Author::Ai { backend: backend.label() }, MessageKind::Text, reply.trim()));
            eprintln!("chatted");

            let mut texts = serde_json::Map::new();
            for f in &s.files {
                let t = gicio_core::diff::file_texts(&s.state.repo, &s.state.base_commit, &f.path, f.old_path.as_ref(), f.status)?;
                texts.insert(f.path.to_string(), serde_json::to_value(t)?);
            }
            let jobs = serde_json::json!([
                {"id": "j1", "kind": "plan", "status": "done", "title": "Plan review steps", "thread": null, "log": plan_log},
                {"id": "j2", "kind": "todo", "status": "running", "title": "Add a unit test for the countdown formatting", "thread": null, "log": ask_log},
            ]);
            s.save()?;
            std::fs::write(&args[3], serde_json::to_vec(&serde_json::json!({"snapshot": s.snapshot(), "texts": texts, "jobs": jobs}))?)?;
        }
        Some("edit") => {
            let path = gicio_core::domain::FilePath::new("src/main.rs")?;
            let text = std::fs::read_to_string(s.state.repo.join(&path))? + "\n// reviewer was here\n";
            let report = s.save_user_edit(&path, &text, None)?;
            let marks: Vec<_> = Vec::<gicio_core::review::Mark>::from(s.state.marks.clone());
            println!("report {report:?}\nmarks: {:?}", marks.iter().map(|m| (m.key.line, &m.text, m.status)).collect::<Vec<_>>());
            s.reset();
            println!("after reset: {} marks, {} chunks", Vec::<gicio_core::review::Mark>::from(s.state.marks.clone()).len(), s.state.chunks.len());
        }
        Some("lsp") => {
            let ra = gicio_core::lsp::RustAnalyzer::start(s.state.repo.clone()).await?;
            let path = gicio_core::domain::FilePath::new("src/main.rs")?;
            for _ in 0..60 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if let Ok(Some(h)) = ra.hover(&path, 3, 12).await {
                    println!("status {:?}\nhover: {h}", ra.status());
                    break;
                }
            }
            println!("definition: {:?}", ra.definition(&path, 3, 12).await?);
        }
        Some("dump") => {
            let mut texts = serde_json::Map::new();
            for f in &s.files {
                let t = gicio_core::diff::file_texts(&s.state.repo, &s.state.base_commit, &f.path, f.old_path.as_ref(), f.status)?;
                texts.insert(f.path.to_string(), serde_json::to_value(t)?);
            }
            let out = serde_json::json!({"snapshot": s.snapshot(), "texts": texts});
            std::fs::write(&args[3], serde_json::to_vec(&out)?)?;
        }
        _ => {}
    }
    for c in &s.state.chunks {
        println!("[{}] {} — {}", c.id, c.title, c.summary);
        for r in &c.hunks {
            println!("    {} {} {:?}", r.path, r.id, r.part.map(|p| p.range()));
        }
        for n in &c.notes {
            println!("    ? {}:{} {}", n.path, n.line, n.text);
        }
    }
    if let Ok(out) = std::env::var("GICIO_DUMP") {
        let mut texts = serde_json::Map::new();
        for f in &s.files {
            let t = gicio_core::diff::file_texts(&s.state.repo, &s.state.base_commit, &f.path, f.old_path.as_ref(), f.status)?;
            texts.insert(f.path.to_string(), serde_json::to_value(t)?);
        }
        std::fs::write(out, serde_json::to_vec(&serde_json::json!({"snapshot": s.snapshot(), "texts": texts}))?)?;
    }
    println!("{} files, {} bytes snapshot", s.files.len(), serde_json::to_string(&s.snapshot())?.len());
    Ok(())
}
