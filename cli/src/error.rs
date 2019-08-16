use failure::{Backtrace, Context, Fail};
use proc_macro2::{LineColumn, Span};
use std::fmt;
use std::path::{Path, PathBuf};

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

#[derive(Debug)]
pub struct SourcegenError {
    inner: Context<SourcegenErrorKind>,
}

impl Fail for SourcegenError {
    fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for SourcegenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl SourcegenError {
    /// Kind of an error happened during source generation.
    pub fn kind(&self) -> &SourcegenErrorKind {
        self.inner.get_context()
    }
}

impl From<SourcegenErrorKind> for SourcegenError {
    fn from(kind: SourcegenErrorKind) -> Self {
        Self {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<SourcegenErrorKind>> for SourcegenError {
    fn from(inner: Context<SourcegenErrorKind>) -> Self {
        Self { inner }
    }
}

#[derive(Debug, Fail, Clone, PartialEq, Eq)]
pub enum SourcegenErrorKind {
    // Tool errors
    #[fail(display = "Failed scan cargo metadata.")]
    MetadataError,
    #[fail(display = "Failed to process source file `{}`.", _0)]
    ProcessFile(String),
    #[fail(display = "{}: generator '{}' is not supported", _0, _1)]
    GeneratorNotFound(Location, String),
    #[fail(display = "{}: Failed to generate source content.", _0)]
    GeneratorError(Location),

    // Source parser errors
    #[fail(display = "{}: multiple `generator` attributes are not allowed", _0)]
    MultipleGeneratorAttributes(Location),
    #[fail(
        display = "{}: `generator` attributes must be a string (for example, `generator = \"sample_generator\"`)",
        _0
    )]
    GeneratorAttributeMustBeString(Location),
    #[fail(
        display = "{}: missing `generator` attribute, must be a string (for example, `generator = \"sample_generator\"`)",
        _0
    )]
    MissingGeneratorAttribute(Location),
    #[fail(
        display = "Failed to resolve module '{}' with a parent module '{}'",
        _1, _0
    )]
    CannotResolveModule(String, String),

    // Formatter errors
    #[fail(
        display = "`rustfmt` is not installed for the current toolchain. Run `rustup component add rustfmt` to install it."
    )]
    NoRustFmt,
    #[fail(display = "`rustup which rustfmt` failed.")]
    WhichRustFmtFailed,
    #[fail(display = "Failed to determine `rustfmt` version via `rustfmt --version`.")]
    RustFmtVersionFailed,
    #[fail(
        display = "Failed to setup a temporary directory and a configuration file `rustfmt.toml` for `rustfmt`."
    )]
    RustFmtSetup,
    #[fail(display = "Failed to format chunk of code via `rustfmt <file>`.")]
    RustFmtFailed,
    #[fail(display = "`rustfmt` returned an error: {}", _0)]
    RustFmtError(String),
}
