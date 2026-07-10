# markdown-to-cucumber

Write Markdown documentation with embedded [Cucumber](https://cucumber.io)
scenarios in fenced ```gherkin code blocks, and execute them as real tests.
The documentation *is* the test suite — nothing generated is committed to
git, so the prose and the behavior it describes cannot drift apart.

```markdown
# Greeting

Running the tool with no arguments prints a friendly greeting.

​```gherkin
Feature: Greeting

  Scenario: No arguments
    When I run "my-tool"
    Then stdout should contain:
      """
      Hello!
      """
​```
```

`markdown-to-cucumber` also ships a small, generic step library for testing
command-line tools — write a file, run the binary, assert on stdout/stderr/
exit code/output files — so most consumers don't need to write any step
definitions of their own beyond the truly tool-specific ones.

## Usage

In `build.rs`:

```rust
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    markdown_to_cucumber::specs::generate_features(std::path::Path::new("specs"), std::path::Path::new(&out_dir));
}
```

In `tests/cucumber.rs`:

```rust
use cucumber::World;
use markdown_to_cucumber::CliState;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug, cucumber::World)]
#[world(init = MyWorld::new)]
struct MyWorld {
    _tmp: TempDir,
    work_dir: PathBuf,
    last_stdout: String,
    last_stderr: String,
    last_exit_code: i32,
}

impl MyWorld {
    async fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let work_dir = tmp.path().to_path_buf();
        Self { _tmp: tmp, work_dir, last_stdout: String::new(), last_stderr: String::new(), last_exit_code: 0 }
    }
}

impl CliState for MyWorld {
    fn work_dir(&self) -> &Path { &self.work_dir }
    fn last_stdout(&self) -> &str { &self.last_stdout }
    fn last_stderr(&self) -> &str { &self.last_stderr }
    fn last_exit_code(&self) -> i32 { self.last_exit_code }
    fn set_last_run(&mut self, stdout: String, stderr: String, exit_code: i32) {
        self.last_stdout = stdout;
        self.last_stderr = stderr;
        self.last_exit_code = exit_code;
    }
    fn binary_path(&self) -> &Path {
        static BIN: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        BIN.get_or_init(|| PathBuf::from(env!("CARGO_BIN_EXE_my-tool")))
    }
    fn invocation_prefixes(&self) -> &[Vec<String>] {
        static PREFIXES: std::sync::OnceLock<Vec<Vec<String>>> = std::sync::OnceLock::new();
        PREFIXES.get_or_init(|| vec![vec!["my-tool".to_string()]])
    }
}

#[tokio::main]
async fn main() {
    let features = format!("{}/features", env!("OUT_DIR"));
    MyWorld::cucumber()
        .given(markdown_to_cucumber::steps::write_file_regex(), markdown_to_cucumber::steps::write_file)
        .when(markdown_to_cucumber::steps::run_command_regex(), markdown_to_cucumber::steps::run_command)
        .then(markdown_to_cucumber::steps::stdout_contains_regex(), markdown_to_cucumber::steps::stdout_contains)
        .run_and_exit(features)
        .await;
}
```

See `AGENTS.md` for the full list of available shared steps and the design
rationale (why steps are wired manually instead of via `#[given]`/etc.).

## Commands

- `just build`
- `just test`
- `just check`
