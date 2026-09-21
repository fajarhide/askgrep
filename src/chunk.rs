//! Splits a tree of source files into the units a question gets asked about.
//!
//! Language agnostic on purpose. A top-level definition is any non-blank line at
//! zero indentation that is not a closing brace or a comment, which catches Rust
//! `fn`/`impl`, Python `def`/`class`, Go `func` and JS `export function` without
//! carrying a parser per language.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

pub struct Chunk {
    pub file: PathBuf,
    /// 1-indexed, so it can be pasted straight into an editor.
    pub line: usize,
    pub text: String,
}

impl Chunk {
    /// First line with something on it. A windowed continuation carries its
    /// definition line, so this names the function either way.
    pub fn head(&self) -> &str {
        self.text
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim_end()
    }
}

/// A nested method is the common case, not the exception: nearly all Rust lives
/// inside an `impl` and nearly all Python inside a `class`, so a zero-indent rule
/// alone turns a 5,000 line file into windows all labelled `impl Foo {`.
const DEFINITION_KEYWORDS: &[&str] = &[
    "fn ",
    "pub fn ",
    "async fn ",
    "pub async fn ",
    "def ",
    "async def ",
    "func ",
    "function ",
    "class ",
    "impl ",
    "struct ",
    "enum ",
    "trait ",
    "interface ",
    "public ",
    "private ",
    "protected ",
    "static ",
    "const fn ",
];

/// ponytail: indentation plus a keyword list, swap in tree-sitter if a language
/// it cannot read (Haskell, Lisp) turns up in real use. A miss degrades into
/// windowing rather than into a wrong answer.
fn starts_definition(line: &str) -> bool {
    let t = line.trim_end();
    if t.is_empty() {
        return false;
    }
    let f = t.trim_start();
    if f.starts_with('}')
        || f.starts_with(')')
        || f.starts_with("//")
        || f.starts_with('#')
        || f.starts_with("/*")
        || f.starts_with('*')
    {
        return false;
    }
    !line.starts_with([' ', '\t']) || DEFINITION_KEYWORDS.iter().any(|k| f.starts_with(k))
}

fn split_file(path: &Path, text: &str, min_bytes: usize, max_bytes: usize) -> Vec<Chunk> {
    let lines: Vec<&str> = text.lines().collect();
    let mut bounds: Vec<usize> = (0..lines.len())
        .filter(|&i| starts_definition(lines[i]))
        .collect();
    if bounds.first() != Some(&0) {
        bounds.insert(0, 0);
    }
    bounds.push(lines.len());

    let mut out = Vec::new();
    for w in bounds.windows(2) {
        let (start, end) = (w[0], w[1]);
        if start == end {
            continue;
        }
        // A span longer than max_bytes is windowed rather than sent whole: the
        // pack-size measurement showed accuracy falls off as the state grows.
        // Every window after the first repeats the definition line, otherwise the
        // model is asked about a fragment with no idea what it belongs to, and the
        // hit prints as `tool_family,` with nothing to act on.
        let header = lines[start];
        let mut i = start;
        while i < end {
            let mut j = i;
            let mut bytes = 0;
            while j < end && bytes < max_bytes {
                bytes += lines[j].len() + 1;
                j += 1;
            }
            let mut body = lines[i..j].join("\n");
            if i > start {
                body = format!("{header}\n    // ...\n{body}");
            }
            if body.trim().len() >= min_bytes {
                out.push(Chunk {
                    file: path.to_path_buf(),
                    line: i + 1,
                    text: body,
                });
            }
            i = j;
        }
    }
    out
}

/// `.git` is never source, and its text files are worse than noise: `config`
/// carries remote URLs, `logs/HEAD` carries the author's name and email on every
/// line, and `COMMIT_EDITMSG` carries whatever was last being written. Sending
/// those to a model API is not something a search tool should do quietly, so the
/// check is explicit rather than left to the hidden-file default.
fn is_git_internal(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == ".git")
}

pub fn collect(root: &Path, min_bytes: usize, max_bytes: usize) -> Vec<Chunk> {
    let mut out = Vec::new();
    // Hidden files are skipped, the same default ripgrep uses. The previous
    // `hidden(false)` walked `.git`, which was 24% of the chunks on one repo and
    // shipped git internals to the model.
    for entry in WalkBuilder::new(root).build().flatten() {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let path = entry.path();
        if is_git_internal(path) {
            continue;
        }
        // Binary and minified files answer no question worth asking.
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if text.len() > 2_000_000 || text.lines().any(|l| l.len() > 2_000) {
            continue;
        }
        out.extend(split_file(path, &text, min_bytes, max_bytes));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_zero_indent_definitions_and_keeps_line_numbers() {
        let src = "use std::io;\n\nfn alpha() {\n    let a = 1;\n    let b = 2;\n}\n\nfn beta() {\n    let c = 3;\n    let d = 4;\n}\n";
        let cs = split_file(Path::new("x.rs"), src, 10, 4096);
        let heads: Vec<&str> = cs.iter().map(|c| c.head()).collect();
        assert!(heads.contains(&"fn alpha() {"), "got {heads:?}");
        assert!(heads.contains(&"fn beta() {"), "got {heads:?}");
        let beta = cs.iter().find(|c| c.head() == "fn beta() {").unwrap();
        assert_eq!(
            beta.line, 8,
            "line numbers are 1-indexed and point at the definition"
        );
        assert!(
            beta.text.contains("let d = 4;"),
            "body runs to the next definition"
        );
    }

    #[test]
    fn a_windowed_continuation_still_names_its_definition() {
        let long: String = (0..300).map(|i| format!("    let v{i} = {i};\n")).collect();
        let src = format!("fn enormous() {{\n{long}}}\n");
        let cs = split_file(Path::new("big.rs"), &src, 10, 800);
        assert!(cs.len() > 2, "expected several windows, got {}", cs.len());
        for c in &cs {
            assert_eq!(
                c.head(),
                "fn enormous() {",
                "every window names the function"
            );
        }
    }

    #[test]
    fn a_file_with_no_definitions_is_windowed_not_sent_whole() {
        let src = (0..400)
            .map(|i| format!("    value_{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let cs = split_file(Path::new("data.txt"), &src, 10, 500);
        assert!(
            cs.len() > 1,
            "expected windowing, got {} chunk(s)",
            cs.len()
        );
        assert!(cs.iter().all(|c| c.text.len() <= 600));
    }

    #[test]
    fn methods_inside_an_impl_block_are_their_own_chunks() {
        let src = "impl Store {\n    pub fn get(&self) -> u8 {\n        self.a\n    }\n\n    pub fn put(&mut self, v: u8) {\n        self.a = v;\n    }\n}\n";
        let cs = split_file(Path::new("s.rs"), src, 5, 4096);
        let heads: Vec<&str> = cs.iter().map(|c| c.head()).collect();
        assert!(heads.iter().any(|h| h.contains("fn get")), "got {heads:?}");
        assert!(heads.iter().any(|h| h.contains("fn put")), "got {heads:?}");
    }

    #[test]
    fn git_internals_are_never_chunked() {
        // Catches the case the hidden-file default misses: a path pointed
        // straight at .git, or a .git nested inside a submodule.
        assert!(is_git_internal(Path::new(".git/config")));
        assert!(is_git_internal(Path::new("/repo/.git/logs/HEAD")));
        assert!(is_git_internal(Path::new(
            "/repo/vendor/dep/.git/COMMIT_EDITMSG"
        )));
        assert!(!is_git_internal(Path::new("/repo/src/git.rs")));
        assert!(!is_git_internal(Path::new(
            "/repo/.github/workflows/ci.yml"
        )));
    }

    #[test]
    fn closing_braces_and_comments_do_not_start_a_chunk() {
        assert!(!starts_definition("}"));
        assert!(!starts_definition("// a note"));
        assert!(
            starts_definition("    fn indented() {"),
            "a method is a definition"
        );
        assert!(!starts_definition("    let x = 1;"), "a statement is not");
        assert!(starts_definition("pub fn real() {"));
        assert!(starts_definition("def python_fn():"));
    }
}
