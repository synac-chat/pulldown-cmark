// Copyright 2015 Google Inc. All rights reserved.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

// This is a copy of pulldown_cmark's html.rs, modified to work in GTK.

//! HTML renderer that takes an iterator of events as input.

use std::borrow::Cow;
use std::collections::HashMap;

use escape::{escape_html, escape_href};
use parse::Event::{Start, End, Text, Html, InlineHtml, SoftBreak, HardBreak, FootnoteReference};
use parse::{Event, Tag};
use linkify::{LinkFinder, LinkKind};

struct Ctx<'b, I> {
    iter: I,
    buf: &'b mut String
}

impl<'a, 'b, I: Iterator<Item=Event<'a>>> Ctx<'b, I> {
    fn fresh_line(&mut self) {
        if !(self.buf.is_empty() || self.buf.ends_with('\n')) {
            self.buf.push('\n');
        }
    }

    pub fn run(&mut self) {
        let mut numbers = HashMap::new();
        while let Some(event) = self.iter.next() {
            match event {
                Start(tag) => self.start_tag(tag, &mut numbers),
                End(tag) => self.end_tag(tag),
                Text(text) => {
                    let mut finder = LinkFinder::new();
                    finder.kinds(&[LinkKind::Url]);
                    let mut appended = 0;

                    let mut escaped = String::with_capacity(text.len());
                    escape_html(&mut escaped, &text, false);

                    for link in finder.links(&escaped.clone()) {
                        let mut string = String::with_capacity(9 + (link.end() - link.start()) + 2);
                        string.push_str("<a href=\"");
                        escape_href(&mut string, link.as_str());
                        string.push_str("\">");

                        escaped.reserve(string.len() + 4);
                        escaped.insert_str(appended + link.end(), "</a>");
                        escaped.insert_str(appended + link.start(), &string);
                        appended += string.len() + 4;
                    }
                    self.buf.push_str(&escaped);
                }
                Html(html) |
                InlineHtml(html) => self.buf.push_str(&html),
                SoftBreak => self.buf.push('\n'),
                HardBreak => self.buf.push_str("<br />\n"),
                FootnoteReference(name) => {
                    let len = numbers.len() + 1;
                    self.buf.push_str("<sup class=\"footnote-reference\"><a href=\"#");
                    escape_html(self.buf, &*name, false);
                    self.buf.push_str("\">");
                    let number = numbers.entry(name).or_insert(len);
                    self.buf.push_str(&*format!("{}", number));
                    self.buf.push_str("</a></sup>");
                },
            }
        }
    }

    fn start_tag(&mut self, tag: Tag<'a>, _numbers: &mut HashMap<Cow<'a, str>, usize>) {
        match tag {
            Tag::Header(_) => {
                self.fresh_line();
                self.buf.push_str("<big>");
            }
            Tag::CodeBlock(_) => {
                self.fresh_line();
                self.buf.push_str("<tt>");
            }
            Tag::Emphasis => self.buf.push_str("<i>"),
            Tag::Strong => self.buf.push_str("<b>"),
            Tag::Code => self.buf.push_str("<tt>"),
            Tag::Link(dest, title) => {
                self.buf.push_str("<a href=\"");
                escape_href(self.buf, &dest);
                if !title.is_empty() {
                    self.buf.push_str("\" title=\"");
                    escape_html(self.buf, &title, false);
                }
                self.buf.push_str("\">");
            }
            _ => ()
        }
    }

    fn end_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Header(_) => self.buf.push_str("</big>"),
            Tag::CodeBlock(_) => self.buf.push_str("</tt>\n"),
            Tag::Emphasis => self.buf.push_str("</i>"),
            Tag::Strong => self.buf.push_str("</b>"),
            Tag::Code => self.buf.push_str("</tt>"),
            Tag::Link(_, _) => self.buf.push_str("</a>"),
            _ => ()
        }
    }
}

/// Iterate over an `Iterator` of `Event`s, generate HTML for each `Event`, and
/// push it to a `String`.
///
/// # Examples
///
/// ```
/// use pulldown_cmark::{html, Parser};
///
/// let markdown_str = r#"
/// hello
/// =====
///
/// * alpha
/// * beta
/// "#;
/// let parser = Parser::new(markdown_str);
///
/// let mut html_buf = String::new();
/// html::push_html(&mut html_buf, parser);
///
/// assert_eq!(html_buf, r#"<h1>hello</h1>
/// <ul>
/// <li>alpha</li>
/// <li>beta</li>
/// </ul>
/// "#);
/// ```
pub fn push_html<'a, I: Iterator<Item=Event<'a>>>(buf: &mut String, iter: I) {
    let mut ctx = Ctx {
        iter: iter,
        buf: buf
    };
    ctx.run();
}
