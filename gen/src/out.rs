use crate::gen::block::Block;
use crate::gen::builtin::Builtins;
use crate::gen::include::Includes;
use crate::gen::Opt;
use crate::syntax::Types;
use std::cell::RefCell;
use std::fmt::{self, Arguments, Write};

pub(crate) struct OutFile<'a> {
    pub header: bool,
    pub opt: &'a Opt,
    pub types: &'a Types<'a>,
    pub include: Includes<'a>,
    pub builtin: Builtins<'a>,
    content: RefCell<Content<'a>>,
}

#[derive(Default)]
pub struct Content<'a> {
    bytes: String,
    blocks: Vec<Block<'a>>,
    section_pending: bool,
    blocks_pending: usize,
}

impl<'a> OutFile<'a> {
    pub fn new(header: bool, opt: &'a Opt, types: &'a Types) -> Self {
        OutFile {
            header,
            opt,
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

    pub fn begin_block(&mut self, block: Block<'a>) {
        self.content.get_mut().begin_block(block);
    }

    pub fn end_block(&mut self, block: Block<'a>) {
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

impl<'a> Write for Content<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}

impl<'a> PartialEq for Content<'a> {
    fn eq(&self, _other: &Content) -> bool {
        true
    }
}

impl<'a> Content<'a> {
    fn new() -> Self {
        Content::default()
    }

    pub fn next_section(&mut self) {
        self.section_pending = true;
    }

    pub fn begin_block(&mut self, block: Block<'a>) {
        self.blocks.push(block);
        self.blocks_pending += 1;
    }

    pub fn end_block(&mut self, block: Block<'a>) {
        let begin_block = self.blocks.pop().unwrap();
        let end_block = block;
        assert_eq!(begin_block, end_block);

        if self.blocks_pending > 0 {
            self.blocks_pending -= 1;
        } else {
            Block::write_end(block, &mut self.bytes);
            self.section_pending = true;
        }
    }

    pub fn write_fmt(&mut self, args: Arguments) {
        Write::write_fmt(self, args).unwrap();
    }

    fn write(&mut self, b: &str) {
        if !b.is_empty() {
            if self.blocks_pending > 0 {
                if !self.bytes.is_empty() {
                    self.bytes.push('\n');
                }
                let pending = self.blocks.len() - self.blocks_pending..;
                for block in &self.blocks[pending] {
                    Block::write_begin(*block, &mut self.bytes);
                }
            } else if self.section_pending && !self.bytes.is_empty() {
                self.bytes.push('\n');
            }
            self.bytes.push_str(b);
            self.section_pending = false;
            self.blocks_pending = 0;
        }
    }
}
