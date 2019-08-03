use crate::error::{SourcegenError, SourcegenErrorKind};
use failure::ResultExt;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// Rust code formatter. Uses an external `rustfmt` executable for formatting the code.
pub struct Formatter {
    rustfmt: String,
}

impl Formatter {
    pub fn new(root: &Path) -> Result<Self, SourcegenError> {
        let basedir = dunce::canonicalize(root).context(SourcegenErrorKind::WhichRustFmtFailed)?;
        let output = Command::new("rustup")
            .current_dir(basedir)
            .arg("which")
            .arg("rustfmt")
            .stderr(Stdio::null())
            .output()
            .context(SourcegenErrorKind::WhichRustFmtFailed)?;
        if !output.status.success() {
            return Err(SourcegenErrorKind::NoRustFmt.into());
        }
        let rustfmt = String::from_utf8(output.stdout)
            .context(SourcegenErrorKind::WhichRustFmtFailed)?
            .trim()
            .to_owned();
        Ok(Self { rustfmt })
    }

    /// Reformat generated block of code via rustfmt
    pub fn format(&self, basefile: &Path, content: &str) -> Result<String, SourcegenError> {
        let basedir = dunce::canonicalize(basefile.parent().unwrap())
            .context(SourcegenErrorKind::RustFmtFailed)?;
        let mut rustfmt = Command::new(&self.rustfmt)
            .current_dir(basedir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .context(SourcegenErrorKind::RustFmtFailed)?;

        rustfmt
            .stdin
            .as_mut()
            .unwrap()
            .write_all(content.as_bytes())
            .context(SourcegenErrorKind::RustFmtFailed)?;
        let output = rustfmt
            .wait_with_output()
            .context(SourcegenErrorKind::RustFmtFailed)?;
        rustfmt_output(output)
    }
}

fn rustfmt_output(output: Output) -> Result<String, SourcegenError> {
    if output.status.success() {
        let result = String::from_utf8(output.stdout).context(SourcegenErrorKind::RustFmtFailed)?;
        Ok(result)
    } else {
        let err = String::from_utf8(output.stderr).context(SourcegenErrorKind::RustFmtFailed)?;
        Err(SourcegenErrorKind::RustFmtError(err).into())
    }
}
