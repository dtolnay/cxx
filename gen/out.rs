use std::fmt::{self, Arguments, Write};

pub(crate) struct OutFile {
    pub namespace: Vec<String>,
    pub header: bool,
    content: Vec<u8>,
    section_pending: bool,
    blocks: Vec<&'static str>,
    blocks_pending: usize,
}

impl OutFile {
    pub fn new(namespace: Vec<String>, header: bool) -> Self {
        OutFile {
            namespace,
            header,
            content: Vec::new(),
            section_pending: false,
            blocks: Vec::new(),
            blocks_pending: 0,
        }
    }

    // Write a blank line if the preceding section had any contents.
    pub fn next_section(&mut self) {
        self.section_pending = true;
    }

    pub fn begin_block(&mut self, block: &'static str) {
        self.blocks.push(block);
        self.blocks_pending += 1;
    }

    pub fn end_block(&mut self) {
        if self.blocks_pending > 0 {
            self.blocks_pending -= 1;
        } else {
            self.content.extend_from_slice(b"} // ");
            self.content
                .extend_from_slice(self.blocks.pop().unwrap().as_bytes());
            self.content.push(b'\n');
            self.section_pending = true;
        }
    }

    pub fn write_fmt(&mut self, args: Arguments) {
        Write::write_fmt(self, args).unwrap();
    }
}

impl Write for OutFile {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !s.is_empty() {
            if self.blocks_pending > 0 {
                self.content.push(b'\n');
                for block in &self.blocks[self.blocks.len() - self.blocks_pending..] {
                    self.content.extend_from_slice(block.as_bytes());
                    self.content.extend_from_slice(b" {\n");
                }
                self.blocks_pending = 0;
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
