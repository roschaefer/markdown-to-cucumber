use std::{fs, path::Path};

/// Extracts fenced ```gherkin (or ```feature) blocks from every `*.md` file
/// in `specs_dir` and writes each one as a `.feature` file under
/// `out_dir/features/`. Intended to be called from a consuming crate's
/// `build.rs`:
///
/// ```no_run
/// let out_dir = std::env::var("OUT_DIR").unwrap();
/// markdown_to_cucumber::specs::generate_features(
///     std::path::Path::new("specs"),
///     std::path::Path::new(&out_dir),
/// );
/// ```
///
/// Nothing generated here is committed to git — documentation and the tests
/// that exercise it cannot drift, because the tests _are_ the documentation.
pub fn generate_features(specs_dir: &Path, out_dir: &Path) {
    let features_dir = out_dir.join("features");
    fs::create_dir_all(&features_dir).unwrap();

    println!("cargo:rerun-if-changed={}", specs_dir.display());

    let mut spec_files: Vec<_> = fs::read_dir(specs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
        .collect();
    spec_files.sort_by_key(|e| e.path());

    for entry in &spec_files {
        let path = entry.path();
        println!("cargo:rerun-if-changed={}", path.display());
        let content = fs::read_to_string(&path).unwrap();
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let blocks = extract_gherkin_blocks(&content);

        for (i, block) in blocks.iter().enumerate() {
            let name = if blocks.len() == 1 {
                stem.to_string()
            } else {
                format!("{stem}-{}", i + 1)
            };
            let feature_content =
                format!("# Generated from specs/{stem}.md\n\n{}\n", block.trim_end());
            fs::write(
                features_dir.join(format!("{name}.feature")),
                feature_content,
            )
            .unwrap();
        }
    }
}

fn extract_gherkin_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if !in_block {
            if trimmed == "```gherkin" || trimmed == "```feature" {
                in_block = true;
                current.clear();
            }
        } else if trimmed == "```" {
            blocks.push(current.clone());
            in_block = false;
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn extracts_a_single_gherkin_block_using_the_file_stem_as_name() {
        let dir = tempdir().unwrap();
        let specs_dir = dir.path().join("specs");
        let out_dir = dir.path().join("out");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("01-basics.md"),
            "# Basics\n\nSome prose.\n\n```gherkin\nFeature: Basics\n\n  Scenario: A\n    Given a thing\n```\n\nMore prose.\n",
        )
        .unwrap();

        generate_features(&specs_dir, &out_dir);

        let feature = std::fs::read_to_string(out_dir.join("features/01-basics.feature")).unwrap();
        assert!(feature.starts_with("# Generated from specs/01-basics.md\n\n"));
        assert!(feature.contains("Feature: Basics"));
        assert!(!feature.contains("Some prose"));
    }

    #[test]
    fn numbers_multiple_blocks_in_the_same_file() {
        let dir = tempdir().unwrap();
        let specs_dir = dir.path().join("specs");
        let out_dir = dir.path().join("out");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("01-basics.md"),
            "```gherkin\nFeature: A\n```\n\n```gherkin\nFeature: B\n```\n",
        )
        .unwrap();

        generate_features(&specs_dir, &out_dir);

        assert!(out_dir.join("features/01-basics-1.feature").exists());
        assert!(out_dir.join("features/01-basics-2.feature").exists());
    }

    #[test]
    fn ignores_non_markdown_files() {
        let dir = tempdir().unwrap();
        let specs_dir = dir.path().join("specs");
        let out_dir = dir.path().join("out");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("README.md"), "No gherkin here.\n").unwrap();
        std::fs::write(specs_dir.join("notes.txt"), "```gherkin\nFeature: X\n```\n").unwrap();

        generate_features(&specs_dir, &out_dir);

        let entries: Vec<_> = std::fs::read_dir(out_dir.join("features"))
            .unwrap()
            .collect();
        assert!(entries.is_empty());
    }
}
