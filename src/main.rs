use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const BINDINGS_FILE: &str = "__bindings.zsh";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("init") => {
            print!("{}", init_script());
            ExitCode::SUCCESS
        }
        Some("hook") => {
            let pwd = args.get(1).map(String::as_str).unwrap_or("");
            let oldpwd = args.get(2).map(String::as_str).unwrap_or("");
            print!("{}", hook(pwd, oldpwd));
            ExitCode::SUCCESS
        }
        Some("-h") | Some("--help") | Some("help") => {
            print!("{}", help_text());
            ExitCode::SUCCESS
        }
        _ => {
            eprint!("{}", help_text());
            ExitCode::FAILURE
        }
    }
}

fn help_text() -> String {
    "\
dir_bin - fast per-directory PATH and keybinding manager for zsh

USAGE:
    dir_bin <COMMAND>

COMMANDS:
    init                 print the zsh shim to add to ~/.zshrc
    hook <pwd> <oldpwd>  print shell commands for a directory change (used by the shim)
    help, -h, --help     show this help

SETUP:
    Add this line to your ~/.zshrc, then restart your shell:

        eval \"$(dir_bin init)\"

    This installs a chpwd hook that adjusts PATH and keybindings as you cd,
    plus a `refresh-dir-bin` function to re-run it for the current directory.

CONFIGURATION:
    Root directory is resolved in this order (first match wins):
        1. $DIR_BIN_ROOT environment variable
        2. `root` key in $HOME/.config/dir_bin/config.toml
        3. default: $HOME/bin/dir_bin
"
    .to_string()
}

/// Resolve the dir_bin root, in order of precedence:
///   1. $DIR_BIN_ROOT env var
///   2. `root` key in $HOME/.config/dir_bin/config.toml
///   3. default $HOME/bin/dir_bin
fn root() -> Option<PathBuf> {
    if let Some(r) = env::var_os("DIR_BIN_ROOT") {
        if !r.is_empty() {
            return Some(PathBuf::from(r));
        }
    }
    if let Some(r) = config_root() {
        return Some(r);
    }
    env::var_os("HOME").map(|h| Path::new(&h).join("bin").join("dir_bin"))
}

/// Path to the config file: $XDG_CONFIG_HOME/dir_bin/config.toml,
/// falling back to $HOME/.config/dir_bin/config.toml.
fn config_path() -> Option<PathBuf> {
    let base = match env::var_os("XDG_CONFIG_HOME") {
        Some(x) if !x.is_empty() => PathBuf::from(x),
        _ => Path::new(&env::var_os("HOME")?).join(".config"),
    };
    Some(base.join("dir_bin").join("config.toml"))
}

/// Read the `root` key from the config file, expanding a leading `~`.
fn config_root() -> Option<PathBuf> {
    let path = config_path()?;
    let contents = fs::read_to_string(&path).ok()?;
    let raw = parse_toml_string(&contents, "root")?;
    Some(expand_tilde(&raw))
}

/// Expand a leading `~` or `~/` to $HOME.
fn expand_tilde(s: &str) -> PathBuf {
    if let Some(home) = env::var_os("HOME") {
        if s == "~" {
            return PathBuf::from(home);
        }
        if let Some(rest) = s.strip_prefix("~/") {
            return Path::new(&home).join(rest);
        }
    }
    PathBuf::from(s)
}

/// Minimal TOML reader for a single top-level `key = "value"` string entry.
/// Ignores comments, blank lines, and any `[section]` headers. Good enough for
/// our one setting without pulling in a TOML dependency.
fn parse_toml_string(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        if k.trim() != key {
            continue;
        }
        // Strip an inline comment that follows the value (outside quotes is the
        // common case; we only support simple quoted/bare values).
        let mut v = v.trim();
        if let Some(stripped) = v.strip_prefix('"') {
            if let Some(end) = stripped.find('"') {
                return Some(stripped[..end].to_string());
            }
        } else if let Some(stripped) = v.strip_prefix('\'') {
            if let Some(end) = stripped.find('\'') {
                return Some(stripped[..end].to_string());
            }
        } else {
            if let Some(idx) = v.find('#') {
                v = v[..idx].trim();
            }
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// The zsh shim emitted by `dir_bin init`.
fn init_script() -> String {
    let exe = env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "dir_bin".to_string());
    let exe = shell_quote(&exe);
    format!(
        "_dir_bin_hook() {{\n  \
           eval \"$({exe} hook \"$PWD\" \"$OLDPWD\")\"\n\
         }}\n\
         autoload -U add-zsh-hook\n\
         add-zsh-hook chpwd _dir_bin_hook\n\
         refresh-dir-bin() {{ _dir_bin_hook }}\n\
         _dir_bin_hook\n"
    )
}

/// Compute the shell commands to emit for a directory change.
fn hook(pwd: &str, oldpwd: &str) -> String {
    let Some(root) = root() else {
        return String::new();
    };
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return String::new();
    };

    let pwd_path = Path::new(pwd);

    // Only act when under $HOME and not inside the root dir itself.
    if !pwd_path.starts_with(&home) || pwd_path.starts_with(&root) {
        return String::new();
    }

    let mut out = String::new();

    // Rebuild PATH: strip existing dir_bin entries, then prepend the current dir's bin.
    let new_path = rebuild_path(&root, pwd_path);
    out.push_str(&format!("export PATH={}\n", shell_quote(&new_path)));

    // Unset previous dir's bindings, then set the current dir's bindings.
    if let Some(prev) = basename(oldpwd) {
        let f = root.join(prev).join(BINDINGS_FILE);
        if f.is_file() {
            out.push_str(&format!("source {} unset\n", shell_quote(&path_str(&f))));
        }
    }
    if let Some(cur) = basename(pwd) {
        let f = root.join(cur).join(BINDINGS_FILE);
        if f.is_file() {
            out.push_str(&format!("source {} set\n", shell_quote(&path_str(&f))));
        }
    }

    out
}

/// Strip any PATH entries living under `root`, then prepend `<root>/<basename(pwd)>`
/// if that directory exists.
fn rebuild_path(root: &Path, pwd: &Path) -> String {
    let current = env::var("PATH").unwrap_or_default();
    let root_prefix = format!("{}/", path_str(root));

    let mut entries: Vec<&str> = current
        .split(':')
        .filter(|e| !e.is_empty() && !e.starts_with(&root_prefix) && *e != path_str(root))
        .collect();

    let bin_dir = basename_path(pwd).map(|b| root.join(b));
    let bin_str = bin_dir.as_ref().map(|p| path_str(p));
    if let Some(ref b) = bin_str {
        if Path::new(b).is_dir() {
            entries.insert(0, b);
        }
    }

    entries.join(":")
}

fn basename(p: &str) -> Option<String> {
    basename_path(Path::new(p)).map(|s| s.to_string())
}

fn basename_path(p: &Path) -> Option<&str> {
    p.file_name().and_then(|n| n.to_str())
}

fn path_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// Single-quote a string for safe use in shell, escaping embedded single quotes.
fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}
