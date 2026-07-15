use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use cucumber::{gherkin::Step, given, then, when, World as _};
use tempfile::TempDir;

#[derive(Debug, cucumber::World)]
#[world(init = Self::new)]
struct ReadmeWorld {
    crate_dir: TempDir,
    last_output: Option<std::process::Output>,
}

impl ReadmeWorld {
    fn new() -> Self {
        Self {
            crate_dir: tempfile::tempdir().unwrap(),
            last_output: None,
        }
    }
}

#[given("a new Rust crate")]
fn new_rust_crate(world: &mut ReadmeWorld) {
    fs::create_dir_all(world.crate_dir.path()).unwrap();
}

#[given(regex = r#"^the file "([^"]+)" contains:$"#)]
fn write_file(world: &mut ReadmeWorld, #[step] step: &Step, relative_path: String) {
    let path = world.crate_dir.path().join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let contents = step
        .docstring()
        .expect("file content step must have a doc string")
        .replace("{repository}", env!("CARGO_MANIFEST_DIR"));
    fs::write(path, contents).unwrap();
}

#[given(regex = r#"^the Markdown spec file "([^"]+)" contains this Gherkin:$"#)]
fn write_markdown_spec(world: &mut ReadmeWorld, #[step] step: &Step, relative_path: String) {
    let path = world.crate_dir.path().join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let gherkin = step
        .docstring()
        .expect("Markdown spec step must have a doc string");
    fs::write(
        path,
        format!("# Generated example\n\n```gherkin\n{gherkin}\n```\n"),
    )
    .unwrap();
}

#[when(regex = r#"^I run "([^"]+)"$"#)]
fn run_command(world: &mut ReadmeWorld, command: String) {
    let mut parts = command.split_whitespace();
    let program = parts.next().expect("command must include a program");

    let output = Command::new(program)
        .args(parts)
        .current_dir(world.crate_dir.path())
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .unwrap();
    world.last_output = Some(output);
}

#[then("the command succeeds")]
fn command_succeeds(world: &mut ReadmeWorld) {
    let output = world.last_output.as_ref().expect("no command has run");

    assert!(
        output.status.success(),
        "command failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[then(regex = r#"^stdout contains "([^"]+)"$"#)]
fn stdout_contains(world: &mut ReadmeWorld, expected: String) {
    let output = world.last_output.as_ref().expect("no command has run");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(&expected),
        "stdout did not contain {expected:?}\nstdout:\n{stdout}",
    );
}

#[tokio::main]
async fn main() {
    let readme_specs_dir = tempfile::tempdir().unwrap();
    fs::copy("README.md", readme_specs_dir.path().join("README.md")).unwrap();

    let generated_dir = tempfile::tempdir().unwrap();
    markdown_to_cucumber::specs::generate_features(readme_specs_dir.path(), generated_dir.path());

    ReadmeWorld::run(features_dir(generated_dir.path())).await;
}

fn features_dir(out_dir: &Path) -> PathBuf {
    out_dir.join("features")
}
