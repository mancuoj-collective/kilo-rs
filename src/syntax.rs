//! Syntax highlighting: rules (data) plus a small lexer (behavior).
//!
//! A [`Syntax`] is pure data — filetype, keywords, comment markers, string quotes,
//! flags. The lexer walks one row's characters and assigns a [`Highlight`] category to
//! each; multi-line comments carry state from one row to the next. How a category
//! *looks* is decided by `ui`, not here: this module knows nothing about the terminal.
//!
//! This is a port of kilo's `editorUpdateSyntax` / `HLDB`, extended to more than one
//! language (kilo hard-codes `"` and `'` as string quotes; we let each rule decide).

/// A highlight category for a single character.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Highlight {
    Normal,
    Comment,
    MultilineComment,
    Keyword1,
    Keyword2,
    String,
    Number,
}

/// One language's highlighting rules.
pub struct Syntax {
    /// Name shown in the status bar.
    pub filetype: &'static str,
    /// File-name matches: `".c"` is an extension, `"Makefile"` a substring.
    filematch: &'static [&'static str],
    /// Keywords; a trailing `|` marks a type keyword ([`Highlight::Keyword2`]).
    keywords: &'static [&'static str],
    singleline_comment_start: Option<&'static str>,
    multiline_comment_start: Option<&'static str>,
    multiline_comment_end: Option<&'static str>,
    /// Quotes that open/close a string; empty means "don't highlight strings".
    string_delimiters: &'static [char],
    /// Whether to highlight numbers.
    highlight_numbers: bool,
    /// Whether block comments nest (Rust's `/* /* */ */`), rather than ending at the first `*/`.
    nested_comments: bool,
}

/// The built-in language table. kilo ships only C; we add Rust.
#[rustfmt::skip]
static HLDB: &[Syntax] = &[
    Syntax {
        filetype: "c",
        filematch: &[".c", ".h", ".cpp"],
        keywords: &[
            "switch", "if", "while", "for", "break", "continue", "return", "else", "struct",
            "union", "typedef", "static", "enum", "class", "case",
            "int|", "long|", "double|", "float|", "char|", "unsigned|", "signed|", "void|",
        ],
        singleline_comment_start: Some("//"),
        multiline_comment_start: Some("/*"),
        multiline_comment_end: Some("*/"),
        string_delimiters: &['"', '\''],
        highlight_numbers: true,
        nested_comments: false,
    },
    Syntax {
        filetype: "rs",
        filematch: &[".rs"],
        keywords: &[
            "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
            "extern", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
            "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
            "type", "unsafe", "use", "where", "while", "yield", "true", "false",
            "bool|", "char|", "str|", "i8|", "i16|", "i32|", "i64|", "i128|", "isize|",
            "u8|", "u16|", "u32|", "u64|", "u128|", "usize|", "f32|", "f64|",
        ],
        singleline_comment_start: Some("//"),
        multiline_comment_start: Some("/*"),
        multiline_comment_end: Some("*/"),
        // Only `"`: a single quote is a lifetime (`'a`) far more often than a char literal.
        string_delimiters: &['"'],
        highlight_numbers: true,
        nested_comments: true,
    },
];

impl Syntax {
    /// Picks the rules for `path`, by extension or substring match.
    #[must_use]
    pub fn for_path(path: &str) -> Option<&'static Syntax> {
        let extension = path.rsplit_once('.').map(|(_, extension)| extension);
        HLDB.iter().find(|syntax| syntax.matches(path, extension))
    }

    fn matches(&self, path: &str, extension: Option<&str>) -> bool {
        self.filematch
            .iter()
            .any(|pattern| match pattern.strip_prefix('.') {
                Some(pattern) => extension == Some(pattern),
                None => path.contains(pattern),
            })
    }

    /// Categorises `chars`, returning the categories (aligned with `chars`) and the
    /// block-comment depth at the end of the row (0 = not in a comment; nested comments
    /// can go deeper). The next row starts at that depth.
    #[must_use]
    pub fn highlight(&self, chars: &[char], begins_in_comment: u32) -> (Vec<Highlight>, u32) {
        let mut hl = vec![Highlight::Normal; chars.len()];
        let mut prev_sep = true;
        let mut in_string: Option<char> = None;
        let mut depth = begins_in_comment;
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            let prev_hl = if i > 0 { hl[i - 1] } else { Highlight::Normal };

            // A single-line comment swallows the rest of the row.
            let singleline = self.singleline_comment_start;
            if in_string.is_none()
                && depth == 0
                && singleline.is_some_and(|start| starts_with(chars, i, start))
            {
                hl[i..].fill(Highlight::Comment);
                break;
            }

            if in_string.is_none() {
                if depth > 0 {
                    if let Some(end) = self.multiline_comment_end {
                        if starts_with(chars, i, end) {
                            hl[i..i + end.chars().count()].fill(Highlight::MultilineComment);
                            i += end.chars().count();
                            depth -= 1;
                            prev_sep = true;
                            continue;
                        }
                    }
                    if self.nested_comments {
                        if let Some(start) = self.multiline_comment_start {
                            if starts_with(chars, i, start) {
                                hl[i..i + start.chars().count()].fill(Highlight::MultilineComment);
                                i += start.chars().count();
                                depth += 1;
                                continue;
                            }
                        }
                    }
                    hl[i] = Highlight::MultilineComment;
                    i += 1;
                    continue;
                }
                if let Some(start) = self.multiline_comment_start {
                    if starts_with(chars, i, start) {
                        hl[i..i + start.chars().count()].fill(Highlight::MultilineComment);
                        i += start.chars().count();
                        depth = 1;
                        continue;
                    }
                }
            }

            if !self.string_delimiters.is_empty() {
                if let Some(quote) = in_string {
                    hl[i] = Highlight::String;
                    if c == '\\' && i + 1 < chars.len() {
                        hl[i + 1] = Highlight::String;
                        i += 2;
                        continue;
                    }
                    if c == quote {
                        in_string = None;
                    }
                    i += 1;
                    prev_sep = true;
                    continue;
                } else if self.string_delimiters.contains(&c) {
                    in_string = Some(c);
                    hl[i] = Highlight::String;
                    i += 1;
                    continue;
                }
            }

            if self.highlight_numbers
                && ((c.is_ascii_digit() && (prev_sep || prev_hl == Highlight::Number))
                    || (c == '.' && prev_hl == Highlight::Number))
            {
                hl[i] = Highlight::Number;
                i += 1;
                prev_sep = false;
                continue;
            }

            if prev_sep {
                if let Some((len, second)) = self.match_keyword(chars, i) {
                    let kind = if second {
                        Highlight::Keyword2
                    } else {
                        Highlight::Keyword1
                    };
                    hl[i..i + len].fill(kind);
                    i += len;
                    prev_sep = false;
                    continue;
                }
            }

            prev_sep = is_separator(c);
            i += 1;
        }

        (hl, depth)
    }

    /// Length of a keyword starting at `at`, and whether it is a "type" keyword.
    fn match_keyword(&self, chars: &[char], at: usize) -> Option<(usize, bool)> {
        self.keywords.iter().find_map(|keyword| {
            let second = keyword.ends_with('|');
            let name = keyword.trim_end_matches('|');
            let len = name.chars().count();
            let boundary = chars.get(at + len).is_none_or(|&c| is_separator(c));
            (boundary && starts_with(chars, at, name)).then_some((len, second))
        })
    }
}

/// Whether `pattern` occurs in `chars` starting at `at`.
fn starts_with(chars: &[char], at: usize, pattern: &str) -> bool {
    pattern
        .chars()
        .enumerate()
        .all(|(offset, expected)| chars.get(at + offset) == Some(&expected))
}

/// Whether `c` ends a word (kilo's separator set).
fn is_separator(c: char) -> bool {
    c.is_whitespace() || ",.()+-/*=~%<>[];".contains(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    #[test]
    fn picks_a_language_by_extension() {
        assert_eq!(Syntax::for_path("main.c").map(|s| s.filetype), Some("c"));
        assert_eq!(Syntax::for_path("main.h").map(|s| s.filetype), Some("c"));
        assert_eq!(Syntax::for_path("main.rs").map(|s| s.filetype), Some("rs"));
        assert!(Syntax::for_path("main.txt").is_none());
    }

    #[test]
    fn keywords_numbers_and_comments() {
        let syntax = Syntax::for_path("x.c").unwrap();
        let text = "return 42; // note";
        let (hl, _) = syntax.highlight(&chars(text), 0);

        assert_eq!(hl[0], Highlight::Keyword1); // return
        assert_eq!(hl[7], Highlight::Number); // 4
        assert_eq!(hl[8], Highlight::Number); // 2
        let comment = text.find("//").unwrap();
        assert_eq!(hl[comment], Highlight::Comment);
    }

    #[test]
    fn type_keywords_are_their_own_category() {
        let syntax = Syntax::for_path("x.c").unwrap();
        let (hl, _) = syntax.highlight(&chars("int x;"), 0);
        assert_eq!(hl[0], Highlight::Keyword2); // int|
    }

    #[test]
    fn strings_include_escapes() {
        let syntax = Syntax::for_path("x.c").unwrap();
        let text = "char *s = \"a\\\"b\";";
        let (hl, _) = syntax.highlight(&chars(text), 0);

        let quote = text.find('"').unwrap();
        assert_eq!(hl[quote], Highlight::String);
        assert_eq!(hl[quote + 3], Highlight::String); // the escaped quote
    }

    #[test]
    fn multiline_comments_carry_across_rows() {
        let syntax = Syntax::for_path("x.c").unwrap();

        let (_, ends) = syntax.highlight(&chars("/* open"), 0);
        assert_eq!(ends, 1);

        let (hl, ends) = syntax.highlight(&chars("still */ int x;"), 1);
        assert_eq!(ends, 0);
        assert_eq!(hl[0], Highlight::MultilineComment);
        assert_eq!(hl[9], Highlight::Keyword2); // int, after the comment closes
    }

    #[test]
    fn rust_block_comments_nest() {
        let syntax = Syntax::for_path("x.rs").unwrap();
        // Rust nests: after the first `*/` the comment is still open.
        let (_, ends) = syntax.highlight(&chars("/* a /* b */"), 0);
        assert_eq!(ends, 1);
        let (_, ends) = syntax.highlight(&chars("still */ done"), ends);
        assert_eq!(ends, 0);

        // C does not nest: the first `*/` closes the comment.
        let c = Syntax::for_path("x.c").unwrap();
        let (_, ends) = c.highlight(&chars("/* a /* b */"), 0);
        assert_eq!(ends, 0);
    }

    #[test]
    fn rust_keywords_strings_and_primitive_types() {
        let syntax = Syntax::for_path("main.rs").unwrap();
        let text = "let n: u32 = 1; // note";
        let (hl, _) = syntax.highlight(&chars(text), 0);

        assert_eq!(hl[0], Highlight::Keyword1); // let
        assert_eq!(hl[7], Highlight::Keyword2); // u32
        assert_eq!(hl[13], Highlight::Number); // 1
        assert_eq!(hl[text.find("//").unwrap()], Highlight::Comment);
    }

    #[test]
    fn rust_lifetimes_do_not_open_a_string() {
        let syntax = Syntax::for_path("x.rs").unwrap();
        let text = "fn f<'a>(s: &'a str) {}";
        let (hl, _) = syntax.highlight(&chars(text), 0);

        // The apostrophe is a lifetime here, not a char-literal quote: nothing is swallowed.
        assert_eq!(hl[text.find('\'').unwrap()], Highlight::Normal);
        assert_eq!(hl[text.find("str").unwrap()], Highlight::Keyword2);
    }
}
