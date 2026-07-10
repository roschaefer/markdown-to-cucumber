# AGENTS.md

`markdown-to-cucumber` lets you write Markdown documentation with fenced
```gherkin code blocks and execute those blocks as
[Cucumber](https://cucumber.io) scenarios at test time — the documentation
*is* the test suite, so it can't drift from the behavior it describes. It
also ships a small library of generic step implementations for testing
command-line tools (write a file, run the binary, assert on
stdout/stderr/exit code/output files), so a consumer doesn't have to
reimplement that plumbing from scratch.

This crate is not specific to any particular tool — it's a general-purpose
"literate executable documentation" library, currently used as a dev-only
dependency by the `hledger-*` CLI tools in this workspace (`hledger-elster`,
`hledger-document-check`, `hledger-bank-import`, `hledger-journal-check`),
pulled in via a `git` dependency rather than a submodule.

- `src/specs.rs`: `generate_features(specs_dir, out_dir)` — extracts fenced ```gherkin/```feature blocks from `specs/*.md` into `.feature` files at build time. Call this as a one-liner from a consuming crate's `build.rs`.
- `src/state.rs`: the `CliState` trait — accessor methods (`work_dir`, `last_stdout`/`last_stderr`/`last_exit_code`, `binary_path`, `invocation_prefixes`, optional `interpolate`) a consumer's own `cucumber::World` type implements so the shared CLI-testing steps in `src/steps.rs` can operate on it generically.
- `src/steps.rs`: the shared CLI-testing step implementations, plus a `_regex()` function per step. Because `#[given]`/`#[when]`/`#[then]` attribute macros register steps per-concrete-`World`-type via `inventory` (they can't be shared across crates), these are written against cucumber's lower-level `fn(&mut W, Context) -> LocalBoxFuture<'_, ()>` step signature and wired in manually via `Cucumber::given/when/then`, generic over any `W: World + CliState`.
- `src/text.rs`: `docstring()`, `gherkin_table()`, `contains_in_order()` (the `...`-wildcard matcher) — pure helpers operating on `gherkin::Step`/plain text.
- `src/paths.rs`: `resolve_safe()` — scenario-relative path resolution that rejects absolute paths and `..` components.

## Using this from a consumer

A consumer's `build.rs`:

```rust
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    markdown_to_cucumber::specs::generate_features(std::path::Path::new("specs"), std::path::Path::new(&out_dir));
}
```

A consumer's `tests/cucumber.rs`: declare a `World` struct with only the
tool's genuinely extra fields, `impl CliState for it`, keep any tool-specific
`#[given]`/`#[when]`/`#[then]` steps locally, and compose the shared ones in
`main()`:

```rust
MyWorld::cucumber()
    .given(markdown_to_cucumber::steps::write_file_regex(), markdown_to_cucumber::steps::write_file)
    .when(markdown_to_cucumber::steps::run_command_regex(), markdown_to_cucumber::steps::run_command)
    .then(markdown_to_cucumber::steps::stdout_contains_regex(), markdown_to_cucumber::steps::stdout_contains)
    // ... only the shared steps this tool's specs actually use
    .run_and_exit(features_dir)
    .await;
```

## Boundaries

- Keep this crate's steps generic and consumer-agnostic — no `hledger`-
  specific assumptions belong here, since it's a general CLI-testing/
  literate-docs library, not an hledger tool. A step that only makes sense
  for one consumer (PDF generation, background-process management, git
  checkout simulation, etc.) belongs in that consumer's own
  `tests/cucumber.rs`, not here.
- Changing behavior here affects every consumer's test suite. When in doubt,
  prefer adding a new step/regex over changing an existing one's matching
  behavior.
