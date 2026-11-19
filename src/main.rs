use std::env;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use clap::Parser;

use weztermhere_shell::launch;

/// Open a WezTerm window in a directory (the same action as the context menu entry).
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The directory to open; defaults to the current directory.
    dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let dir = match args.dir {
        Some(dir) => dir,
        None => env::current_dir().context("cannot read the current directory")?,
    };
    if !dir.is_dir() {
        bail!("not a directory: {}", dir.display());
    }
    let exe = launch::locate().with_context(|| {
        format!(
            "cannot find {}; install WezTerm or set {}",
            launch::WEZTERM_GUI,
            launch::OVERRIDE_ENV
        )
    })?;
    Command::new(exe)
        .args(launch::wezterm_args(&dir))
        .current_dir(&dir)
        .spawn()
        .context("cannot start WezTerm")?;
    Ok(())
}
