# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# weztermhere

An "Open WezTerm here" entry for the Windows 11 (new style) File Explorer context menu, written in Rust.

## Motivation

The new Windows 11 context menu ignores the classic `HKLM\...\Directory\Background\shell\<verb>` registry entries, which only show under "Show more options". A verb must be an `IExplorerCommand` COM class registered through a package identity. This project builds that class as a DLL and registers it with a *sparse package* (a manifest-only MSIX that points at the DLL through an external location), so nothing else is installed.

## Status

The DLL, the CLI and the packaging scripts are implemented. The package is built and signed by `scripts/build.ps1`; `scripts/install.ps1` registers it and needs an elevated shell (the certificate goes into `LocalMachine\TrustedPeople`).

## Commands

```bash
cargo build                      # build
cargo test                       # all tests
cargo clippy --all-targets       # lint
cargo fmt                        # format
cargo run -- <DIR>               # open WezTerm in DIR (the CLI shares code with the menu entry)
```

```powershell
pwsh scripts\build.ps1           # release build, then assemble and sign dist\weztermhere.msix (not elevated)
pwsh scripts\install.ps1         # elevated: trust the certificate, register the package with dist\ as external location
pwsh scripts\uninstall.ps1       # elevated: remove the package and the certificate
```

Use `pwsh` (PowerShell 7), not Windows PowerShell 5.1: the latter fails to see the `Cert:` drive when started from Git Bash. Explorer keeps the DLL loaded, so run `uninstall.ps1` (or `Stop-Process -Name explorer`) before rebuilding `dist\`. Installing registers a package and trusts a self-signed certificate system-wide; ask the user first.

## Architecture

- `src/launch.rs`: pure logic. Finds `wezterm-gui.exe` (the `WEZTERMHERE_EXE` override, then `%ProgramFiles%\WezTerm`, `%LOCALAPPDATA%\Programs\WezTerm`, then `PATH`) and builds the arguments `start --no-auto-connect --cwd <dir>`, the same as the legacy registry verb. Unit-tested.
- `src/shell.rs`: all `unsafe` FFI. The `IExplorerCommand` class `WezTermHere`, its class factory and the `DllGetClassObject` / `DllCanUnloadNow` exports. `Invoke` takes the first item of the `IShellItemArray` (the folder, or the folder whose background was clicked) as the working directory. The CLSID in `shell.rs` must match the one in `packaging/AppxManifest.xml`.
- `src/main.rs`: the `weztermhere [DIR]` CLI. It also serves as the `Executable` that the sparse package manifest requires.
- `packaging/AppxManifest.xml`: the sparse package. `Publisher` must equal the certificate subject in `scripts/build.ps1`. `windows.fileExplorerContextMenus` registers the verb for `Directory` and `Directory\Background`; `windows.comServer` maps the CLSID to `weztermhere_shell.dll`.
- `dist/`: build output and the external location (gitignored). The library is named `weztermhere_shell` so its `.dll` does not collide with the CLI's `.exe`/`.pdb`.

The `windows` and `windows-core` crates must stay on the same version, because the `#[implement]` macro refers to `windows_core` directly.

## Conventions (from vischat)

- Error handling: `anyhow` with `Result<T>`. CLI parsing: `clap` with the `derive` feature. Rust edition 2024.
- Put unit tests in the same file as the code, and integration and CLI tests in `tests/`.
- Prefer a `use` import to an absolute path at the point of use. Write `use crate::foo::Bar;` and then `Bar`, not `crate::foo::Bar` inline.
- Keep each `use` group to a single brace level. A flat list is fine (`use crate::model::{A, B};`), but put nested paths on their own lines (`use ratatui::Terminal;` and `use ratatui::backend::CrosstermBackend;`, not `use ratatui::{Terminal, backend::CrosstermBackend};`).
- Add doc comments to public APIs.
- Do not hard-wrap lines in Markdown docs (README.md, this file). Write each paragraph or list item on one line.
- `x_*` and `tmp/` are scratch paths and should be gitignored, so don't commit them.
- Commit messages: in English. The headline has fewer than 10 words, starts with a capitalized verb and has no trailing period. Details go in a bullet list after one blank line. Wrap code items and file paths in backticks. Put the most important change first. Avoid vague verbs ("refactor", "enhance", "improve") when a more precise one exists. Describe moves and renames as moves and renames, not as a deletion plus an addition. A minor change gets a one-line message.
