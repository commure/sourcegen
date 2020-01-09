use proc_macro2::{LineColumn, Span};
use std::fmt;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Pointer to the file with an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    path: PathBuf,
    start: LineColumn,
    end: LineColumn,
}

impl Location {
    pub(crate) fn from_path_span(path: &Path, span: Span) -> Self {
        Location {
            path: path.to_owned(),
            start: span.start(),
            end: span.end(),
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} (line: {}, column: {})",
            self.path.display(),
            self.start.line,
            self.start.column
        )
    }
}

pub type SourcegenError = anyhow::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SourcegenErrorKind {
    // Tool errors
    #[error("Failed scan cargo metadata.")]
    MetadataError,
    #[error("Failed to process source file `{0}`.")]
    ProcessFile(String),
    #[error("{0}: generator '{1}' is not supported")]
    GeneratorNotFound(Location, String),
    #[error("{0}: Failed to generate source content.")]
    GeneratorError(Location),

    // Source parser errors
    #[error("{0}: multiple `generator` attributes are not allowed")]
    MultipleGeneratorAttributes(Location),
    #[error("{0}: `generator` attributes must be a string (for example, `generator = \"sample_generator\"`)")]
    GeneratorAttributeMustBeString(Location),
    #[error("{0}: missing `generator` attribute, must be a string (for example, `generator = \"sample_generator\"`)")]
    MissingGeneratorAttribute(Location),
    #[error("Failed to resolve module '{1}' with a parent module '{0}'")]
    CannotResolveModule(String, String),

    // Formatter errors
    #[error("`rustfmt` is not installed for the current toolchain. Run `rustup component add rustfmt` to install it.")]
    NoRustFmt,
    #[error("`rustup which rustfmt` failed.")]
    WhichRustFmtFailed,
    #[error("Failed to format chunk of code via `rustfmt <file>`.")]
    RustFmtFailed,
    #[error("`rustfmt` returned an error: {0}")]
    RustFmtError(String),

    #[error("Invalid package names: {0}")]
    InvalidPackageNames(String),
}
