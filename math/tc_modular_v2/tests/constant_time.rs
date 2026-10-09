//! Reads the source of the crate and holds it to its timing conventions.
//!
//! Every public function and every trait implementation says in its doc
//! whether it is constant time or variable time, and a variable-time one
//! says it is only for public values. A big type, which a variable-time
//! `BigUint` is behind, names the padded type that takes secret values.
//! Code that says it is constant time calls nothing that only variable-time
//! items of the crate define.
//!
//! The scan reads every file under `src`, so new code is held to the same
//! rules without this file changing. It reads lines, not Rust: an item is
//! a line that starts one, its doc is the `///` lines right above it, and
//! a line inside it belongs to the closest documented item it is indented
//! under. A test module, from `#[cfg(test)]` on, is not read, nor is a file
//! that only a `#[cfg(test)] mod` declares.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// What the doc of an item says about its timing.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Timing {
    Constant,
    Variable,
}

/// One line the scan read: where, what it says, the doc above it when it
/// starts an item, how far it is indented, and the timing that doc states.
struct Line {
    file: String,
    text: String,
    docs: String,
    indent: usize,
    item: bool,
    timing: Option<Timing>,
}

/// The `.rs` files under `src`, by their path from it.
fn sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, root: &Path, files: &mut Vec<(String, String)>) {
        let mut entries: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, root, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let name = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push((name, fs::read_to_string(&path).unwrap()));
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk(&root, &root, &mut files);
    files
}

/// The timing a doc states, or `None`: both labels at once is no answer.
fn timing(docs: &str) -> Option<Timing> {
    match (
        docs.contains("Constant time"),
        docs.contains("Variable time"),
    ) {
        (true, false) => Some(Timing::Constant),
        (false, true) => Some(Timing::Variable),
        _ => None,
    }
}

/// Whether `text`, trimmed, starts an item that carries a doc: a function,
/// an implementation, a type or a trait.
fn starts_item(text: &str) -> bool {
    let text = text
        .trim_start_matches("pub(crate) ")
        .trim_start_matches("pub(super) ")
        .trim_start_matches("pub ")
        .trim_start_matches("const ")
        .trim_start_matches("unsafe ");
    [
        "fn ", "impl ", "impl<", "struct ", "enum ", "trait ", "type ",
    ]
    .iter()
    .any(|start| text.starts_with(start))
}

/// The lines of `files` that hold code, without the test modules and the
/// files only they declare.
fn lines_of(files: &[(String, String)]) -> Vec<Line> {
    let mut test_only = BTreeSet::new();
    for (name, source) in files {
        let parent = name.trim_end_matches(".rs");
        let parent = if parent == "lib" {
            String::new()
        } else {
            format!("{parent}/")
        };
        let mut lines = source.lines().map(str::trim);
        while let Some(text) = lines.next() {
            if text == "#[cfg(test)]" {
                let next = lines.next().unwrap_or_default();
                if let Some(module) = next
                    .strip_prefix("mod ")
                    .and_then(|rest| rest.strip_suffix(';'))
                {
                    test_only.insert(format!("{parent}{module}.rs"));
                }
            }
        }
    }

    let mut read = Vec::new();
    for (file, source) in files.iter().filter(|(name, _)| !test_only.contains(name)) {
        let mut docs = String::new();
        let mut attribute_depth = 0_i32;
        for raw in source.lines() {
            let text = raw.trim();
            if raw.starts_with("#[cfg(test)]") || raw.starts_with("#[cfg(all(test") {
                break;
            }
            if let Some(comment) = text.strip_prefix("///") {
                docs.push_str(comment.trim());
                docs.push(' ');
                continue;
            }
            if text.starts_with("#[") || text.starts_with("#![") || attribute_depth > 0 {
                attribute_depth +=
                    text.matches('[').count() as i32 - text.matches(']').count() as i32;
                continue;
            }
            if text.is_empty() || text.starts_with("//") {
                continue;
            }
            let item = starts_item(text);
            read.push(Line {
                file: file.clone(),
                text: text.to_owned(),
                docs: if item { docs.clone() } else { String::new() },
                indent: raw.len() - raw.trim_start().len(),
                item,
                timing: if item { timing(&docs) } else { None },
            });
            docs.clear();
        }
    }
    read
}

/// Whether the line starts an item that must state its timing: a public
/// function, or a trait implementation with a body.
fn must_state_timing(line: &Line) -> bool {
    let public_fn = line.text.starts_with("pub fn ") || line.text.starts_with("pub const fn ");
    let trait_impl = line.indent == 0
        && (line.text.starts_with("impl ") || line.text.starts_with("impl<"))
        && line.text.contains(" for ")
        && !line.text.ends_with("{}");
    line.item && (public_fn || trait_impl)
}

/// The name a function line defines.
fn function_name(text: &str) -> Option<&str> {
    let start = text.find("fn ")? + 3;
    let rest = &text[start..];
    let end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_'))?;
    Some(&rest[..end])
}

fn located(line: &Line) -> String {
    format!("{}: {}", line.file, line.text)
}

/// The items that must state their timing but state none, or both.
fn without_timing(lines: &[Line]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| must_state_timing(line) && line.timing.is_none())
        .map(located)
        .collect()
}

/// The variable-time items that do not say they are only for public values.
fn unqualified(lines: &[Line]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| must_state_timing(line) && line.timing == Some(Timing::Variable))
        .filter(|line| {
            !(line.docs.contains("only for public") || line.docs.contains("only for a public"))
        })
        .map(located)
        .collect()
}

/// The big types whose doc links no padded type.
fn big_without_padded(lines: &[Line]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| line.item && line.text.starts_with("pub struct Big"))
        .filter(|line| !line.docs.contains("[`Padded"))
        .map(located)
        .collect()
}

/// The lines in constant-time code that call a function only variable-time
/// items define, as `line calls name`.
fn constant_calling_variable(lines: &[Line]) -> Vec<String> {
    let named = |wanted: Timing| -> BTreeSet<&str> {
        lines
            .iter()
            .filter(|line| line.timing == Some(wanted))
            .filter_map(|line| function_name(&line.text))
            .collect()
    };
    let constant = named(Timing::Constant);
    let variable: BTreeSet<_> = named(Timing::Variable)
        .difference(&constant)
        .copied()
        .collect();

    let mut enclosing: Vec<(usize, Option<Timing>)> = Vec::new();
    let mut file = "";
    let mut calls = Vec::new();
    for line in lines {
        if line.file != file {
            file = &line.file;
            enclosing.clear();
        }
        while enclosing
            .last()
            .is_some_and(|&(indent, _)| indent >= line.indent)
        {
            enclosing.pop();
        }
        if line.item {
            enclosing.push((line.indent, line.timing));
        }
        if enclosing.iter().rev().find_map(|&(_, timing)| timing) != Some(Timing::Constant) {
            continue;
        }
        for name in &variable {
            let pattern = format!("{name}(");
            let called = line.text.match_indices(&pattern).any(|(at, _)| {
                !line.text[..at].ends_with(|c: char| c.is_alphanumeric() || c == '_')
            });
            if called && function_name(&line.text) != Some(name) {
                calls.push(format!("{} calls {name}", located(line)));
            }
        }
    }
    calls
}

#[test]
fn the_scan_finds_each_breach_it_is_written_for() {
    let source = "\
/// Constant time.
pub fn safe() {
    leaky();
}

/// Variable time: only for public values.
pub fn leaky() {}

/// Variable time.
pub fn unqualified() {}

pub fn undocumented() {}

/// Arbitrary.
pub struct BigThing;

impl Clone for Thing {
    fn clone(&self) -> Self {
        Thing
    }
}

impl Eq for Thing {}

#[cfg(test)]
mod tests {
    pub fn ignored() {}
}
";
    let lines = lines_of(&[("x.rs".to_owned(), source.to_owned())]);
    assert_eq!(
        without_timing(&lines),
        [
            "x.rs: pub fn undocumented() {}",
            "x.rs: impl Clone for Thing {"
        ]
    );
    assert_eq!(unqualified(&lines), ["x.rs: pub fn unqualified() {}"]);
    assert_eq!(big_without_padded(&lines), ["x.rs: pub struct BigThing;"]);
    assert_eq!(
        constant_calling_variable(&lines),
        ["x.rs: leaky(); calls leaky"]
    );
}

#[test]
fn every_public_function_and_trait_implementation_states_its_timing() {
    let lines = lines_of(&sources());
    assert!(lines.iter().any(must_state_timing), "nothing scanned");
    let missing = without_timing(&lines);
    assert!(
        missing.is_empty(),
        "no single timing stated:\n{}",
        missing.join("\n")
    );
}

#[test]
fn a_variable_time_item_says_it_is_only_for_public_values() {
    let missing = unqualified(&lines_of(&sources()));
    assert!(
        missing.is_empty(),
        "variable time without a word on its use:\n{}",
        missing.join("\n")
    );
}

#[test]
fn a_big_type_names_the_padded_type_for_secret_values() {
    let lines = lines_of(&sources());
    assert!(
        lines
            .iter()
            .any(|line| line.text.starts_with("pub struct Big")),
        "no big type scanned"
    );
    let missing = big_without_padded(&lines);
    assert!(
        missing.is_empty(),
        "big types naming no padded type:\n{}",
        missing.join("\n")
    );
}

#[test]
fn constant_time_code_calls_nothing_only_variable_time_items_define() {
    let calls = constant_calling_variable(&lines_of(&sources()));
    assert!(
        calls.is_empty(),
        "constant-time code calling variable-time code:\n{}",
        calls.join("\n")
    );
}
