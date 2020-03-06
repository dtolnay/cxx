use crate::gen::include::Includes;
use std::fmt::{self, Arguments, Write};

pub(crate) struct OutFile {
    pub namespace: Vec<String>,
    pub header: bool,
    pub include: Includes,
    content: Vec<u8>,
    section_pending: bool,
    blocks_pending: Vec<&'static str>,
}

impl OutFile {
    pub fn new(namespace: Vec<String>, header: bool) -> Self {
        OutFile {
            namespace,
            header,
            include: Includes::new(),
            content: Vec::new(),
            section_pending: false,
            blocks_pending: Vec::new(),
        }
    }

    // Write a blank line if the preceding section had any contents.
    pub fn next_section(&mut self) {
        self.section_pending = true;
    }

    pub fn begin_block(&mut self, block: &'static str) {
        self.blocks_pending.push(block);
    }

    pub fn end_block(&mut self, block: &'static str) {
        if self.blocks_pending.pop().is_none() {
            self.content.extend_from_slice(b"} // ");
            self.content.extend_from_slice(block.as_bytes());
            self.content.push(b'\n');
            self.section_pending = true;
        }
    }

    pub fn prepend(&mut self, section: String) {
        self.content.splice(..0, section.into_bytes());
    }

    pub fn write_fmt(&mut self, args: Arguments) {
        Write::write_fmt(self, args).unwrap();
    }
}

impl Write for OutFile {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !s.is_empty() {
            if !self.blocks_pending.is_empty() {
                self.content.push(b'\n');
                for block in self.blocks_pending.drain(..) {
                    self.content.extend_from_slice(block.as_bytes());
                    self.content.extend_from_slice(b" {\n");
                }
                self.section_pending = false;
            } else if self.section_pending {
                self.content.push(b'\n');
                self.section_pending = false;
            }
            self.content.extend_from_slice(s.as_bytes());
        }
        Ok(())
    }
}

impl AsRef<[u8]> for OutFile {
    fn as_ref(&self) -> &[u8] {
        &self.content
    }
}
