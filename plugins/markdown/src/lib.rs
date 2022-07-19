use gal_bindings::*;
use pulldown_cmark::{Event::*, *};
use std::collections::HashMap;

#[export]
fn plugin_type() -> PluginType {
    PluginType::Action
}

#[export]
fn process_action(frontend: FrontendType, mut action: Action) -> Action {
    match frontend {
        FrontendType::Html => {
            let line = action
                .line
                .into_iter()
                .map(|s| s.into_string())
                .collect::<Vec<_>>()
                .concat();
            let parser = Parser::new(&line);
            action.line = HtmlWriter::new(parser).run().into_line()
        }
        _ => {}
    }
    action
}

// The below code are modified from pulldown_cmark

fn escape_html(s: &str) -> String {
    let mut buffer = String::new();
    pulldown_cmark::escape::escape_html(&mut buffer, s).unwrap();
    buffer
}

enum TableState {
    Head,
    Body,
}

struct HtmlWriter<'a, I> {
    iter: I,
    writer: Vec<ActionLine>,
    end_newline: bool,
    table_state: TableState,
    table_alignments: Vec<Alignment>,
    table_cell_index: usize,
    numbers: HashMap<CowStr<'a>, usize>,
}

impl<'a, I> HtmlWriter<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn new(iter: I) -> Self {
        Self {
            iter,
            writer: vec![],
            end_newline: true,
            table_state: TableState::Head,
            table_alignments: vec![],
            table_cell_index: 0,
            numbers: HashMap::new(),
        }
    }

    /// Writes a new line.
    fn write_newline(&mut self) {
        self.write_chars("\n");
    }

    /// Writes a buffer, and tracks whether or not a newline was written.
    fn write_chars(&mut self, s: impl Into<String>) {
        let s: String = s.into();
        if !s.is_empty() {
            self.end_newline = s.ends_with('\n');
        }
        self.writer.push(ActionLine::Chars(s));
    }

    fn write_block(&mut self, s: impl Into<String>) {
        let s: String = s.into();
        if !s.is_empty() {
            self.end_newline = s.ends_with('\n');
        }
        self.writer.push(ActionLine::Block(s));
    }

    fn run(mut self) -> Self {
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
                    self.end_newline = text.ends_with('\n');
                }
                Code(text) => {
                    self.write_block("<code>");
                    self.write_chars(escape_html(&text));
                    self.write_block("</code>");
                }
                Html(html) => {
                    self.write_block(html.into_string());
                }
                SoftBreak => {
                    self.write_newline();
                }
                HardBreak => {
                    self.write_block("<br />\n");
                }
                Rule => {
                    if self.end_newline {
                        self.write_block("<hr />\n");
                    } else {
                        self.write_block("\n<hr />\n");
                    }
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
                    self.write_block("<input disabled=\"\" type=\"checkbox\" checked=\"\"/>\n");
                }
                TaskListMarker(false) => {
                    self.write_block("<input disabled=\"\" type=\"checkbox\"/>\n");
                }
            }
        }
        self
    }

    /// Writes the start of an HTML tag.
    fn start_tag(&mut self, tag: Tag<'a>) {
        match tag {
            Tag::Paragraph => {
                if self.end_newline {
                    self.write_block("<p>")
                } else {
                    self.write_block("\n<p>")
                }
            }
            Tag::Heading(level, id, classes) => {
                if self.end_newline {
                    self.end_newline = false;
                    self.write_block("<");
                } else {
                    self.write_block("\n<");
                }
                self.write_block(level.to_string());
                if let Some(id) = id {
                    self.write_block(" id=\"");
                    self.write_block(escape_html(id));
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
            Tag::BlockQuote => {
                if self.end_newline {
                    self.write_block("<blockquote>\n")
                } else {
                    self.write_block("\n<blockquote>\n")
                }
            }
            Tag::CodeBlock(info) => {
                if !self.end_newline {
                    self.write_newline();
                }
                match info {
                    CodeBlockKind::Fenced(info) => {
                        let lang = info.split(' ').next().unwrap();
                        if lang.is_empty() {
                            self.write_block("<pre><code>")
                        } else {
                            self.write_block("<pre><code class=\"language-");
                            self.write_block(escape_html(lang));
                            self.write_block("\">")
                        }
                    }
                    CodeBlockKind::Indented => self.write_block("<pre><code>"),
                }
            }
            Tag::List(Some(1)) => {
                if self.end_newline {
                    self.write_block("<ol>\n")
                } else {
                    self.write_block("\n<ol>\n")
                }
            }
            Tag::List(Some(start)) => {
                if self.end_newline {
                    self.write_block("<ol start=\"");
                } else {
                    self.write_block("\n<ol start=\"");
                }
                self.write_chars(start.to_string());
                self.write_block("\">\n")
            }
            Tag::List(None) => {
                if self.end_newline {
                    self.write_block("<ul>\n")
                } else {
                    self.write_block("\n<ul>\n")
                }
            }
            Tag::Item => {
                if self.end_newline {
                    self.write_block("<li>")
                } else {
                    self.write_block("\n<li>")
                }
            }
            Tag::Emphasis => self.write_block("<em>"),
            Tag::Strong => self.write_block("<strong>"),
            Tag::Strikethrough => self.write_block("<del>"),
            Tag::Link(LinkType::Email, dest, title) => {
                self.write_block("<a href=\"mailto:");
                self.write_block(escape_html(&dest));
                if !title.is_empty() {
                    self.write_block("\" title=\"");
                    self.write_block(escape_html(&title));
                }
                self.write_block("\">")
            }
            Tag::Link(_link_type, dest, title) => {
                self.write_block("<a href=\"");
                self.write_block(escape_html(&dest));
                if !title.is_empty() {
                    self.write_block("\" title=\"");
                    self.write_block(escape_html(&title));
                }
                self.write_block("\">")
            }
            Tag::Image(_link_type, dest, title) => {
                self.write_block("<img src=\"");
                self.write_block(escape_html(&dest));
                self.write_block("\" alt=\"");
                self.raw_text();
                if !title.is_empty() {
                    self.write_block("\" title=\"");
                    self.write_block(escape_html(&title));
                }
                self.write_block("\" />")
            }
            Tag::FootnoteDefinition(name) => {
                if self.end_newline {
                    self.write_block("<div class=\"footnote-definition\" id=\"");
                } else {
                    self.write_block("\n<div class=\"footnote-definition\" id=\"");
                }
                self.write_block(escape_html(&name));
                self.write_block("\"><sup class=\"footnote-definition-label\">");
                let len = self.numbers.len() + 1;
                let number = *self.numbers.entry(name).or_insert(len);
                self.write_chars(number.to_string());
                self.write_block("</sup>")
            }
        }
    }

    fn end_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => {
                self.write_block("</p>\n");
            }
            Tag::Heading(level, _id, _classes) => {
                self.write_block("</");
                self.write_block(level.to_string());
                self.write_block(">\n");
            }
            Tag::Table(_) => {
                self.write_block("</tbody></table>\n");
            }
            Tag::TableHead => {
                self.write_block("</tr></thead><tbody>\n");
                self.table_state = TableState::Body;
            }
            Tag::TableRow => {
                self.write_block("</tr>\n");
            }
            Tag::TableCell => {
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
            Tag::BlockQuote => {
                self.write_block("</blockquote>\n");
            }
            Tag::CodeBlock(_) => {
                self.write_block("</code></pre>\n");
            }
            Tag::List(Some(_)) => {
                self.write_block("</ol>\n");
            }
            Tag::List(None) => {
                self.write_block("</ul>\n");
            }
            Tag::Item => {
                self.write_block("</li>\n");
            }
            Tag::Emphasis => {
                self.write_block("</em>");
            }
            Tag::Strong => {
                self.write_block("</strong>");
            }
            Tag::Strikethrough => {
                self.write_block("</del>");
            }
            Tag::Link(_, _, _) => {
                self.write_block("</a>");
            }
            Tag::Image(_, _, _) => (), // shouldn't happen, handled in start
            Tag::FootnoteDefinition(_) => {
                self.write_block("</div>\n");
            }
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
                Html(text) | Code(text) | Text(text) => {
                    self.write_chars(escape_html(&text));
                    self.end_newline = text.ends_with('\n');
                }
                SoftBreak | HardBreak | Rule => {
                    self.write_chars(" ");
                }
                FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    let number = *self.numbers.entry(name).or_insert(len);
                    self.write_chars(format!("[{}]", number));
                }
                TaskListMarker(true) => self.write_chars("[x]"),
                TaskListMarker(false) => self.write_chars("[ ]"),
            }
        }
    }

    pub fn into_line(self) -> Vec<ActionLine> {
        self.writer
    }
}
