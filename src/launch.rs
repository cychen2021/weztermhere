//! Locating WezTerm and building the command that opens it in a directory. Pure logic, no Windows calls.

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The file name of the WezTerm GUI executable.
pub const WEZTERM_GUI: &str = "wezterm-gui.exe";

/// The environment variable that overrides where `wezterm-gui.exe` is looked up.
pub const OVERRIDE_ENV: &str = "WEZTERMHERE_EXE";

/// Returns the arguments that open a new WezTerm window with `dir` as its working directory.
///
/// They match the legacy `Open WezTerm here` registry verb. Passing them as separate arguments (instead of one command line) leaves the quoting to `std::process::Command`, so a trailing backslash such as `D:\` is safe.
pub fn wezterm_args(dir: &Path) -> Vec<OsString> {
    vec![
        OsString::from("start"),
        OsString::from("--no-auto-connect"),
        OsString::from("--cwd"),
        dir.as_os_str().to_owned(),
    ]
}

/// Returns the places where `wezterm-gui.exe` may live, most preferred first.
///
/// The override variable comes first, then the per-machine and per-user install directories, then every `PATH` entry.
pub fn candidates(env_lookup: impl Fn(&str) -> Option<OsString>) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Some(path) = env_lookup(OVERRIDE_ENV) {
        found.push(PathBuf::from(path));
    }
    if let Some(dir) = env_lookup("ProgramFiles") {
        found.push(PathBuf::from(dir).join("WezTerm").join(WEZTERM_GUI));
    }
    if let Some(dir) = env_lookup("LOCALAPPDATA") {
        found.push(
            PathBuf::from(dir)
                .join("Programs")
                .join("WezTerm")
                .join(WEZTERM_GUI),
        );
    }
    if let Some(path) = env_lookup("PATH") {
        found.extend(env::split_paths(&path).map(|dir| dir.join(WEZTERM_GUI)));
    }
    found
}

/// Returns the first existing `wezterm-gui.exe` according to [`candidates`].
pub fn locate() -> Option<PathBuf> {
    candidates(|name| env::var_os(name))
        .into_iter()
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(name: &str) -> Option<OsString> {
        match name {
            "ProgramFiles" => Some(OsString::from(r"C:\Program Files")),
            "LOCALAPPDATA" => Some(OsString::from(r"C:\Users\u\AppData\Local")),
            "PATH" => Some(OsString::from(r"C:\a;C:\b")),
            _ => None,
        }
    }

    #[test]
    fn args_keep_the_directory_as_one_argument() {
        let args = wezterm_args(Path::new(r"D:\My Files\"));
        assert_eq!(args.len(), 4);
        assert_eq!(args[2], "--cwd");
        assert_eq!(args[3], r"D:\My Files\");
    }

    #[test]
    fn candidates_are_ordered_by_preference() {
        let found = candidates(lookup);
        assert_eq!(
            found[0],
            PathBuf::from(r"C:\Program Files\WezTerm\wezterm-gui.exe")
        );
        assert_eq!(
            found[1],
            PathBuf::from(r"C:\Users\u\AppData\Local\Programs\WezTerm\wezterm-gui.exe")
        );
        assert_eq!(found[2], PathBuf::from(r"C:\a\wezterm-gui.exe"));
        assert_eq!(found.len(), 4);
    }

    #[test]
    fn override_comes_first() {
        let found = candidates(|name| {
            if name == OVERRIDE_ENV {
                Some(OsString::from(r"X:\wez.exe"))
            } else {
                lookup(name)
            }
        });
        assert_eq!(found[0], PathBuf::from(r"X:\wez.exe"));
    }
}
