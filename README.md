# dir_bin

A fast, single-binary per-directory environment manager for zsh. When you `cd` into a
directory, `dir_bin` prepends that directory's matching bin folder to `$PATH` and loads
any per-directory keybindings — then cleans them up when you leave.

It works like `zoxide`/`direnv`/`starship`: the binary prints shell commands to stdout and
a tiny shim `eval`s them. This replaces the previous pure-zsh plugin (`path_manager.zsh`)
to cut the per-`cd` latency.

## Layout

`dir_bin` looks for a folder under its root whose name matches the **basename** of your
current directory:

```
$DIR_BIN_ROOT/                 # default: $HOME/bin/dir_bin
  lexabyte/                    # matches any dir basename "lexabyte"
    db                         # executable added to $PATH while inside lexabyte
    __bindings.zsh             # optional: sourced with `set` on enter, `unset` on leave
  entai-cdn/
    ...
```

- **PATH:** while you're in a matching directory, `<root>/<basename>` is prepended to
  `$PATH`. Stale `dir_bin` entries are stripped on every change, so PATH never accumulates.
- **Bindings:** an optional `__bindings.zsh` is sourced with argument `set` when you enter
  and `unset` when you leave (used to register zle widgets / `bindkey`). These must run in
  the interactive shell, so the binary emits `source ... set|unset` lines for the shim to run.
- `dir_bin` only acts when you're under `$HOME` and not inside the root directory itself.

## Install via Homebrew

```sh
brew tap statbit/dir_bin
brew install dir_bin
```

`brew tap statbit/dir_bin` clones `github.com/statbit/homebrew-dir_bin` (Homebrew adds the
mandatory `homebrew-` prefix automatically). Once tapped, `brew install dir_bin` resolves
the formula; you can also use the fully qualified name `brew install statbit/dir_bin/dir_bin`.

> Note: `brew install statbit/dir_bin` (without tapping first) does **not** work — that
> two-part form is a *tap* reference, not a formula.

### Publishing a release (maintainers)

The Homebrew formula lives in a **separate tap repo**, `statbit/homebrew-dir_bin`
(at `Formula/dir_bin.rb`), because that's what `brew tap statbit/dir_bin` clones and
installs from. This code repo holds only the source.

To cut a release:

1. Tag this repo: `git tag v0.1.0 && git push origin v0.1.0`. GitHub then auto-generates
   the source tarball at
   `https://github.com/statbit/dir_bin/archive/refs/tags/v0.1.0.tar.gz`.
2. From a clone of the **tap repo**, bump the formula in one step:
   ```sh
   brew bump-formula-pr \
     --url=https://github.com/statbit/dir_bin/archive/refs/tags/v0.1.0.tar.gz \
     dir_bin
   ```
   This downloads the tarball, computes the `sha256`, edits `url` + `sha256`, and opens a
   PR against the tap. (Manual alternative: `curl -sL <tarball-url> | shasum -a 256`, then
   edit `url` and `sha256` in the tap's `Formula/dir_bin.rb` and push.)

## Build from source

```sh
cargo build --release
```

Produces a single self-contained binary at `target/release/dir_bin`. Put it on your `$PATH`:

```sh
cp target/release/dir_bin ~/bin/      # or anywhere on PATH
```

## Install (zsh)

Add to `~/.zshrc`:

```sh
eval "$(dir_bin init)"
```

That installs a `chpwd` hook plus a `refresh-dir-bin` function (re-runs the hook for the
current directory if startup ever misses).

## Configuration

The root directory is resolved in this order (first match wins):

1. `DIR_BIN_ROOT` environment variable
2. `root` key in `$XDG_CONFIG_HOME/dir_bin/config.toml` (falls back to
   `$HOME/.config/dir_bin/config.toml`)
3. Default: `$HOME/bin/dir_bin`

Example `~/.config/dir_bin/config.toml`:

```toml
# Path to your dir_bin root. A leading ~ is expanded to $HOME.
root = "~/bin/dir_bin"
```

## Commands

- `dir_bin init` — print the zsh shim for `~/.zshrc`.
- `dir_bin hook <pwd> <oldpwd>` — print the shell commands for a directory change (called
  by the shim; you normally don't run this by hand).
