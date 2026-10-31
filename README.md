# WARNING!

At this stage, this project (except this and the next paragraps) is a full-ai-slop. Use it at your own risk. Treat it as a proof of concept — although a fully usable and incredibly useful one.

...which is kind of funny, considering that this project exists to deal with the huge amount of changes recently introduced mainly by AI. So, essentially, it's AI slop that prevents AI slop.

Hopefully, one day I'll use it to review its own code and polish it to the point where I feel comfortable removing this warning.

### Small disclaimer

I'm not a vibe coder, and I'm strongly against vibe coding anything beyond a proof of concept or an experiment.

That doesn't mean I'm against AI, though. Quite the opposite. I believe that responsible use of AI (which, hopefully, this project can make easier) opens up new possibilities and brings real value to us developers.

---

# Gicio

<img src="docs/icon.png" alt="Gicio: a smiling OK hand gesture" width="128" height="128" />

**gicio** is a desktop app for reviewing huge changes, such as a branch an AI agent wrote for you.
Instead of scrolling through a 3000-line diff, you get a queue of small, logically ordered review steps.
You accept code line by line, put things off for later, and ask questions right where you have them.
An AI (Claude Code, Codex, Copilot CLI or any command you like) answers. When you ask, it fixes the code,
and the queue adjusts to its edits.

**Works with** the AI coding CLIs you already have, with no API keys of its own:

- **Claude Code** (`claude`)
- **OpenAI Codex CLI** (`codex`), also the one bundled in the ChatGPT desktop app, signed in with your
  ChatGPT plan
- **GitHub Copilot CLI** (`copilot`)
- **anything else** you can run as a command that reads a prompt on stdin and prints the answer, such
  as a local model through `ollama run …`

So far I have tested it properly only with Claude Code.

## Why "gicio"?

Gicio (roughly "GHEE-cho") is a playful, affectionate way to say "all good" in Polish. It comes from git, which in everyday Polish today means "good" or "OK". A change that passes review is gicio - and the name gives a nod to Git, too.

![Reviewing a step planned by the AI, with an AI note, a discussion and background AI activity](docs/screenshots/review.png)

## How it works

1. Point gicio at a repository and choose what to review: **uncommitted changes** (staged, unstaged
   and new files) or **everything since a branch or commit** (e.g. an AI's branch against `main`).
2. The AI reads the whole diff and plans **review steps**: one concern each, ordered so that
   understanding builds up (types → logic → wiring → tests). A big hunk or a new file can be split across
   steps. The AI adds short **notes (?)** where a reviewer would likely wonder "why?".
3. You go through the steps. Every changed line is **pending**, **accepted** or **put off for later**.
   Marks follow the code when the diff moves.
4. Select lines and **comment**. Each comment is either a note for yourself, a question the AI answers
   **now**, or an **AI to-do** that runs in the background while you keep reviewing.
5. When the AI edits code, gicio re-reads the working tree. Changed hunks stay in their step, and new
   code becomes a follow-up step (after the current one for answers, at the end of the queue for to-dos).

Nothing is committed, ever. The AI only edits your working tree, and only when you let it.

## Features

**Reviewing**
- Queue of AI-planned steps (or plain file by file), with a "Done" list and progress everywhere.
- Side-by-side or unified diff (added and deleted files are always unified), with syntax highlighting
  and word-level highlights of what changed inside a line. Whitespace-only changes can be ignored.
- More context on demand: 20 more lines above or below a hunk, or the whole enclosing function
  (via rust-analyzer, with a guess for other languages).
- Per-line marks: click the gutter dot to toggle a line, drag in the gutter to select a range, then
  accept, put off for later or reset, using the keyboard or the floating bar.
- Accept whole code items at once: a "✓ fn new", "✓ struct Store", "✓ impl …", "✓ use ×4",
  "✓ thread_local!" button sits on each changed item's first line (also in the right-click menu), and
  hunks inside a function offer to accept the enclosing one.
- "Later" tab: everything you put off, lines and discussions, to come back to at the end.
- File tree with per-file and per-folder progress. Files currently on screen are highlighted, and
  the file under your pointer is highlighted more subtly.
- Sticky file bar while scrolling, with one-click "Accept file" / "Accept this part". Accepted files
  collapse, and anything accepted can be taken back ("↺ Unaccept" on a hunk, file or step, also from
  the Done list). Jump between files with `f` / `Shift+F`.
- Back / forward navigation (⌘− / ⌘+), undo for marks, "Start over".

**From review to commit**
- Optionally **stage what you accept**. Accepted lines go to the git index, including parts of files,
  and nothing else does, so you can commit exactly what you reviewed. Choose it on the start screen
  (for uncommitted changes); everything previously staged is unstaged first.
- **Commit** dialog with an AI-suggested message for what is staged.
- **Export** the review (progress, summary, discussions, open findings) as markdown.

**Talking to the AI**
- Discussions anchored to lines, with four actions:
  - **Note**: just for you.
  - **Add to review**: collect comments, then **Submit review** sends them all to the AI as one task,
    like a GitHub review.
  - **Ask now**: the AI answers now.
  - **Add AI to-do**: queued background work; its changes land at the end of the queue.
- Discussions, findings and AI notes follow their code when it moves; when it is changed or removed
  they are marked *outdated* and disappear from the code.
- Live **AI activity** panel showing what the AI is doing (files read, edits, commands), with **Stop**.
- **Chat** tab for questions about the change as a whole, plus a one-click **summary**: what it does,
  how, risks, and where to look closely.
- Backends: Claude Code (`claude -p`), OpenAI Codex (`codex exec`), GitHub Copilot CLI (`copilot -p`)
  or a custom command that reads the prompt on stdin. Model and reasoning effort are configurable,
  also in the middle of a review.
- Project instructions and skills are respected: `CLAUDE.md`, `AGENTS.md`,
  `.github/copilot-instructions.md`, `.github/instructions/`, `.cursor/rules`, `GEMINI.md` and skills
  in `.claude/skills`, `.agents/skills`, `.codex/skills` and `.github/skills`. Whatever the chosen CLI
  does not read by itself is added to the prompt. Your own standing instructions from Settings come on top.
- Answers are instructed to be short and strictly on topic.

**Catching slop**
- **AI review** of the whole change, one step, or selected lines. The AI acts as a strict reviewer and
  reports findings (bugs, edge cases, swallowed errors, dead code, made-up APIs…) next to the code
  and in a Findings tab, with **Fix**, **Fix all**, **Discuss** and **Dismiss**.
- Lock files and generated code (`Cargo.lock`, `package-lock.json`, `*.min.js`, `dist/**`…) are left
  out of AI planning, reviews and summaries by default; the patterns are configurable in Settings.
- **cargo fmt**, **cargo clippy** (warnings become findings) and **cargo clippy --fix** from the Tools menu.

**Code navigation and editing**
- rust-analyzer integration: hover, ⌘-click go to definition, right-click find references, and
  **rename** (F2) across the whole project. Lines you had already accepted stay accepted (or ask for
  another look, your choice), and lines the rename adds to the change count as your own edit. If the
  target is already on screen, the view scrolls there; otherwise the file opens in the main window.
- Open any changed file in a built-in editor (CodeMirror). Lines you write yourself count as
  accepted, and the diff and queue update on save.

**Settings**
- Default AI, model and reasoning; your own instructions for the AI; preferences (default layout,
  default order, auto-advance, notes expanded, hover delay…).
- Every keyboard shortcut can be changed, and every action has at least two of them.

## Screenshots

| | |
|---|---|
| ![Start screen](docs/screenshots/start.png) | ![Chat with a summary of the change](docs/screenshots/chat.png) |
| Start: what to review, in which order, with which AI | Chat about the whole change, with a generated summary |
| ![File tree and unified diff](docs/screenshots/files.png) | ![Built-in editor](docs/screenshots/file.png) |
| File tree with progress, unified layout | Built-in editor with rust-analyzer |
| ![Settings: shortcuts](docs/screenshots/settings.png) | |
| Every shortcut is configurable | |

The screenshots show a review of doppio (`cargo install doppio-bar`), a small macOS menu bar app,
done with Claude Code. All AI texts in them are real output.

## Running

Requirements: Rust, Node.js (the frontend is built automatically), `git`, and at least one of `claude`,
`codex` or `copilot` on your PATH. For Rust projects, `rust-analyzer` enables code navigation.

Install:

```sh
cargo install --git https://github.com/synek317/gicio gicio
```

Then run it inside the repository you want to review:

```sh
gicio                          # this repository, start screen prefilled
gicio ~/code/app --since main  # everything since main in another repository
gicio --start                  # skip the start screen, review uncommitted changes right away
```

Tick "Remember these choices" on the start screen and `gicio` opens the review straight away from
then on. Everything the start screen asks (what to review, staging, order, AI) can be changed during a
review from the **Start over ▾** menu, and the defaults in Settings → Preferences.

Or run it from a clone:

```sh
cargo run --release
```

or install your local checkout with `cargo install-local` (an alias for
`cargo install --path src-tauri --locked --force`).

For working on gicio itself, `npm install && npm run tauri dev` gives hot reload, and
`npm run tauri build` makes a macOS app bundle.

If a CLI is not found (apps started from Finder get a minimal PATH), point gicio at it with
`GICIO_CLAUDE_BIN`, `GICIO_CODEX_BIN`, `GICIO_COPILOT_BIN` or `GICIO_RUST_ANALYZER_BIN`.

Review state lives in `.git/gicio/` inside the reviewed repository, one review per scope (uncommitted
changes, or everything since a ref), so you can close the app and continue later, also after commits
or small edits in between: the plan, your marks on unchanged lines and the discussions carry over, and
gicio tells you what moved. If the change is completely different (say you threw it away and started
over), you get a fresh review and the old one is archived next to it. While gicio runs, edits made
outside it (your editor, another agent) are picked up automatically. App settings live in the app's config directory.

## License

gicio is **free for non-commercial use** under the
[PolyForm Noncommercial License 1.0.0](LICENSE): personal use, hobby projects, research, education,
and use by charities and public institutions.

For commercial use, please contact me at [synek317@gmail.com](mailto:synek317@gmail.com). Honestly though, I don't
recommend it: this is an experimental proof of concept (see the warning at the top), not something to
build a business on.
