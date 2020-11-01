use crate::gen::builtin::Builtins;
use crate::gen::include::Includes;
use crate::syntax::Types;
use std::cell::RefCell;
use std::fmt::{self, Arguments, Write};

pub(crate) struct OutFile<'a> {
    pub header: bool,
    pub types: &'a Types<'a>,
    pub include: Includes,
    pub builtin: Builtins,
    content: RefCell<Content>,
}

#[derive(Default)]
pub struct Content {
    bytes: String,
    section_pending: bool,
    blocks_pending: Vec<String>,
}

impl<'a> OutFile<'a> {
    pub fn new(header: bool, types: &'a Types) -> Self {
        OutFile {
            header,
            types,
            include: Includes::new(),
            builtin: Builtins::new(),
            content: RefCell::new(Content::new()),
        }
    }

    // Write a blank line if the preceding section had any contents.
    pub fn next_section(&mut self) {
        self.content.get_mut().next_section();
    }

    pub fn begin_block(&mut self, block: &str) {
        self.content.get_mut().begin_block(block);
    }

    pub fn end_block(&mut self, block: &str) {
        self.content.get_mut().end_block(block);
    }

    pub fn write_fmt(&self, args: Arguments) {
        let content = &mut *self.content.borrow_mut();
        Write::write_fmt(content, args).unwrap();
    }

    pub fn content(&self) -> Vec<u8> {
        let include = &self.include.content.bytes;
        let builtin = &self.builtin.content.bytes;
        let content = &self.content.borrow().bytes;
        let len = include.len() + builtin.len() + content.len() + 2;
        let mut out = String::with_capacity(len);
        out.push_str(include);
        if !out.is_empty() && !builtin.is_empty() {
            out.push('\n');
        }
        out.push_str(builtin);
        if !out.is_empty() && !content.is_empty() {
            out.push('\n');
        }
        out.push_str(content);
        if out.is_empty() {
            out.push_str("// empty\n");
        }
        out.into_bytes()
    }
}

impl Write for Content {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}

impl PartialEq for Content {
    fn eq(&self, _other: &Content) -> bool {
        true
    }
}

impl Content {
    fn new() -> Self {
        Content::default()
    }

    pub fn next_section(&mut self) {
        self.section_pending = true;
    }

    pub fn begin_block(&mut self, block: &str) {
        self.blocks_pending.push(block.to_owned());
    }

    pub fn end_block(&mut self, block: &str) {
        if self.blocks_pending.pop().is_none() {
            self.bytes.push_str("} // ");
            self.bytes.push_str(block);
            self.bytes.push('\n');
            self.section_pending = true;
        }
    }

    pub fn write_fmt(&mut self, args: Arguments) {
        Write::write_fmt(self, args).unwrap();
    }

    fn write(&mut self, b: &str) {
        if !b.is_empty() {
            if !self.blocks_pending.is_empty() {
                if !self.bytes.is_empty() {
                    self.bytes.push('\n');
                }
                for block in self.blocks_pending.drain(..) {
                    self.bytes.push_str(&block);
                    self.bytes.push_str(" {\n");
                }
                self.section_pending = false;
            } else if self.section_pending {
                if !self.bytes.is_empty() {
                    self.bytes.push('\n');
                }
                self.section_pending = false;
            }
            self.bytes.push_str(b);
        }
    }
}
