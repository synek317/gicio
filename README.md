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

![Reviewing a step planned by the AI, with an AI note, a conversation and background AI activity](docs/screenshots/review.png)

## How it works

1. Point gicio at a repository and choose what to review: **uncommitted changes** (staged, unstaged
   and new files) or **everything since a branch or commit** (e.g. an AI's branch against `main`).
2. The AI reads the whole diff and plans **review steps**: one feature at a time, a large feature split
   into several steps (shown nested under it), ordered so that understanding builds up
   (types → logic → wiring → tests). A big hunk or a new file can be split across steps. The AI adds
   short **notes (?)** where a reviewer would likely wonder "why?".
3. You go through the steps. Every changed line is **pending**, **accepted** or **put off for later**.
   Marks follow the code when the diff moves. Finished steps stay in the list, struck through and
   stamped "Done".
4. Select lines and type your question in the box that appears next to them: **start a conversation**
   (the AI answers now) or **add it for later**.
   Everything left for later, lines and conversations, waits in the last step, **Resolve issues left
   for later**, where the AI can take all of it in one go.
5. When the AI edits code, gicio re-reads the working tree. Changed hunks stay in their step, and new
   code becomes a follow-up step (after the discussed step, or at the end for issues resolved later).

Nothing is committed, ever. The AI only edits your working tree, and only when you let it.

## Features

**Reviewing**
- One list of AI-planned steps (or plain file by file), grouped by feature, with progress everywhere;
  finished steps are struck through and stamped "Done".
- Side-by-side or unified diff (added and deleted files are always unified), with syntax highlighting
  and word-level highlights of what changed inside a line. Whitespace-only changes can be ignored.
- More context on demand: 20 more lines above or below a hunk, or the whole enclosing function
  (via rust-analyzer or csharp-ls, with a guess for other languages).
- Per-line marks: click the gutter dot to toggle a line, or the bar at the left of a block of changed
  lines to accept the whole block. Drag in the gutter to select a range, then put it off for later,
  reset it or start a conversation about it (keyboard or the floating bar).
- Accept whole code items at once: a "✓ fn new", "✓ struct Store", "✓ impl …", "✓ use ×4",
  "✓ thread_local!" (in C#: "✓ class Order", "✓ method GetAsync", "✓ property Name", "✓ using ×3")
  button sits on each changed item's first line (also in the right-click menu), and
  hunks inside a function offer to accept the enclosing one.
- "Resolve issues left for later": the last step gathers everything you put off, lines and
  conversations, to come back to at the end.
- File tree with per-file and per-folder progress. Files of the current step are marked, files currently
  on screen are highlighted, and the file under your pointer is highlighted more subtly. The selected
  step also lists its files right under it in the step list.
- Sticky file bar while scrolling, with one-click "Accept file" / "Accept this part". Accepted files
  collapse, and anything accepted can be taken back ("↺ Unaccept" on a hunk, file or step, also from
  the step list). Jump between files with `f` / `Shift+F`.
- Back / forward navigation (⌘− / ⌘+), undo for marks (`z`), "Start over" in the **Review** menu.

**From review to commit**
- Optionally **stage what you accept**. Accepted lines go to the git index, including parts of files,
  and nothing else does, so you can commit exactly what you reviewed. Choose it on the start screen
  (for uncommitted changes); everything previously staged is unstaged first.
- **Commit** dialog with an AI-suggested message for what is staged.
- **Export** the review (progress, summary, discussions, open findings) as markdown.

**Talking to the AI**
- Each step keeps its own history of conversations, and the whole change has one too; a switch at the
  top of the Conversations tab goes between them. Resolved conversations fold away.
- Conversations about lines, a step or the whole change, with two actions:
  - **Start conversation**: the AI answers now.
  - **Add for later**: it waits in the last step; **Let the AI resolve them** sends all of them to the
    AI as one task (like submitting a GitHub review), or you resolve them yourself.
- Conversations, findings and AI notes follow their code when it moves; when it is changed or removed
  they are marked *outdated* and disappear from the code.
- **AI changes** on their own: every AI job that edits code (and `cargo fmt` / `clippy --fix` /
  `dotnet format`) is recorded as a snapshot of the working tree before and after it. "Show changes"
  under its note, or the "AI changes to check" list above the steps, shows only what that job changed.
  **Accept** keeps it and counts the lines it wrote as reviewed; **Reject** puts its files back as they
  were before it (files you edited again since are left alone).
- Live **AI activity** panel showing what the AI is doing (files read, edits, commands), with **Stop**.
- A **Whole change** conversation for questions about the change as a whole, with a one-click
  **summary**: what it does, how, risks, and where to look closely.
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
- **cargo fmt**, **cargo clippy** (whole workspace, all targets; warnings become findings) and
  **cargo clippy --fix** from the Tools menu.
- For C#: **dotnet format** and **dotnet build** (a full rebuild of the solution; compiler and
  analyzer warnings become findings) from the Tools menu.

**Code navigation and editing**
- Language server integration (rust-analyzer for Cargo projects, csharp-ls for .NET ones): hover,
  ⌘-click go to definition, right-click find references, and **rename** (F2) across the whole
  project. Lines you had already accepted stay accepted (or ask for another look, your choice), and
  lines the rename adds to the change count as your own edit. If the target is already on screen, the
  view scrolls there; otherwise the file opens in the main window.
- Edit code **in place**: "✎ Edit" (or `e`) turns a hunk into an editor right in the diff. "Open file"
  shows the whole file in the built-in editor (CodeMirror). Lines you write yourself count as accepted,
  and the diff and steps update on save.

**Settings**
- Default AI, model (with the models the installed CLI names in its help suggested), reasoning and the
  language the AI writes in (your system's language by default); your own instructions for the AI;
  preferences (default layout,
  default order, auto-advance, notes expanded, hover delay…).
- Every keyboard shortcut can be changed, and every action has at least two of them. Modifier + arrow
  keys are left to text editing (jumping by word or line, selecting with Shift).

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

## Installation

gicio runs on macOS (Apple silicon), Linux and Windows. Whichever way you install it, it needs `git`
and at least one of `claude`, `codex` or `copilot` (or your own command, see [Running](#running)).
For Rust projects, `rust-analyzer` enables code navigation; for C# projects (a `.sln`, `.slnx` or
`.csproj` up to three directories deep), `csharp-ls` does. When the project's server is missing, gicio
offers to install it (`rustup component add rust-analyzer`, `dotnet tool update --global csharp-ls`);
without it everything but code navigation works.

Ready-made builds are on the [Releases](https://github.com/synek317/gicio/releases) page.

### macOS (Apple silicon)

With [Homebrew](https://brew.sh):

```sh
brew tap synek317/gicio https://github.com/synek317/gicio
brew trust --formula synek317/gicio/gicio
brew install synek317/gicio/gicio
```

and later `brew upgrade gicio`. `brew trust` is needed since Homebrew 7, which ignores formulae from
other taps until you trust them. This installs the `gicio` command; start it from a terminal.

Or download `gicio_<version>_aarch64.dmg` from the releases and drag gicio to Applications. The app
is not signed with an Apple developer certificate, so macOS says it "is damaged" or "cannot be
verified". Remove the quarantine flag once, after copying it:

```sh
xattr -dr com.apple.quarantine /Applications/gicio.app
```

### Linux (x86_64)

From the releases, pick one:

- **Debian, Ubuntu**: `sudo apt install ./gicio_<version>_amd64.deb`
- **Fedora, openSUSE**: `sudo dnf install ./gicio-<version>-1.x86_64.rpm`
- **anything else**: `gicio_<version>_amd64.AppImage` (`chmod +x` it and run it)

The packages install the `gicio` command and a desktop entry. They need WebKitGTK 4.1, which the
package manager pulls in (Ubuntu 22.04, Debian 12, Fedora 36 or newer).

### Windows (x64)

From the releases, run `gicio_<version>_x64-setup.exe` (or the `.msi`), then start gicio from the
Start menu. There is also a portable `gicio-<version>-x86_64-pc-windows-msvc.zip` with just
`gicio.exe`. It needs [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/), which
Windows 10 and 11 already have, and [Git for Windows](https://git-scm.com/download/win).

### From source (any system)

Requirements: Rust 1.88 or newer, Node.js 20.19 or newer (the frontend is built automatically), and on
Linux the WebKitGTK development packages (see [`install-linux-deps.sh`](.github/scripts/install-linux-deps.sh)
for Debian/Ubuntu, or the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)).
`cargo install` builds the frontend and embeds it in the binary, and puts `gicio` into `~/.cargo/bin`.

Straight from GitHub:

```sh
cargo install --git https://github.com/synek317/gicio gicio --locked
```

Or from a clone, which is handy for trying local changes or a branch:

```sh
git clone https://github.com/synek317/gicio
cd gicio
cargo install-local
```

`cargo install-local` is an alias (defined in [`.cargo/config.toml`](.cargo/config.toml)) for
`cargo install --path src-tauri --locked --force`: the repository root is a workspace, so a plain
`cargo install --path .` does not work there. Run it again after `git pull` to update. To try
gicio without installing it, run `cargo run --release` in the clone.

## Running

Run gicio inside the repository you want to review:

```sh
gicio                          # this repository, start screen prefilled
gicio ~/code/app --since main  # everything since main in another repository
gicio --start                  # skip the start screen, review uncommitted changes right away
```

Tick "Remember these choices" on the start screen and `gicio` opens the review straight away from
then on. Everything the start screen asks (what to review, staging, order, AI) can be changed during a
review from the **Start over ▾** menu, and the defaults in Settings → Preferences.

For working on gicio itself, `npm install && npm run tauri dev` gives hot reload, and
`npm run tauri build` makes the installers for your system (a macOS app bundle, `.deb`/`.rpm`/AppImage,
or `.msi`/setup `.exe`).

A custom AI command runs through `/bin/sh` (on Windows, `cmd.exe`). If a CLI is not found (apps
started from Finder or the Start menu get a minimal PATH), point gicio at it with `GICIO_CLAUDE_BIN`,
`GICIO_CODEX_BIN`, `GICIO_COPILOT_BIN`, `GICIO_RUST_ANALYZER_BIN` or `GICIO_CSHARP_LS_BIN`.

### Releasing

Bump the version in `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` and `package.json`, then push a
matching tag (`git tag v0.2.0 && git push origin v0.2.0`). The [Release](.github/workflows/release.yml)
workflow builds all platforms, publishes a GitHub release and updates `Formula/gicio.rb` for Homebrew.

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
