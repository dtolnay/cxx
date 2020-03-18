use crate::gen::Error;
use crate::syntax;
use anyhow::anyhow;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
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

    let mut files = SimpleFiles::new();
    let file = files.add(path.to_string_lossy(), source);

    let diagnostic = diagnose(file, start_offset..end_offset, error);

    let config = Config::default();
    let _ = term::emit(stderr, &config, &files, &diagnostic);
}

fn diagnose(file: usize, range: Range<usize>, error: syn::Error) -> Diagnostic<usize> {
    let message = error.to_string();
    let info = syntax::error::ERRORS
        .iter()
        .find(|e| message.contains(e.msg));
    let mut diagnostic = Diagnostic::error().with_message(&message);
    let mut label = Label::primary(file, range);
    if let Some(info) = info {
        label.message = info.label.map_or(message, str::to_owned);
        diagnostic.labels.push(label);
        diagnostic.notes.extend(info.note.map(str::to_owned));
    } else {
        label.message = message;
        diagnostic.labels.push(label);
    }
    diagnostic.code = Some("cxxbridge".to_owned());
    diagnostic
}
