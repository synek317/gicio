# Ideas

Features worth considering next, most promising first.

## Run tests from the Tools menu

Run `cargo test` and `dotnet test` from the Tools menu and turn failed tests into findings.

To check first: a compiler error points at a line, but a failed test does not always point at the
line that caused it. Failures without a usable location may need their own representation.

## Review GitHub pull requests

Open a pull request by number or URL, and publish conversations and findings back as review comments.

This is the most expensive idea here. gicio compares the working tree with a base commit, so a
pull request would need its own worktree. It also needs GitHub authentication and a mapping from
gicio's anchors to GitHub's diff positions.

## Language servers for TypeScript and Python

Code navigation (hover, go to definition, find references, rename) works only for Rust and C#, even
though the detection of code items already covers more languages, such as Python. Add
`typescript-language-server` and a Python server such as `pyright` to `crates/gicio-core/src/lsp`,
with the same offer to install a missing server.
