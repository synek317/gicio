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
    Anchor { path, side, start: line, end: line, excerpt, outdated: false }
}
use std::path::PathBuf;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut backend = AiBackend::default();
    match std::env::var("GICIO_ENGINE").as_deref() {
        Ok("codex") => backend.engine = ai::Engine::Codex,
        Ok("copilot") => backend.engine = ai::Engine::Copilot,
        _ => {}
    }
    let mut s = Session::open(&PathBuf::from(&args[0]), GitRef::new(&args[1])?, backend.clone(), gicio_core::stage::StageMode::Memory)?;
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
            let t = s.create_thread(Some(chunk.clone()), None, true);
            s.push_message(&t, gicio_core::review::Message::new(gicio_core::review::Author::Human, gicio_core::review::MessageKind::Text, args[3].clone()))?;
            let prompt = ai::build_ask_prompt(&s, &t, &Default::default())?;
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
            let t = s.create_thread(Some(chunk.id.clone()), Some(anchor_at(anchor.0.clone(), anchor.1, anchor.2, excerpt)), false);
            let q = "Why is it done this way? Is there a simpler option?";
            s.push_message(&t, Message::new(Author::Human, MessageKind::Text, q))?;
            let prompt = ai::build_ask_prompt(&s, &t, &Default::default())?;
            let answer = ai::run_streaming(&backend, &s.state.repo, &prompt, AccessMode::ReadOnly, sink(&logs), None).await?;
            s.push_message(&t, Message::new(Author::Ai { backend: backend.label() }, MessageKind::Text, answer.trim()))?;
            let ask_log = std::mem::take(&mut *logs.lock().unwrap());
            eprintln!("answered");

            let prompt = ai::build_summary_prompt(&s, &Default::default(), &Default::default());
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
                {"id": "j2", "kind": "review", "status": "running", "title": "Add a unit test for the countdown formatting", "thread": null, "log": ask_log},
            ]);
            s.save()?;
            std::fs::write(&args[3], serde_json::to_vec(&serde_json::json!({"snapshot": s.snapshot(), "texts": texts, "jobs": jobs}))?)?;
        }
        Some("rename") => {
            // rename <path> <line0> <col0> <new>: accept everything first, then rename with RA.
            let keys: Vec<_> = s.files.iter().flat_map(|f| f.hunks.iter().flat_map(move |h| h.lines.iter().filter_map(move |l| l.review_pos().map(|(side, line)| gicio_core::review::LineKey { path: f.path.clone(), side, line })))).take(20).collect();
            s.set_marks(keys, Some(gicio_core::review::MarkStatus::Accepted));
            let accepted_before = Vec::<gicio_core::review::Mark>::from(s.state.marks.clone()).len();
            let ra = gicio_core::lsp::RustAnalyzer::start(s.state.repo.clone()).await?;
            let path = gicio_core::domain::FilePath::new(&args[3])?;
            let (line, col): (u32, u32) = (args[4].parse()?, args[5].parse()?);
            for _ in 0..90 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if let Ok(Some(_)) = ra.hover(&path, line, col).await { break; }
            }
            println!("prepare: {:?}", ra.prepare_rename(&path, line, col).await?);
            let ops = ra.rename(&path, line, col, &gicio_core::lsp::NewName::new(&args[6])?).await?;
            let paths: Vec<_> = ops.iter().flat_map(|o| o.paths()).filter_map(|p| s.state.repo.relative(p)).collect();
            println!("ops: {} files, {} edits", ops.len(), ops.iter().map(|o| o.edit_count()).sum::<usize>());
            s.record_rename(&paths, &[], true, None, || gicio_core::lsp::apply_file_ops(&ops))?;
            let marks: Vec<gicio_core::review::Mark> = s.state.marks.clone().into();
            println!("marks before {accepted_before}, after {} ({} accepted)", marks.len(), marks.iter().filter(|m| m.status == gicio_core::review::MarkStatus::Accepted).count());
            return Ok(());
        }
        Some("anchors") => {
            let path = FilePath::new("src/item.rs")?;
            let excerpt = s.excerpt_of(&path, Side::New, 11, 11);
            let t = s.create_thread(None, Some(Anchor { path: path.clone(), side: Side::New, start: 11, end: 11, excerpt, outdated: false }), false);
            let show = |s: &Session, what: &str| {
                let a = s.thread(&t).unwrap().anchor.clone().unwrap();
                println!("{what}: lines {}-{} outdated={} ({:?})", a.start, a.end, a.outdated, a.excerpt.trim());
            };
            show(&s, "created");
            let abs = s.state.repo.join(&path);
            let text = std::fs::read_to_string(&abs)?;
            std::fs::write(&abs, format!("// one\n// two\n// three\n{text}"))?;
            s.refresh(None)?;
            show(&s, "after 3 lines inserted above");
            let without: String = text.lines().filter(|l| !l.contains("to_lowercase")).map(|l| format!("{l}\n")).collect();
            std::fs::write(&abs, without)?;
            s.refresh(None)?;
            show(&s, "after the line was deleted");
            std::fs::write(&abs, &text)?;
            s.refresh(None)?;
            show(&s, "after it came back");
            return Ok(());
        }
        Some("status") => {
            let marks: Vec<gicio_core::review::Mark> = s.state.marks.clone().into();
            println!("notice: {:?}\nchunks: {:?}\nmarks: {}", s.notice, s.state.chunks.iter().map(|c| c.title.clone()).collect::<Vec<_>>(), marks.len());
            return Ok(());
        }
        Some("mark") => {
            let keys: Vec<_> = s.files.iter().flat_map(|f| f.hunks.iter().flat_map(move |h| h.lines.iter().filter_map(move |l| l.review_pos().map(|(side, line)| gicio_core::review::LineKey { path: f.path.clone(), side, line })))).take(8).collect();
            s.set_marks(keys, Some(gicio_core::review::MarkStatus::Accepted));
            s.save()?;
            println!("marked 8 lines");
            return Ok(());
        }
        Some("stage") => {
            // Re-open in index mode and accept every other added line of src/store.rs.
            let backend = s.state.backend.clone();
            let mut s = Session::open(&PathBuf::from(&args[0]), GitRef::new(&args[1])?, backend, gicio_core::stage::StageMode::Index)?;
            let path = gicio_core::domain::FilePath::new("src/store.rs")?;
            let keys: Vec<_> = s.file(&path).unwrap().hunks.iter().flat_map(|h| h.lines.iter())
                .filter_map(|l| l.review_pos()).filter(|(side, _)| *side == Side::New).step_by(2)
                .map(|(side, line)| gicio_core::review::LineKey { path: path.clone(), side, line }).collect();
            println!("accepting {} lines", keys.len());
            s.set_marks(keys, Some(gicio_core::review::MarkStatus::Accepted));
            let out = std::process::Command::new("git").arg("-C").arg(&args[0]).args(["diff", "--cached", "--stat"]).output()?;
            println!("{}", String::from_utf8_lossy(&out.stdout));
            let out = std::process::Command::new("git").arg("-C").arg(&args[0]).args(["diff", "--cached", "--", "src/store.rs"]).output()?;
            println!("{}", String::from_utf8_lossy(&out.stdout).lines().take(30).collect::<Vec<_>>().join("\n"));
        }
        Some("aireview") => {
            let refs: Vec<_> = s.state.chunks[0].hunks.clone();
            let paths = refs.iter().map(|r| r.path.clone()).collect();
            let prompt = ai::build_ai_review_prompt(&s, &refs, None, &Default::default());
            let answer = ai::run(&backend, &s.state.repo, &prompt, AccessMode::ReadOnly).await?;
            let raw = ai::parse_findings(&answer).ok_or_else(|| anyhow::anyhow!("unparsable: {answer}"))?;
            let n = s.add_ai_findings(raw, &paths);
            for f in &s.state.findings {
                println!("[{:?}] {}:{} {} — {}", f.severity, f.path, f.line, f.title, f.detail);
            }
            println!("{n} findings");
            s.save()?;
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
