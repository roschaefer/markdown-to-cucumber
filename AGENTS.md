# AGENTS.md

`markdown-to-cucumber` extracts fenced ```gherkin (or ```feature) code blocks
from Markdown files into `.feature` files at build time, so a crate can write
its behavior specification as ordinary Markdown documentation — prose plus
runnable [Cucumber](https://cucumber.io) scenarios — and have it double as
its test suite. Nothing generated is committed to git, so the docs and the
tests they describe cannot drift apart.

This is the entire scope of the library API: **Markdown → `.feature` files.**
Cucumber step definitions, `World` state, and any other test-runner glue
belong in the consuming crate's own `tests/cucumber.rs` — a reader of a spec
should be able to find what a step actually does without leaving that repo.
This crate must not export shared step definitions, a shared `World`, or
runtime glue for consumers. Test-only Cucumber glue in this repository is fine
when it verifies this crate's own documentation or behavior.

Used as a dev-only (and build-only) dependency by the `hledger-*` CLI tools
in this workspace, pulled in via a `git` dependency rather than a submodule.

- `src/specs.rs`: `generate_features(specs_dir, out_dir)` — the whole crate. Call it as a one-liner from a consuming crate's `build.rs`.

## Using this from a consumer

`Cargo.toml`:

```toml
[build-dependencies]
markdown-to-cucumber = { git = "https://github.com/roschaefer/markdown-to-cucumber.git", branch = "main" }
```

`build.rs`:

```rust
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    markdown_to_cucumber::specs::generate_features(std::path::Path::new("specs"), std::path::Path::new(&out_dir));
}
```

`tests/cucumber.rs` then points its `World::run` at `format!("{}/features", env!("OUT_DIR"))`
and defines its own `World` and step functions as usual — this crate is not involved from there on.

## Boundaries

- Do not add exported step definitions, a reusable `World` type, or consumer
  cucumber-runtime code to the library. If a future need for shared
  CLI-testing helpers comes up, that's a separate, explicitly-scoped library —
  not a reason to grow this one.
- Test-only step definitions under `tests/` are allowed for this repository's
  own specs and documentation checks.
