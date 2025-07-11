#![deny(unsafe_code)]

use ayaka_bindings::*;
use latex2mathml::{latex_to_mathml, DisplayStyle};
use pulldown_cmark::{Event::*, *};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
};

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder().action().build()
}

#[export]
fn process_action(mut ctx: ActionProcessContext) -> ActionProcessResult {
    let line = ctx
        .action
        .text
        .into_iter()
        .map(|s| s.into_string())
        .collect::<Vec<_>>()
        .concat();
    let parser = Parser::new_ext(&line, Options::all());
    let writer = Writer::new(parser);
    ctx.action.text = match ctx.frontend {
        FrontendType::Html => writer.run_html().into_lines(),
        FrontendType::Text => writer.run_text().into_lines(),
        FrontendType::Latex => writer.run_latex().into_lines(),
    };
    ActionProcessResult { action: ctx.action }
}

// The below code are modified from pulldown_cmark

fn escape_html(s: &str) -> String {
    let mut buffer = String::new();
    pulldown_cmark_escape::escape_html(&mut buffer, s).expect("cannot write escaped HTML");
    buffer
}

enum TableState {
    Head,
    Body,
}

struct Writer<'a, I> {
    iter: I,
    writer: ActionText,
    table_state: TableState,
    table_alignments: Vec<Alignment>,
    table_cell_index: usize,
    numbers: HashMap<CowStr<'a>, usize>,
}

impl<'a, I> Writer<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn new(iter: I) -> Self {
        Self {
            iter,
            writer: ActionText::default(),
            table_state: TableState::Head,
            table_alignments: vec![],
            table_cell_index: 0,
            numbers: HashMap::new(),
        }
    }

    /// Writes a buffer, and tracks whether or not a newline was written.
    fn write_chars<'s>(&mut self, s: impl Into<Cow<'s, str>>) {
        self.writer.push_back_chars(s);
    }

    fn write_block<'s>(&mut self, s: impl Into<Cow<'s, str>>) {
        self.writer.push_back_block(s);
    }

    fn run_text(mut self) -> Self {
        while let Some(event) = self.iter.next() {
            match event {
                Text(text) | Code(text) | Html(text) => self.write_chars(text.as_ref()),
                SoftBreak | HardBreak | Rule => self.write_chars("\n"),
                _ => {}
            }
        }
        self
    }

    fn run_html(mut self) -> Self {
        while let Some(event) = self.iter.next() {
            match event {
                Start(tag) => {
                    self.start_tag(tag);
                }
                End(tag) => {
                    self.end_tag(tag);
                }
                Text(text) => {
                    self.write_chars(escape_html(&text));
                }
                Code(text) => {
                    self.write_block("<code>");
                    self.write_chars(escape_html(&text));
                    self.write_block("</code>");
                }
                Html(html) | InlineHtml(html) => {
                    self.write_block(html);
                }
                SoftBreak => {
                    self.write_chars("\n");
                }
                HardBreak | Rule => {
                    self.write_block("<br />");
                }
                FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    self.write_block("<sup class=\"footnote-reference\"><a href=\"#");
                    self.write_block(escape_html(&name));
                    self.write_block("\">");
                    let number = *self.numbers.entry(name).or_insert(len);
                    self.write_chars(number.to_string());
                    self.write_block("</a></sup>");
                }
                TaskListMarker(true) => {
                    self.write_block("<input disabled=\"\" type=\"checkbox\" checked=\"\"/>");
                }
                TaskListMarker(false) => {
                    self.write_block("<input disabled=\"\" type=\"checkbox\"/>");
                }
                InlineMath(text) => {
                    if let Ok(mathml) = latex_to_mathml(&text, DisplayStyle::Inline) {
                        self.write_block(mathml);
                    } else {
                        self.write_block(text)
                    }
                }
                DisplayMath(text) => {
                    if let Ok(mathml) = latex_to_mathml(&text, DisplayStyle::Block) {
                        self.write_block(mathml);
                    } else {
                        self.write_block(text)
                    }
                }
            }
        }
        self
    }

    /// Writes the start of an HTML tag.
    fn start_tag(&mut self, tag: Tag<'a>) {
        match tag {
            Tag::Paragraph => self.write_block("<p>"),
            Tag::Heading {
                level, id, classes, ..
            } => {
                self.write_block("<");
                self.write_block(level.to_string());
                if let Some(id) = id {
                    self.write_block(" id=\"");
                    self.write_block(escape_html(&id));
                    self.write_block("\"");
                }
                let mut classes = classes.iter();
                if let Some(class) = classes.next() {
                    self.write_block(" class=\"");
                    self.write_block(escape_html(class));
                    for class in classes {
                        self.write_block(" ");
                        self.write_block(escape_html(class));
                    }
                    self.write_block("\"");
                }
                self.write_block(">")
            }
            Tag::Table(alignments) => {
                self.table_alignments = alignments;
                self.write_block("<table>")
            }
            Tag::TableHead => {
                self.table_state = TableState::Head;
                self.table_cell_index = 0;
                self.write_block("<thead><tr>")
            }
            Tag::TableRow => {
                self.table_cell_index = 0;
                self.write_block("<tr>")
            }
            Tag::TableCell => {
                match self.table_state {
                    TableState::Head => {
                        self.write_block("<th");
                    }
                    TableState::Body => {
                        self.write_block("<td");
                    }
                }
                match self.table_alignments.get(self.table_cell_index) {
                    Some(&Alignment::Left) => self.write_block(" style=\"text-align: left\">"),
                    Some(&Alignment::Center) => self.write_block(" style=\"text-align: center\">"),
                    Some(&Alignment::Right) => self.write_block(" style=\"text-align: right\">"),
                    _ => self.write_block(">"),
                }
            }
            Tag::BlockQuote(_) => self.write_block("<blockquote>"),
            Tag::CodeBlock(info) => match info {
                CodeBlockKind::Fenced(info) => {
                    let lang = info.split(' ').next().unwrap_or_default();
                    if lang.is_empty() {
                        self.write_block("<pre><code>")
                    } else {
                        self.write_block("<pre><code class=\"language-");
                        self.write_block(escape_html(lang));
                        self.write_block("\">")
                    }
                }
                CodeBlockKind::Indented => self.write_block("<pre><code>"),
            },
            Tag::List(Some(1)) => self.write_block("<ol>"),
            Tag::List(Some(start)) => {
                self.write_block("<ol start=\"");
                self.write_chars(start.to_string());
                self.write_block("\">")
            }
            Tag::List(None) => self.write_block("<ul>"),
            Tag::Item => self.write_block("<li>"),
            Tag::Emphasis => self.write_block("<em>"),
            Tag::Strong => self.write_block("<strong>"),
            Tag::Strikethrough => self.write_block("<del>"),
            Tag::Link {
                link_type,
                dest_url,
                title,
                ..
            } => {
                match link_type {
                    LinkType::Email => self.write_block("<a href=\"mailto:"),
                    _ => self.write_block("<a href=\""),
                }
                self.write_block(escape_html(&dest_url));
                if !title.is_empty() {
                    self.write_block("\" title=\"");
                    self.write_block(escape_html(&title));
                }
                self.write_block("\">")
            }
            Tag::Image {
                dest_url, title, ..
            } => {
                self.write_block("<img src=\"");
                self.write_block(escape_html(&dest_url));
                self.write_block("\" alt=\"");
                self.raw_text();
                if !title.is_empty() {
                    self.write_block("\" title=\"");
                    self.write_block(escape_html(&title));
                }
                self.write_block("\" />")
            }
            Tag::FootnoteDefinition(name) => {
                self.write_block("<div class=\"footnote-definition\" id=\"");
                self.write_block(escape_html(&name));
                self.write_block("\"><sup class=\"footnote-definition-label\">");
                let len = self.numbers.len() + 1;
                let number = *self.numbers.entry(name).or_insert(len);
                self.write_chars(number.to_string());
                self.write_block("</sup>")
            }
            Tag::HtmlBlock => {}
            Tag::DefinitionList => self.write_block("<dl>"),
            Tag::DefinitionListTitle => self.write_block("<dt>"),
            Tag::DefinitionListDefinition => self.write_block("<dd>"),
            Tag::MetadataBlock(_) => {}
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                self.write_block("</p>");
            }
            TagEnd::Heading(level) => {
                self.write_block("</");
                self.write_block(level.to_string());
                self.write_block(">");
            }
            TagEnd::Table => {
                self.write_block("</tbody></table>");
            }
            TagEnd::TableHead => {
                self.write_block("</tr></thead><tbody>");
                self.table_state = TableState::Body;
            }
            TagEnd::TableRow => {
                self.write_block("</tr>");
            }
            TagEnd::TableCell => {
                match self.table_state {
                    TableState::Head => {
                        self.write_block("</th>");
                    }
                    TableState::Body => {
                        self.write_block("</td>");
                    }
                }
                self.table_cell_index += 1;
            }
            TagEnd::BlockQuote(_) => {
                self.write_block("</blockquote>");
            }
            TagEnd::CodeBlock => {
                self.write_block("</code></pre>");
            }
            TagEnd::List(true) => {
                self.write_block("</ol>");
            }
            TagEnd::List(false) => {
                self.write_block("</ul>");
            }
            TagEnd::Item => {
                self.write_block("</li>");
            }
            TagEnd::Emphasis => {
                self.write_block("</em>");
            }
            TagEnd::Strong => {
                self.write_block("</strong>");
            }
            TagEnd::Strikethrough => {
                self.write_block("</del>");
            }
            TagEnd::Link => {
                self.write_block("</a>");
            }
            TagEnd::Image => {}
            TagEnd::FootnoteDefinition => {
                self.write_block("</div>");
            }
            TagEnd::HtmlBlock => {}
            TagEnd::DefinitionList => self.write_block("</dl>"),
            TagEnd::DefinitionListTitle => self.write_block("</dt>"),
            TagEnd::DefinitionListDefinition => self.write_block("</dd>"),
            TagEnd::MetadataBlock(_) => {}
            _ => {}
        }
    }

    // run raw text, consuming end tag
    fn raw_text(&mut self) {
        let mut nest = 0;
        while let Some(event) = self.iter.next() {
            match event {
                Start(_) => nest += 1,
                End(_) => {
                    if nest == 0 {
                        break;
                    }
                    nest -= 1;
                }
                Html(text) | InlineHtml(text) | Code(text) | Text(text) => {
                    self.write_chars(escape_html(&text));
                }
                SoftBreak | HardBreak | Rule => {
                    self.write_chars(" ");
                }
                FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    let number = *self.numbers.entry(name).or_insert(len);
                    self.write_chars(format!("[{number}]"));
                }
                TaskListMarker(true) => self.write_chars("[x]"),
                TaskListMarker(false) => self.write_chars("[ ]"),
                InlineMath(text) | DisplayMath(text) => self.write_block(text),
            }
        }
    }

    pub fn run_latex(mut self) -> Self {
        while let Some(event) = self.iter.next() {
            match event {
                Text(text) => {
                    self.write_chars(text);
                }
                Code(text) => {
                    self.write_block("\\texttt{");
                    self.write_chars(text);
                    self.write_block("}");
                }
                SoftBreak => {
                    self.write_chars("\n");
                }
                HardBreak | Rule => {
                    self.write_block("\\par ");
                }
                InlineMath(text) => {
                    self.write_block("\\(");
                    self.write_block(text);
                    self.write_block("\\)");
                }
                DisplayMath(text) => {
                    self.write_block("\\[");
                    self.write_block(text);
                    self.write_block("\\]");
                }
                _ => {}
            }
        }
        self
    }

    pub fn into_lines(self) -> VecDeque<ActionSubText> {
        self.writer.text
    }
}
