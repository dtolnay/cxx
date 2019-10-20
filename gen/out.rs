use std::fmt::{self, Arguments, Write};

pub(crate) struct OutFile {
    pub namespace: Vec<String>,
    pub header: bool,
    content: Vec<u8>,
    section_pending: bool,
    block: &'static str,
    block_pending: bool,
}

impl OutFile {
    pub fn new(namespace: Vec<String>, header: bool) -> Self {
        OutFile {
            namespace,
            header,
            content: Vec::new(),
            section_pending: false,
            block: "",
            block_pending: false,
        }
    }

    // Write a blank line if the preceding section had any contents.
    pub fn next_section(&mut self) {
        self.section_pending = true;
    }

    pub fn begin_block(&mut self, block: &'static str) {
        self.block = block;
        self.block_pending = true;
    }

    pub fn end_block(&mut self) {
        if self.block_pending {
            self.block_pending = false;
        } else {
            self.content.extend_from_slice(b"} // ");
            self.content.extend_from_slice(self.block.as_bytes());
            self.content.push(b'\n');
            self.block = "";
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
            if self.block_pending {
                self.content.push(b'\n');
                self.content.extend_from_slice(self.block.as_bytes());
                self.content.extend_from_slice(b" {\n");
                self.block_pending = false;
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
