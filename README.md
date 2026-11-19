# weztermhere

Adds "Open WezTerm here" to the Windows 11 File Explorer context menu (the new style menu, not just "Show more options"), for both a folder and the background of an open folder.

## Install

Requires Rust, Visual Studio's MSVC tools and the Windows SDK.

```powershell
pwsh scripts\build.ps1      # builds and signs dist\weztermhere.msix with a self-signed certificate
pwsh scripts\install.ps1    # from an elevated shell: trusts the certificate and registers the package
```

If the entry does not show up, restart Explorer with `Stop-Process -Name explorer`.

To remove it, run `scripts\uninstall.ps1` from an elevated shell.

## How it works

Windows 11 only shows a verb in the new menu if it is an `IExplorerCommand` COM class registered by a package. `weztermhere_shell.dll` implements that class, and a sparse package (`packaging\AppxManifest.xml`) registers it with `dist\` as the external location. Choosing the entry runs `wezterm-gui.exe start --no-auto-connect --cwd <folder>`.

`wezterm-gui.exe` is looked up from `WEZTERMHERE_EXE`, then `%ProgramFiles%\WezTerm`, `%LOCALAPPDATA%\Programs\WezTerm` and `PATH`.

## CLI

```
weztermhere [DIR]
```

Opens WezTerm in `DIR` (default: the current directory), the same action as the menu entry.
