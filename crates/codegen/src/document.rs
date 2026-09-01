//! Target-neutral immutable document algebra and iterative pretty-printer.
//!
//! ```
//! use portable_codegen::{
//!     Document, FinalNewline, RenderOptions, render,
//! };
//!
//! let word = |value| Document::text(value).expect("no raw controls");
//! let document = Document::concat([
//!     word("item"),
//!     Document::line(),
//!     word("="),
//!     Document::line(),
//!     word("value"),
//! ]).group();
//! let compact = render(
//!     &document,
//!     RenderOptions {
//!         width: 20,
//!         final_newline: FinalNewline::Never,
//!         ..RenderOptions::default()
//!     },
//! ).unwrap();
//! let broken = render(
//!     &document,
//!     RenderOptions {
//!         width: 5,
//!         final_newline: FinalNewline::Never,
//!         ..RenderOptions::default()
//!     },
//! ).unwrap();
//! assert_eq!(compact, "item = value");
//! assert_eq!(broken, "item\n=\nvalue");
//! ```

use std::{fmt, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document(Arc<Node>);

#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Empty,
    Text { value: String, raw: bool },
    SoftLine,
    HardLine,
    Concat(Arc<[Document]>),
    Indent { spaces: usize, document: Document },
    Group(Document),
    IfBreak { broken: Document, flat: Document },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawText(String);

impl RawText {
    /// Makes control-character use explicit. Rendering still normalizes CRLF
    /// and lone CR to LF.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextError {
    pub character: char,
    pub scalar_index: usize,
}

impl fmt::Display for TextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "document text contains control character {:?} at scalar index {}; use RawText or a line node",
            self.character, self.scalar_index
        )
    }
}

impl std::error::Error for TextError {}

impl Document {
    pub fn empty() -> Self {
        Self(Arc::new(Node::Empty))
    }

    pub fn text(value: impl Into<String>) -> Result<Self, TextError> {
        let value = value.into();
        if let Some((scalar_index, character)) = value
            .chars()
            .enumerate()
            .find(|(_, character)| character.is_control())
        {
            return Err(TextError {
                character,
                scalar_index,
            });
        }
        Ok(Self(Arc::new(Node::Text { value, raw: false })))
    }

    pub fn raw_text(value: RawText) -> Self {
        Self(Arc::new(Node::Text {
            value: value.0,
            raw: true,
        }))
    }

    /// A space in flat mode and a newline in broken mode.
    pub fn line() -> Self {
        Self(Arc::new(Node::SoftLine))
    }

    /// A newline in every layout mode.
    pub fn hard_line() -> Self {
        Self(Arc::new(Node::HardLine))
    }

    pub fn concat(documents: impl IntoIterator<Item = Self>) -> Self {
        let mut flattened = Vec::new();
        for document in documents {
            match document.0.as_ref() {
                Node::Empty => {}
                Node::Concat(children) => flattened.extend(children.iter().cloned()),
                _ => flattened.push(document),
            }
        }
        let documents = flattened;
        match documents.as_slice() {
            [] => Self::empty(),
            [document] => document.clone(),
            _ => Self(Arc::new(Node::Concat(documents.into()))),
        }
    }

    pub fn indent(self, spaces: usize) -> Self {
        Self(Arc::new(Node::Indent {
            spaces,
            document: self,
        }))
    }

    pub fn group(self) -> Self {
        Self(Arc::new(Node::Group(self)))
    }

    pub fn if_break(broken: Self, flat: Self) -> Self {
        Self(Arc::new(Node::IfBreak { broken, flat }))
    }

    pub fn join(separator: Self, documents: impl IntoIterator<Item = Self>) -> Self {
        let mut output = Vec::new();
        for document in documents {
            if !output.is_empty() {
                output.push(separator.clone());
            }
            output.push(document);
        }
        Self::concat(output)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinalNewline {
    Preserve,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderLimits {
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_output_bytes: usize,
}

impl Default for RenderLimits {
    fn default() -> Self {
        Self {
            max_depth: 4_096,
            max_nodes: 1_000_000,
            max_output_bytes: 16 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderOptions {
    pub width: usize,
    pub final_newline: FinalNewline,
    pub limits: RenderLimits,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 100,
            final_newline: FinalNewline::Always,
            limits: RenderLimits::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderStats {
    pub nodes_visited: usize,
    pub peak_pending_frames: usize,
    pub peak_output_capacity_bytes: usize,
    pub output_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    DepthLimit { limit: usize },
    NodeLimit { limit: usize },
    OutputLimit { limit: usize },
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DepthLimit { limit } => {
                write!(formatter, "document depth exceeds limit {limit}")
            }
            Self::NodeLimit { limit } => {
                write!(formatter, "document traversal exceeds node limit {limit}")
            }
            Self::OutputLimit { limit } => {
                write!(formatter, "rendered document exceeds {limit} bytes")
            }
        }
    }
}

impl std::error::Error for RenderError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedDocument {
    pub text: String,
    pub stats: RenderStats,
}

pub fn render(document: &Document, options: RenderOptions) -> Result<String, RenderError> {
    render_with_stats(document, options).map(|rendered| rendered.text)
}

#[derive(Clone, Copy)]
enum Mode {
    Flat,
    Break,
}

#[derive(Clone, Copy)]
struct Frame<'a> {
    indent: usize,
    mode: Mode,
    depth: usize,
    document: &'a Document,
}

struct Renderer {
    options: RenderOptions,
    output: String,
    column: usize,
    stats: RenderStats,
}

pub fn render_with_stats(
    document: &Document,
    options: RenderOptions,
) -> Result<RenderedDocument, RenderError> {
    let mut renderer = Renderer {
        options,
        output: String::new(),
        column: 0,
        stats: RenderStats::default(),
    };
    renderer.render(document)?;
    renderer.apply_final_newline()?;
    renderer.stats.output_bytes = renderer.output.len();
    renderer.stats.peak_output_capacity_bytes = renderer
        .stats
        .peak_output_capacity_bytes
        .max(renderer.output.capacity());
    Ok(RenderedDocument {
        text: renderer.output,
        stats: renderer.stats,
    })
}

impl Renderer {
    fn render(&mut self, document: &Document) -> Result<(), RenderError> {
        let mut stack = vec![Frame {
            indent: 0,
            mode: Mode::Break,
            depth: 1,
            document,
        }];
        self.observe_stack(&stack);
        while let Some(frame) = stack.pop() {
            self.visit(frame.depth)?;
            match frame.document.0.as_ref() {
                Node::Empty => {}
                Node::Text { value, raw } => self.append_text(value, *raw)?,
                Node::SoftLine => match frame.mode {
                    Mode::Flat => self.append_text(" ", false)?,
                    Mode::Break => self.append_line(frame.indent)?,
                },
                Node::HardLine => self.append_line(frame.indent)?,
                Node::Concat(documents) => {
                    let depth = frame.depth.saturating_add(1);
                    for document in documents.iter().rev() {
                        stack.push(Frame {
                            indent: frame.indent,
                            mode: frame.mode,
                            depth,
                            document,
                        });
                    }
                    self.observe_stack(&stack);
                }
                Node::Indent { spaces, document } => {
                    stack.push(Frame {
                        indent: frame.indent.saturating_add(*spaces),
                        mode: frame.mode,
                        depth: frame.depth.saturating_add(1),
                        document,
                    });
                    self.observe_stack(&stack);
                }
                Node::Group(document) => {
                    let mode = match frame.mode {
                        Mode::Flat => Mode::Flat,
                        Mode::Break => {
                            let remaining = self.options.width.saturating_sub(self.column);
                            if self.fits(document, remaining, frame.depth.saturating_add(1))? {
                                Mode::Flat
                            } else {
                                Mode::Break
                            }
                        }
                    };
                    stack.push(Frame {
                        indent: frame.indent,
                        mode,
                        depth: frame.depth.saturating_add(1),
                        document,
                    });
                    self.observe_stack(&stack);
                }
                Node::IfBreak { broken, flat } => {
                    stack.push(Frame {
                        indent: frame.indent,
                        mode: frame.mode,
                        depth: frame.depth.saturating_add(1),
                        document: match frame.mode {
                            Mode::Flat => flat,
                            Mode::Break => broken,
                        },
                    });
                    self.observe_stack(&stack);
                }
            }
        }
        Ok(())
    }

    fn fits(
        &mut self,
        document: &Document,
        mut remaining: usize,
        depth: usize,
    ) -> Result<bool, RenderError> {
        let mut stack = vec![(document, depth)];
        self.stats.peak_pending_frames = self.stats.peak_pending_frames.max(stack.len());
        while let Some((document, depth)) = stack.pop() {
            self.visit(depth)?;
            match document.0.as_ref() {
                Node::Empty => {}
                Node::Text { value, raw } => {
                    if *raw && contains_newline(value) {
                        return Ok(false);
                    }
                    let width = value.chars().count();
                    if width > remaining {
                        return Ok(false);
                    }
                    remaining -= width;
                }
                Node::SoftLine => {
                    if remaining == 0 {
                        return Ok(false);
                    }
                    remaining -= 1;
                }
                Node::HardLine => return Ok(false),
                Node::Concat(documents) => {
                    let child_depth = depth.saturating_add(1);
                    for document in documents.iter().rev() {
                        stack.push((document, child_depth));
                    }
                }
                Node::Indent { document, .. } | Node::Group(document) => {
                    stack.push((document, depth.saturating_add(1)));
                }
                Node::IfBreak { flat, .. } => {
                    stack.push((flat, depth.saturating_add(1)));
                }
            }
            self.stats.peak_pending_frames = self.stats.peak_pending_frames.max(stack.len());
        }
        Ok(true)
    }

    fn visit(&mut self, depth: usize) -> Result<(), RenderError> {
        if depth > self.options.limits.max_depth {
            return Err(RenderError::DepthLimit {
                limit: self.options.limits.max_depth,
            });
        }
        self.stats.nodes_visited = self.stats.nodes_visited.saturating_add(1);
        if self.stats.nodes_visited > self.options.limits.max_nodes {
            return Err(RenderError::NodeLimit {
                limit: self.options.limits.max_nodes,
            });
        }
        Ok(())
    }

    fn append_text(&mut self, value: &str, raw: bool) -> Result<(), RenderError> {
        if !raw {
            self.reserve_output(value.len())?;
            self.output.push_str(value);
            self.column = self.column.saturating_add(value.chars().count());
            self.observe_output();
            return Ok(());
        }

        let mut characters = value.chars().peekable();
        while let Some(character) = characters.next() {
            match character {
                '\r' => {
                    if characters.peek() == Some(&'\n') {
                        characters.next();
                    }
                    self.append_raw_character('\n')?;
                }
                other => self.append_raw_character(other)?,
            }
        }
        Ok(())
    }

    fn append_raw_character(&mut self, character: char) -> Result<(), RenderError> {
        self.reserve_output(character.len_utf8())?;
        self.output.push(character);
        if character == '\n' {
            self.column = 0;
        } else {
            self.column = self.column.saturating_add(1);
        }
        self.observe_output();
        Ok(())
    }

    fn append_line(&mut self, indent: usize) -> Result<(), RenderError> {
        self.reserve_output(1_usize.saturating_add(indent))?;
        self.output.push('\n');
        self.output.extend(std::iter::repeat_n(' ', indent));
        self.column = indent;
        self.observe_output();
        Ok(())
    }

    fn reserve_output(&self, additional: usize) -> Result<(), RenderError> {
        if self.output.len().saturating_add(additional) > self.options.limits.max_output_bytes {
            Err(RenderError::OutputLimit {
                limit: self.options.limits.max_output_bytes,
            })
        } else {
            Ok(())
        }
    }

    fn apply_final_newline(&mut self) -> Result<(), RenderError> {
        match self.options.final_newline {
            FinalNewline::Preserve => {}
            FinalNewline::Always => {
                while self.output.ends_with('\n') {
                    self.output.pop();
                }
                self.reserve_output(1)?;
                self.output.push('\n');
            }
            FinalNewline::Never => {
                while self.output.ends_with('\n') {
                    self.output.pop();
                }
            }
        }
        self.observe_output();
        Ok(())
    }

    fn observe_stack<T>(&mut self, stack: &[T]) {
        self.stats.peak_pending_frames = self.stats.peak_pending_frames.max(stack.len());
    }

    fn observe_output(&mut self) {
        self.stats.peak_output_capacity_bytes = self
            .stats
            .peak_output_capacity_bytes
            .max(self.output.capacity());
    }
}

fn contains_newline(value: &str) -> bool {
    value.contains('\n') || value.contains('\r')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(value: &str) -> Document {
        Document::text(value).unwrap()
    }

    fn options(width: usize) -> RenderOptions {
        RenderOptions {
            width,
            final_newline: FinalNewline::Never,
            ..RenderOptions::default()
        }
    }

    #[test]
    fn document_golden_flat_and_broken_groups() {
        let document = Document::concat([text("alpha"), Document::line(), text("beta")]).group();
        assert_eq!(render(&document, options(10)), Ok("alpha beta".to_owned()));
        assert_eq!(render(&document, options(9)), Ok("alpha\nbeta".to_owned()));

        let conditional = Document::concat([
            text("item"),
            Document::if_break(text(","), text(";")),
            Document::line(),
            text("next"),
        ])
        .group();
        assert_eq!(
            render(&conditional, options(20)),
            Ok("item; next".to_owned())
        );
        assert_eq!(
            render(&conditional, options(5)),
            Ok("item,\nnext".to_owned())
        );
    }

    #[test]
    fn nested_indentation_empty_join_long_tokens_and_unicode_are_golden() {
        let body = Document::concat([
            Document::hard_line(),
            Document::join(Document::hard_line(), [text("first"), text("second")]),
        ])
        .indent(4);
        let document = Document::concat([text("header"), body]);
        assert_eq!(
            render(&document, options(80)),
            Ok("header\n    first\n    second".to_owned())
        );
        assert_eq!(
            render(&Document::join(text(","), []), options(80)),
            Ok(String::new())
        );
        assert_eq!(
            render(&text("extraordinary"), options(3)),
            Ok("extraordinary".to_owned())
        );
        assert_eq!(
            render(
                &Document::concat([text("🦀"), Document::line(), text("e\u{301}")]).group(),
                options(4)
            ),
            Ok("🦀 e\u{301}".to_owned())
        );
    }

    #[test]
    fn line_endings_and_final_newline_policies_are_exact() {
        let raw = Document::raw_text(RawText::new("a\r\nb\rc\n"));
        assert_eq!(
            render(
                &raw,
                RenderOptions {
                    final_newline: FinalNewline::Preserve,
                    ..options(80)
                }
            ),
            Ok("a\nb\nc\n".to_owned())
        );
        assert_eq!(
            render(
                &raw,
                RenderOptions {
                    final_newline: FinalNewline::Always,
                    ..options(80)
                }
            ),
            Ok("a\nb\nc\n".to_owned())
        );
        assert_eq!(
            render(
                &raw,
                RenderOptions {
                    final_newline: FinalNewline::Never,
                    ..options(80)
                }
            ),
            Ok("a\nb\nc".to_owned())
        );
        assert_eq!(
            render(
                &Document::empty(),
                RenderOptions {
                    final_newline: FinalNewline::Always,
                    ..options(80)
                }
            ),
            Ok("\n".to_owned())
        );
    }

    #[test]
    fn normal_text_rejects_raw_controls_explicitly() {
        assert_eq!(
            Document::text("a\tb").unwrap_err(),
            TextError {
                character: '\t',
                scalar_index: 1,
            }
        );
        assert!(Document::text("plain 🦀").is_ok());
        assert_eq!(
            render(&Document::raw_text(RawText::new("a\tb")), options(80)),
            Ok("a\tb".to_owned())
        );
    }

    #[test]
    fn width_boundaries_follow_flat_scalar_width() {
        let document = Document::concat([text("a"), Document::line(), text("b")]).group();
        assert_eq!(render(&document, options(3)), Ok("a b".to_owned()));
        assert_eq!(render(&document, options(2)), Ok("a\nb".to_owned()));

        let tokens = ["a", "bb", "ccc", "dddd"];
        let joined = Document::join(Document::line(), tokens.iter().copied().map(text)).group();
        for width in 1..=20 {
            let rendered = render(&joined, options(width)).unwrap();
            for line in rendered.lines() {
                assert!(
                    line.chars().count() <= width
                        || tokens.iter().any(|token| token.chars().count() > width),
                    "width {width}, line {line:?}"
                );
            }
        }
    }

    #[test]
    fn repeated_rendering_is_deterministic_including_stats() {
        let document = Document::join(
            Document::line(),
            (0..100).map(|index| text(&format!("item_{index}"))),
        )
        .group();
        let first = render_with_stats(&document, options(40)).unwrap();
        for _ in 0..20 {
            assert_eq!(render_with_stats(&document, options(40)).unwrap(), first);
        }
        assert!(first.stats.nodes_visited > 100);
        assert!(first.stats.peak_output_capacity_bytes >= first.text.len());
    }

    #[test]
    fn depth_node_and_output_limits_are_structured_without_recursion() {
        let mut deep = text("leaf");
        for _ in 0..4_095 {
            deep = deep.indent(1);
        }
        assert_eq!(render(&deep, options(80)), Ok("leaf".to_owned()));
        assert_eq!(
            render(
                &deep,
                RenderOptions {
                    limits: RenderLimits {
                        max_depth: 4_095,
                        ..RenderLimits::default()
                    },
                    ..options(80)
                }
            ),
            Err(RenderError::DepthLimit { limit: 4_095 })
        );

        let nodes = Document::concat((0..10).map(|_| text("x")));
        assert_eq!(
            render(
                &nodes,
                RenderOptions {
                    limits: RenderLimits {
                        max_nodes: 5,
                        ..RenderLimits::default()
                    },
                    ..options(80)
                }
            ),
            Err(RenderError::NodeLimit { limit: 5 })
        );
        assert_eq!(
            render(
                &text("12345"),
                RenderOptions {
                    limits: RenderLimits {
                        max_output_bytes: 4,
                        ..RenderLimits::default()
                    },
                    ..options(80)
                }
            ),
            Err(RenderError::OutputLimit { limit: 4 })
        );
    }

    #[test]
    fn toy_indentation_sensitive_and_delimited_layouts_share_the_writer() {
        let indentation_sensitive = Document::concat([
            text("section:"),
            Document::concat([Document::hard_line(), text("item")]).indent(2),
        ]);
        assert_eq!(
            render(&indentation_sensitive, options(80)),
            Ok("section:\n  item".to_owned())
        );

        let delimited = Document::concat([
            text("section {"),
            Document::concat([Document::line(), text("item")]).indent(2),
            Document::line(),
            text("}"),
        ])
        .group();
        assert_eq!(
            render(&delimited, options(80)),
            Ok("section { item }".to_owned())
        );
        assert_eq!(
            render(&delimited, options(10)),
            Ok("section {\n  item\n}".to_owned())
        );
    }
}
