# markdown-to-cucumber

Extracts fenced [Cucumber](https://cucumber.io) (```gherkin) code blocks from
Markdown files into `.feature` files at build time, so a crate can write its
behavior specification as ordinary Markdown — prose plus runnable
scenarios — and have it double as its test suite. Nothing generated is
committed to git, so the documentation and the tests it describes cannot
drift apart.

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

That's the entire scope: Markdown in, `.feature` files out. Step
definitions, `World` state, and everything else about how those scenarios
actually get executed is up to you, in your own `tests/cucumber.rs` — this
crate has no opinion on that and doesn't even depend on `cucumber`.

## Usage

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

Then in `tests/cucumber.rs`, point your `World::run` (or `Cucumber::run`) at
`format!("{}/features", env!("OUT_DIR"))` and write your own `World` and step
definitions as you normally would with `cucumber`.

The feature below is part of this README and is exercised by this repository's
own test suite. It writes a temporary consumer crate and runs that crate's
Cucumber test, proving that the documented integration works.

```gherkin
Feature: Using markdown-to-cucumber from a consumer crate

  Scenario: A consumer crate runs Cucumber scenarios generated from Markdown
    Given a new Rust crate
    And the file "Cargo.toml" contains:
      """
      [package]
      name = "markdown-to-cucumber-readme-example"
      version = "0.1.0"
      edition = "2021"
      publish = false

      [build-dependencies]
      markdown-to-cucumber = { path = "{repository}" }

      [dev-dependencies]
      cucumber = "0.23"
      tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

      [[test]]
      name = "cucumber"
      harness = false
      """
    And the file "build.rs" contains:
      """
      fn main() {
          let out_dir = std::env::var("OUT_DIR").unwrap();
          markdown_to_cucumber::specs::generate_features(
              std::path::Path::new("specs"),
              std::path::Path::new(&out_dir),
          );
      }
      """
    And the file "src/lib.rs" contains:
      """
      pub fn greeting(name: &str) -> String {
          format!("Hello, {name}!")
      }
      """
    And the Markdown spec file "specs/greeting.md" contains this Gherkin:
      """
      Feature: Greeting

        Scenario: Greeting a person by name
          When I greet "Ada"
          Then the greeting should be "Hello, Ada!"
      """
    And the file "tests/cucumber.rs" contains:
      """
      use cucumber::{then, when, World as _};

      #[derive(Debug, Default, cucumber::World)]
      struct ExampleWorld {
          greeting: Option<String>,
      }

      #[when(regex = r#"^I greet "([^"]+)"$"#)]
      fn greet(world: &mut ExampleWorld, name: String) {
          world.greeting = Some(markdown_to_cucumber_readme_example::greeting(&name));
      }

      #[then(regex = r#"^the greeting should be "([^"]+)"$"#)]
      fn assert_greeting(world: &mut ExampleWorld, expected: String) {
          assert_eq!(world.greeting.as_deref(), Some(expected.as_str()));
      }

      #[tokio::main]
      async fn main() {
          ExampleWorld::run(format!("{}/features", env!("OUT_DIR"))).await;
      }
      """
    When I run "cargo test"
    Then the command succeeds
    And stdout contains "1 scenario (1 passed)"
```

## Commands

- `just test`
- `just check`
