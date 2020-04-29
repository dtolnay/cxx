use crate::gen::include::Includes;
use crate::syntax::namespace::Namespace;
use std::cell::RefCell;
use std::fmt::{self, Arguments, Write};

pub(crate) struct OutFile {
    pub namespace: Namespace,
    pub header: bool,
    pub include: Includes,
    content: RefCell<Content>,
}

struct Content {
    bytes: Vec<u8>,
    section_pending: bool,
    blocks_pending: Vec<&'static str>,
}

impl OutFile {
    pub fn new(namespace: Namespace, header: bool) -> Self {
        OutFile {
            namespace,
            header,
            include: Includes::new(),
            content: RefCell::new(Content {
                bytes: Vec::new(),
                section_pending: false,
                blocks_pending: Vec::new(),
            }),
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
            content.bytes.extend_from_slice(b"} // ");
            content.bytes.extend_from_slice(block.as_bytes());
            content.bytes.push(b'\n');
            content.section_pending = true;
        }
    }

    pub fn prepend(&mut self, section: String) {
        let content = self.content.get_mut();
        content.bytes.splice(..0, section.into_bytes());
    }

    pub fn write_fmt(&self, args: Arguments) {
        let content = &mut *self.content.borrow_mut();
        Write::write_fmt(content, args).unwrap();
    }

    pub fn content(&self) -> Vec<u8> {
        self.content.borrow().bytes.clone()
    }
}

impl Write for Content {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !s.is_empty() {
            if !self.blocks_pending.is_empty() {
                if !self.bytes.is_empty() {
                    self.bytes.push(b'\n');
                }
                for block in self.blocks_pending.drain(..) {
                    self.bytes.extend_from_slice(block.as_bytes());
                    self.bytes.extend_from_slice(b" {\n");
                }
                self.section_pending = false;
            } else if self.section_pending {
                if !self.bytes.is_empty() {
                    self.bytes.push(b'\n');
                }
                self.section_pending = false;
            }
            self.bytes.extend_from_slice(s.as_bytes());
        }
        Ok(())
    }
}
