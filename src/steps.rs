//! Generic step implementations, shared across every tool's `tests/cucumber.rs`.
//!
//! These are written against the low-level [`cucumber::step::Step`] function
//! signature (`fn(&mut W, Context) -> LocalBoxFuture<'_, ()>`) rather than
//! the `#[given]`/`#[when]`/`#[then]` attribute macros, because those macros
//! register steps per concrete `World` type via `inventory` and can't be
//! shared across crates. Wire these into a tool's own `main()` via
//! `MyWorld::cucumber().given(markdown_to_cucumber::steps::write_file_regex(), markdown_to_cucumber::steps::write_file)`
//! (and so on) — `Cucumber::given`/`when`/`then` accept any function
//! matching the step signature, including a monomorphized generic one.

use crate::state::CliState;
use crate::text::{contains_in_order, docstring, gherkin_table};
use cucumber::{step::Context, World};
use futures::future::LocalBoxFuture;
use regex::Regex;
use std::fmt::Debug;
use std::path::Path;

fn capture(ctx: &Context, index: usize) -> String {
    ctx.matches[index].1.clone()
}

// ---------------------------------------------------------------------------
// Given
// ---------------------------------------------------------------------------

pub fn write_file_regex() -> Regex {
    Regex::new(r#"^a file named "([^"]+)" with content:$"#).unwrap()
}

pub fn write_file<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let path = capture(&ctx, 1);
        let content = docstring(&ctx.step);
        let target = world.resolve(&path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, format!("{content}\n")).unwrap();
    })
}

pub fn write_files_table_regex() -> Regex {
    Regex::new(r"^files named:$").unwrap()
}

/// A table with `path` and `content` columns; `\n` in a cell is a literal
/// newline (data tables can't hold real newlines).
pub fn write_files_table<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let table = gherkin_table(&ctx.step);
        let headers = &table[0];
        let path_col = headers
            .iter()
            .position(|h| h == "path")
            .expect("table needs a path column");
        let content_col = headers
            .iter()
            .position(|h| h == "content")
            .expect("table needs a content column");
        for row in table.iter().skip(1) {
            let path = &row[path_col];
            let content = row[content_col].replace("\\n", "\n");
            let target = world.resolve(path);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(&target, format!("{content}\n")).unwrap();
        }
    })
}

pub fn empty_directory_regex() -> Regex {
    Regex::new(r#"^an empty directory named "([^"]+)"$"#).unwrap()
}

pub fn empty_directory<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let path = capture(&ctx, 1);
        std::fs::create_dir_all(world.resolve(&path)).unwrap();
    })
}

// ---------------------------------------------------------------------------
// When
// ---------------------------------------------------------------------------

pub fn run_command_regex() -> Regex {
    Regex::new(r#"^I run "([^"]+)"$"#).unwrap()
}

fn strip_known_prefix<'a>(args: &'a [String], prefixes: &[Vec<String>]) -> Option<&'a [String]> {
    for prefix in prefixes {
        if args.len() >= prefix.len() && args[..prefix.len()] == prefix[..] {
            return Some(&args[prefix.len()..]);
        }
    }
    None
}

/// Runs the tool's compiled test binary, dispatching `hledger <sub>` or
/// `hledger-<bin>`-style invocations (per [`CliState::invocation_prefixes`])
/// to it. Captures stdout/stderr/exit code into the world via
/// [`CliState::set_last_run`] but does **not** assert success — a scenario
/// that expects failure follows up with an explicit exit-code/stderr step.
pub fn run_command<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let command = capture(&ctx, 1);
        let args: Vec<String> = command.split_whitespace().map(String::from).collect();
        let rest = strip_known_prefix(&args, world.invocation_prefixes())
            .unwrap_or_else(|| panic!("Unsupported command: {command}"))
            .to_vec();

        let output = tokio::process::Command::new(world.binary_path())
            .args(&rest)
            .current_dir(world.work_dir())
            .output()
            .await
            .expect("failed to run command");

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let code = output.status.code().unwrap_or(-1);
        world.set_last_run(stdout, stderr, code);
    })
}

// ---------------------------------------------------------------------------
// Then
// ---------------------------------------------------------------------------

pub fn exit_code_equals_regex() -> Regex {
    Regex::new(r"^(?:the command exits with code|the exit code is) (\d+)$").unwrap()
}

pub fn exit_code_equals<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let expected: i32 = capture(&ctx, 1).parse().unwrap();
        assert_eq!(
            world.last_exit_code(),
            expected,
            "exit code mismatch\nstdout:\n{}\nstderr:\n{}",
            world.last_stdout(),
            world.last_stderr(),
        );
    })
}

pub fn stdout_contains_regex() -> Regex {
    Regex::new(r"^stdout (?:should )?contains?:$").unwrap()
}

pub fn stdout_contains<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let expected = world.interpolate(&docstring(&ctx.step));
        assert!(
            contains_in_order(world.last_stdout(), &expected),
            "stdout did not contain:\n{expected}\n\nActual stdout:\n{}",
            world.last_stdout(),
        );
    })
}

pub fn stderr_contains_regex() -> Regex {
    Regex::new(r"^stderr (?:should )?contains?:$").unwrap()
}

pub fn stderr_contains<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let expected = world.interpolate(&docstring(&ctx.step));
        assert!(
            contains_in_order(world.last_stderr(), &expected),
            "stderr did not contain:\n{expected}\n\nActual stderr:\n{}",
            world.last_stderr(),
        );
    })
}

pub fn stdout_does_not_contain_regex() -> Regex {
    Regex::new(r"^stdout (?:should not|does not) contains?:$").unwrap()
}

pub fn stdout_does_not_contain<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let unexpected = world.interpolate(&docstring(&ctx.step));
        assert!(
            !world.last_stdout().contains(unexpected.as_str()),
            "stdout contained unexpected text:\n{unexpected}\n\nActual stdout:\n{}",
            world.last_stdout(),
        );
    })
}

pub fn stdout_equals_regex() -> Regex {
    Regex::new(r"^stdout equals:$").unwrap()
}

pub fn see_this_output_regex() -> Regex {
    Regex::new(r"^I see this output:$").unwrap()
}

pub fn stdout_equals<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let expected = world.interpolate(&format!("{}\n", docstring(&ctx.step)));
        assert_eq!(
            world.last_stdout(),
            expected,
            "stdout did not equal expected\nExpected:\n{expected}\nActual:\n{}",
            world.last_stdout(),
        );
    })
}

pub fn file_contains_exactly_regex() -> Regex {
    Regex::new(r#"^the file "([^"]+)" (?:should )?contains? exactly:$"#).unwrap()
}

pub fn file_contains_exactly<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let path = capture(&ctx, 1);
        let expected = world.interpolate(&format!("{}\n", docstring(&ctx.step)));
        let actual_path = world.resolve(&path);
        let actual = std::fs::read_to_string(&actual_path)
            .unwrap_or_else(|_| panic!("file not found: {path}"));
        assert_eq!(
            actual, expected,
            "file {path} did not equal expected\nExpected:\n{expected}\nActual:\n{actual}",
        );
    })
}

pub fn file_contains_regex() -> Regex {
    Regex::new(r#"^the file "([^"]+)" contains:$"#).unwrap()
}

pub fn file_contains<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let path = capture(&ctx, 1);
        let expected = world.interpolate(&docstring(&ctx.step));
        let actual_path = world.resolve(&path);
        let actual = std::fs::read_to_string(&actual_path)
            .unwrap_or_else(|_| panic!("file not found: {path}"));
        assert!(
            contains_in_order(&actual, &expected),
            "file {path} did not contain:\n{expected}\n\nActual:\n{actual}",
        );
    })
}

pub fn file_exists_regex() -> Regex {
    Regex::new(r#"^the file "([^"]+)" exists$"#).unwrap()
}

pub fn file_exists<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let path = capture(&ctx, 1);
        assert!(
            world.resolve(&path).is_file(),
            "expected file to exist: {path}"
        );
    })
}

pub fn file_does_not_exist_regex() -> Regex {
    Regex::new(r#"^the file "([^"]+)" does not exist$"#).unwrap()
}

pub fn file_does_not_exist<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let path = capture(&ctx, 1);
        assert!(
            !world.resolve(&path).exists(),
            "expected file not to exist: {path}"
        );
    })
}

pub fn csv_file_contains_exactly_regex() -> Regex {
    Regex::new(r#"^the CSV file "([^"]+)" should contain exactly:$"#).unwrap()
}

pub fn csv_file_contains_exactly<W: World + CliState + Debug>(
    world: &mut W,
    ctx: Context,
) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let path = capture(&ctx, 1);
        let actual_path = world.resolve(&path);
        assert!(
            actual_path.exists(),
            "Expected output file was not created: {path}"
        );

        let actual = read_csv(&actual_path);
        assert_eq!(actual, gherkin_table(&ctx.step));
    })
}

fn read_csv(path: &Path) -> Vec<Vec<String>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .unwrap();
    reader
        .records()
        .map(|r| r.unwrap().iter().map(str::to_string).collect())
        .collect()
}
