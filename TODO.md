# TODO

## Precompiled binaries via Homebrew bottles (future work)

Currently the Homebrew formula (`Formula/dir_bin.rb` in the `statbit/homebrew-dir_bin` tap
repo) builds **from source** on each
user's machine: brew downloads the GitHub source tarball, verifies the `sha256`, ensures a
Rust toolchain (`depends_on "rust" => :build`), and runs `cargo install`. This is fine for
a small zero-dependency crate, but if compile time ever matters we can ship prebuilt
binaries instead.

Homebrew's mechanism for prebuilt binaries is **bottles**. Two options:

### Option A — homebrew-core (managed bottles)
Submit the formula to the official `homebrew/core` tap. If accepted, BrewTestBot builds and
hosts bottles automatically across platforms. Downside: submission/review process and
ongoing core-tap policy constraints. Likely overkill for this project.

### Option B — self-hosted bottles in our own tap (recommended if we bottle)
Build and host bottles ourselves from the `statbit/homebrew-dir_bin` tap.

Rough steps:
1. Add a GitHub Actions workflow that, on a new version tag, builds the formula on each
   target platform (at minimum macOS arm64 and x86_64; add Linux if desired).
2. Run `brew install --build-bottle dir_bin` then `brew bottle dir_bin` to produce the
   bottle tarball(s) and their sha256 lines.
3. Upload the bottle tarballs as assets on the GitHub release.
4. Add a `bottle do ... end` block to the tap's `Formula/dir_bin.rb` with `root_url` (pointing at the
   release assets) and the per-platform `sha256` entries. `brew bump-formula-pr` /
   `brew bottle --merge` can help regenerate this block.

Notes / gotchas:
- Bottles are platform-specific — one tarball per OS/arch combo. Need CI runners (or
  cross-compilation) for each target we want to support.
- Keep the build-from-source path working as a fallback for unbottled platforms; brew
  falls back to source automatically when no matching bottle exists.
- The `root_url` must match wherever the assets are actually hosted (GitHub release URL or
  a custom bucket).

### Decision
Sticking with build-from-source for now (2026-05-30). Revisit bottling only if local
compile time becomes a real pain point.
