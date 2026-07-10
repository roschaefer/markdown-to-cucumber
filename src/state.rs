use crate::paths::resolve_safe;
use std::path::{Path, PathBuf};

/// Accessor trait a tool's own `cucumber::World` type implements so the
/// shared step functions in [`crate::steps`] can operate on it generically.
///
/// Only the handful of fields every tool's e2e suite needs are here: a work
/// directory to sandbox scenario file I/O in, the last command's captured
/// output, and enough to dispatch `I run "..."` to the right compiled test
/// binary.
pub trait CliState: Sized {
    /// The scenario's sandboxed working directory (usually a `TempDir`).
    fn work_dir(&self) -> &Path;

    /// Resolves a scenario-relative path against [`CliState::work_dir`].
    /// Override only if a tool needs different safety rules than
    /// [`resolve_safe`]'s reject-absolute/reject-`..` default.
    fn resolve(&self, path: &str) -> PathBuf {
        resolve_safe(self.work_dir(), path)
    }

    fn last_stdout(&self) -> &str;
    fn last_stderr(&self) -> &str;
    fn last_exit_code(&self) -> i32;

    /// Records the outcome of the most recently run command.
    fn set_last_run(&mut self, stdout: String, stderr: String, exit_code: i32);

    /// Absolute path to this tool's compiled test binary, e.g.
    /// `PathBuf::from(env!("CARGO_BIN_EXE_hledger-bank-import"))`.
    fn binary_path(&self) -> &Path;

    /// Token sequences that, when found as a prefix of a scenario's `I run
    /// "..."` command, get replaced by [`CliState::binary_path`] — e.g.
    /// `[["hledger", "bank-import"], ["hledger-bank-import"]]` so a spec can
    /// write either invocation style.
    fn invocation_prefixes(&self) -> &[Vec<String>];

    /// Substitutes tool-specific placeholders (e.g. `{work_dir}`) into
    /// expected text before comparing it against captured output. Most
    /// tools don't need this; the default is the identity function.
    fn interpolate(&self, text: &str) -> String {
        text.to_string()
    }
}
