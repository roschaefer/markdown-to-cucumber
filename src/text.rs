use cucumber::gherkin::Step;

/// Extracts the text of a Gherkin docstring (`"""..."""`) argument, with
/// common leading indentation stripped so the Markdown-embedded scenario can
/// stay nicely indented under its `Given`/`When`/`Then` line.
///
/// The `gherkin` crate's raw `docstring` value always includes a literal
/// first line reserved for an optional media-type annotation (`"""toml`) —
/// empty when no media type is given. That first line is always dropped;
/// what's left has its minimum indentation stripped and reassembled, which
/// mirrors the behavior of Python `behave`'s `context.text`.
pub fn docstring(step: &Step) -> String {
    let raw = step.docstring.as_deref().unwrap_or("");
    let lines: Vec<&str> = raw.split('\n').collect();
    let content = if lines.is_empty() {
        &lines[..]
    } else {
        &lines[1..]
    };

    let min_indent = content
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    let result = content
        .iter()
        .map(|l| {
            if l.len() >= min_indent {
                &l[min_indent..]
            } else {
                l.trim_start()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    result.trim_end_matches('\n').to_string()
}

/// Returns a step's Gherkin data table as rows of cells.
pub fn gherkin_table(step: &Step) -> Vec<Vec<String>> {
    let table = step.table.as_ref().expect("step requires a table");
    table.rows.clone()
}

/// Checks that `text` contains every non-`...` chunk of `pattern`, in order,
/// allowing arbitrary text in between chunks wherever a line consisting of
/// exactly `...` appears in `pattern`. With no `...` present, this is a
/// plain substring check.
pub fn contains_in_order(text: &str, pattern: &str) -> bool {
    let chunks = ellipsis_chunks(pattern);
    let mut pos = 0;
    for chunk in &chunks {
        if chunk.is_empty() {
            continue;
        }
        match text[pos..].find(chunk.as_str()) {
            Some(idx) => pos += idx + chunk.len(),
            None => return false,
        }
    }
    true
}

fn ellipsis_chunks(pattern: &str) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in pattern.lines() {
        if line.trim() == "..." {
            chunks.push(current.join("\n").trim_matches('\n').to_string());
            current.clear();
        } else {
            current.push(line);
        }
    }
    chunks.push(current.join("\n").trim_matches('\n').to_string());
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_in_order_matches_plain_substring_with_no_ellipsis() {
        assert!(contains_in_order("hello world", "hello"));
        assert!(!contains_in_order("hello world", "goodbye"));
    }

    #[test]
    fn contains_in_order_allows_gaps_at_ellipsis_lines() {
        let text = "start\nmiddle stuff\nend";
        assert!(contains_in_order(text, "start\n...\nend"));
        assert!(!contains_in_order(text, "end\n...\nstart"));
    }
}
