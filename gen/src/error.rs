use crate::syntax;
use anyhow::anyhow;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream, WriteColor};
use codespan_reporting::term::{self, Config};
use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io::{self, Write};
use std::ops::Range;
use std::path::Path;
use std::process;

pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub(super) enum Error {
    NoBridgeMod,
    OutOfLineMod,
    Io(io::Error),
    Syn(syn::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NoBridgeMod => write!(f, "no #[cxx::bridge] module found"),
            Error::OutOfLineMod => write!(f, "#[cxx::bridge] module must have inline contents"),
            Error::Io(err) => err.fmt(f),
            Error::Syn(err) => err.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Syn(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<syn::Error> for Error {
    fn from(err: syn::Error) -> Self {
        Error::Syn(err)
    }
}

pub(super) fn format_err(path: &Path, source: &str, error: Error) -> ! {
    match error {
        Error::Syn(syn_error) => {
            let syn_error = sort_syn_errors(syn_error);
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

fn sort_syn_errors(error: syn::Error) -> Vec<syn::Error> {
    let mut errors: Vec<_> = error.into_iter().collect();
    errors.sort_by_key(|e| {
        let start = e.span().start();
        (start.line, start.column)
    });
    errors
}

fn display_syn_error(stderr: &mut dyn WriteColor, path: &Path, source: &str, error: syn::Error) {
    let span = error.span();
    let start = span.start();
    let end = span.end();

    let mut start_offset = 0;
    for _ in 1..start.line {
        start_offset += source[start_offset..].find('\n').unwrap() + 1;
    }
    let start_column = source[start_offset..]
        .chars()
        .take(start.column)
        .map(char::len_utf8)
        .sum::<usize>();
    start_offset += start_column;

    let mut end_offset = start_offset;
    if start.line == end.line {
        end_offset -= start_column;
    } else {
        for _ in 0..end.line - start.line {
            end_offset += source[end_offset..].find('\n').unwrap() + 1;
        }
    }
    end_offset += source[end_offset..]
        .chars()
        .take(end.column)
        .map(char::len_utf8)
        .sum::<usize>();

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
