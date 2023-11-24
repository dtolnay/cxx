use core::char;
use core::fmt::{self, Write as _};
use core::str::Utf8Chunks;

pub(crate) fn display(bytes: &[u8], f: &mut fmt::Formatter) -> fmt::Result {
    for chunk in Utf8Chunks::new(bytes) {
        f.write_str(chunk.valid())?;
        if !chunk.invalid().is_empty() {
            f.write_char(char::REPLACEMENT_CHARACTER)?;
        }
    }
    Ok(())
}

pub(crate) fn debug(bytes: &[u8], f: &mut fmt::Formatter) -> fmt::Result {
    f.write_char('"')?;

    for chunk in Utf8Chunks::new(bytes) {
        let valid = chunk.valid();

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

        for b in chunk.invalid() {
            write!(f, "\\x{:02x}", b)?;
        }
    }

    f.write_char('"')
}
