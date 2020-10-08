use crate::gen::include::Includes;
use crate::syntax::namespace::Namespace;
use std::cell::RefCell;
use std::fmt::{self, Arguments, Write};

pub(crate) struct OutFile {
    pub namespace: Namespace,
    pub header: bool,
    pub include: Includes,
    pub front: Content,
    content: RefCell<Content>,
}

pub struct Content {
    bytes: String,
    section_pending: bool,
    blocks_pending: Vec<&'static str>,
}

impl OutFile {
    pub fn new(namespace: Namespace, header: bool) -> Self {
        OutFile {
            namespace,
            header,
            include: Includes::new(),
            front: Content::new(),
            content: RefCell::new(Content::new()),
        }
    }

    // Write a blank line if the preceding section had any contents.
    pub fn next_section(&mut self) {
        let content = self.content.get_mut();
        content.section_pending = true;
    }

    pub fn begin_block(&mut self, block: &'static str) {
        let content = self.content.get_mut();
        content.blocks_pending.push(block);
    }

    pub fn end_block(&mut self, block: &'static str) {
        let content = self.content.get_mut();
        if content.blocks_pending.pop().is_none() {
            content.bytes.push_str("} // ");
            content.bytes.push_str(block);
            content.bytes.push('\n');
            content.section_pending = true;
        }
    }

    pub fn write_fmt(&self, args: Arguments) {
        let content = &mut *self.content.borrow_mut();
        Write::write_fmt(content, args).unwrap();
    }

    pub fn content(&self) -> Vec<u8> {
        let front = &self.front.bytes;
        let content = &self.content.borrow().bytes;
        let len = front.len() + content.len() + 1;
        let mut out = String::with_capacity(len);
        out.push_str(front);
        if !front.is_empty() && !content.is_empty() {
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

impl Content {
    pub fn write_fmt(&mut self, args: Arguments) {
        Write::write_fmt(self, args).unwrap();
    }

    fn new() -> Self {
        Content {
            bytes: String::new(),
            section_pending: false,
            blocks_pending: Vec::new(),
        }
    }

    fn write(&mut self, b: &str) {
        if !b.is_empty() {
            if !self.blocks_pending.is_empty() {
                if !self.bytes.is_empty() {
                    self.bytes.push('\n');
                }
                for block in self.blocks_pending.drain(..) {
                    self.bytes.push_str(block);
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
