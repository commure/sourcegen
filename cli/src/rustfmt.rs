use crate::error::{SourcegenError, SourcegenErrorKind};
use failure::ResultExt;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// Rust code formatter. Uses an external `rustfmt` executable for formatting the code.
pub struct Formatter {
    rustfmt: String,
    // If we should make rustfmt to normalize doc attributes. Currently only works on nightly.
    normalize_doc_attributes: bool,
}

impl Formatter {
    pub fn new() -> Result<Self, SourcegenError> {
        let output = Command::new("rustup")
            .arg("which")
            .arg("rustfmt")
            .stderr(Stdio::null())
            .output()
            .context(SourcegenErrorKind::WhichRustFmtFailed)?;
        if !output.status.success() {
            return Err(SourcegenErrorKind::NoRustFmt.into());
        }
        let rustfmt =
            String::from_utf8(output.stdout).context(SourcegenErrorKind::WhichRustFmtFailed)?;

        // Probe rustfmt version
        let version = Command::new(rustfmt.trim())
            .arg("--version")
            .output()
            .context(SourcegenErrorKind::RustFmtVersionFailed)?;
        let version =
            String::from_utf8(version.stdout).context(SourcegenErrorKind::RustFmtVersionFailed)?;

        let is_nightly = version.contains("nightly");
        Ok(Self {
            rustfmt,
            normalize_doc_attributes: is_nightly,
        })
    }

    /// Reformat generated block of code via rustfmt
    pub fn format(&self, basefile: &Path, content: &str) -> Result<String, SourcegenError> {
        // This should be dropped after we are done with `rustfmt`, so declare it here.
        let tempconfig;
        let mut rustfmt = Command::new(self.rustfmt.trim());

        // If we are want to normalize doc attributes, capture `rustfmt` configuration and turn on
        // `normalize_doc_attributes` flag in it.
        if self.normalize_doc_attributes {
            tempconfig = rustfmt_adjusted_config(self.rustfmt.trim(), basefile)?;
            rustfmt.arg("--config-path").arg(tempconfig.path());
        }
        let mut process = rustfmt
            .current_dir(basefile.parent().unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .context(SourcegenErrorKind::RustFmtFailed)?;

        process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(content.as_bytes())
            .context(SourcegenErrorKind::RustFmtFailed)?;
        let output = process
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

/// Capture current `rustfmt` config that will be used for formatting given file.
fn rustfmt_config(rustfmt: &str, basefile: &Path) -> Result<String, SourcegenError> {
    // Note: was broken on stable, so we only do this on nightly.
    // https://github.com/rust-lang/rustfmt/issues/3536
    let output = Command::new(rustfmt)
        .arg("--print-config")
        .arg("current")
        .arg(basefile)
        .output()
        .context(SourcegenErrorKind::RustFmtFailed)?;
    rustfmt_output(output)
}

/// Generate adjusted rustfmt configuration. Currently the only adjustment we make is we turn on
/// the `normalize_doc_attributes` flag so all `#[doc = ..]` comments are rendered as `///`.
/// This currently only works on nightl `rustfmt`.
fn rustfmt_adjusted_config(
    rustfmt: &str,
    basefile: &Path,
) -> Result<tempfile::NamedTempFile, SourcegenError> {
    let config = rustfmt_config(rustfmt, basefile)?;

    // We do a naive string replacement here.
    let config = config.replace(
        "normalize_doc_attributes = false",
        "normalize_doc_attributes = true",
    );

    let mut tempfile = tempfile::Builder::new()
        .tempfile()
        .context(SourcegenErrorKind::RustFmtFailed)?;

    tempfile
        .write_all(config.as_bytes())
        .context(SourcegenErrorKind::RustFmtFailed)?;
    Ok(tempfile)
}
