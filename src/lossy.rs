use self::utf8_chunks::Utf8Chunks;
use core::char;
use core::fmt::{self, Write as _};

mod utf8_chunks {
    use core::str;

    pub struct Utf8Chunks<'a> {
        rest: &'a [u8],
    }

    impl<'a> Utf8Chunks<'a> {
        pub fn new(bytes: &'a [u8]) -> Self {
            Utf8Chunks { rest: bytes }
        }

        pub fn is_empty(&self) -> bool {
            self.rest.is_empty()
        }

        pub fn next_valid(&mut self) -> &'a str {
            let valid = match str::from_utf8(self.rest) {
                Ok(valid) => valid,
                Err(utf8_error) => {
                    let valid_up_to = utf8_error.valid_up_to();
                    unsafe { str::from_utf8_unchecked(&self.rest[..valid_up_to]) }
                }
            };
            self.rest = &self.rest[valid.len()..];
            valid
        }

        pub fn next_invalid(&mut self) -> &'a [u8] {
            let invalid = if let Err(utf8_error) = str::from_utf8(self.rest) {
                if utf8_error.valid_up_to() > 0 {
                    &[]
                } else if let Some(error_len) = utf8_error.error_len() {
                    &self.rest[..error_len]
                } else {
                    self.rest
                }
            } else {
                &[]
            };
            self.rest = &self.rest[invalid.len()..];
            invalid
        }
    }
}

pub(crate) fn display(bytes: &[u8], f: &mut fmt::Formatter) -> fmt::Result {
    let mut chunks = Utf8Chunks::new(bytes);
    while !chunks.is_empty() {
        f.write_str(chunks.next_valid())?;
        if !chunks.next_invalid().is_empty() {
            f.write_char(char::REPLACEMENT_CHARACTER)?;
        }
    }
    Ok(())
}

pub(crate) fn debug(bytes: &[u8], f: &mut fmt::Formatter) -> fmt::Result {
    f.write_char('"')?;

    let mut chunks = Utf8Chunks::new(bytes);
    while !chunks.is_empty() {
        let valid = chunks.next_valid();

        let mut written = 0;
        for (i, ch) in valid.char_indices() {
            let esc = ch.escape_debug();
            if esc.len() != 1 && ch != '\'' {
                f.write_str(&valid[written..i])?;
                for ch in esc {
                    f.write_char(ch)?;
                }
                written = i + ch.len_utf8();
            }
        }
        f.write_str(&valid[written..])?;

        for b in chunks.next_invalid() {
            write!(f, "\\x{:02x}", b)?;
        }
    }

    f.write_char('"')
}
