use crate::gen::out::{Content, OutFile};
use std::collections::BTreeSet;

#[derive(Default)]
pub(crate) struct Pragma<'a> {
    pub diagnostic_ignore: BTreeSet<&'a str>,
    pub begin: Content<'a>,
    pub end: Content<'a>,
}

impl<'a> Pragma<'a> {
    pub fn new() -> Self {
        Pragma::default()
    }
}

pub(super) fn write(out: &mut OutFile) {
    if out.pragma.diagnostic_ignore.is_empty() {
        return;
    }

    let begin = &mut out.pragma.begin;
    writeln!(begin, "#ifdef __GNUC__");
    if out.header {
        writeln!(begin, "#pragma GCC diagnostic push");
    }
    for diag in &out.pragma.diagnostic_ignore {
        writeln!(begin, "#pragma GCC diagnostic ignored \"{diag}\"");
    }
    writeln!(begin, "#endif");

    if out.header {
        let end = &mut out.pragma.end;
        writeln!(end, "#ifdef __GNUC__");
        writeln!(end, "#pragma GCC diagnostic pop");
        writeln!(end, "#endif");
    }
}
