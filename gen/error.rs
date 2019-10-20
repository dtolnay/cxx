use crate::gen::Error;
use crate::syntax;
use anyhow::anyhow;
use codespan::{FileId, Files};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream, WriteColor};
use codespan_reporting::term::{self, Config};
use std::io::Write;
use std::ops::Range;
use std::path::Path;
use std::process;

pub(super) fn format_err(path: &Path, source: &str, error: Error) -> ! {
    match error {
        Error::Syn(syn_error) => {
            let writer = StandardStream::stderr(ColorChoice::Auto);
            let ref mut stderr = writer.lock();
            for error in syn_error {
                let _ = writeln!(stderr);
                display_syn_error(stderr, path, source, error);
            }
        }
        _ => eprintln!("cxxbridge: {:?}", anyhow!(error)),
    }
    process::exit(1);
}

fn display_syn_error(stderr: &mut dyn WriteColor, path: &Path, source: &str, error: syn::Error) {
    let span = error.span();
    let start = span.start();
    let end = span.end();

    let mut start_offset = 0;
    for _ in 1..start.line {
        start_offset += source[start_offset..].find('\n').unwrap() + 1;
    }
    start_offset += start.column;

    let mut end_offset = start_offset;
    if start.line == end.line {
        end_offset -= start.column;
    } else {
        for _ in 0..end.line - start.line {
            end_offset += source[end_offset..].find('\n').unwrap() + 1;
        }
    }
    end_offset += end.column;

    let mut files = Files::new();
    let file = files.add(path.to_string_lossy(), source);

    let range = start_offset as u32..end_offset as u32;
    let diagnostic = diagnose(file, range, error);

    let config = Config::default();
    let _ = term::emit(stderr, &config, &files, &diagnostic);
}

fn diagnose(file: FileId, range: Range<u32>, error: syn::Error) -> Diagnostic {
    let message = error.to_string();
    let info = syntax::error::ERRORS
        .iter()
        .find(|e| message.contains(e.msg));
    let mut diagnostic = if let Some(info) = info {
        let label = Label::new(file, range, info.label.unwrap_or(&message));
        let mut diagnostic = Diagnostic::new_error(&message, label);
        if let Some(note) = info.note {
            diagnostic = diagnostic.with_notes(vec![note.to_owned()]);
        }
        diagnostic
    } else {
        let label = Label::new(file, range, &message);
        Diagnostic::new_error(&message, label)
    };
    diagnostic.code = Some("cxxbridge".to_owned());
    diagnostic
}
