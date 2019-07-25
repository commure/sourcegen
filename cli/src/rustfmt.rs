use crate::error::{SourcegenError, SourcegenErrorKind};
use failure::ResultExt;
use std::process::{Command, Stdio};

/// Rust code formatter. Uses an external `rustfmt` executable for formatting the code.
pub struct Formatter {
    rustfmt: String,
    tempdir: tempfile::TempDir,
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

        // Setup a temporary directory
        let tempdir = tempfile::Builder::new()
            .prefix("source-expand")
            .tempdir()
            .context(SourcegenErrorKind::RustFmtSetup)?;

        // Can only normalize doc comments on nightly!
        // https://github.com/rust-lang/rustfmt/issues/3351
        if version.contains("nightly") {
            let cfg_path = tempdir.path().join("rustfmt.toml");
            std::fs::write(cfg_path, "normalize_doc_attributes = true")
                .context(SourcegenErrorKind::RustFmtSetup)?;
        }
        Ok(Self { rustfmt, tempdir })
    }

    /// Reformat generated block of code via rustfmt
    pub fn format(&self, content: &str) -> Result<String, SourcegenError> {
        let temp = tempfile::Builder::new()
            .tempfile_in(&self.tempdir)
            .context(SourcegenErrorKind::RustFmtFailed)?;
        std::fs::write(temp.path(), content).context(SourcegenErrorKind::RustFmtFailed)?;

        let output = Command::new(self.rustfmt.trim())
            .current_dir(&self.tempdir)
            .arg(temp.path())
            .output()
            .context(SourcegenErrorKind::RustFmtFailed)?;
        if output.status.success() {
            Ok(std::fs::read_to_string(temp.path()).context(SourcegenErrorKind::RustFmtFailed)?)
        } else {
            let err =
                String::from_utf8(output.stderr).context(SourcegenErrorKind::RustFmtFailed)?;
            Err(SourcegenErrorKind::RustFmtError(err).into())
        }
    }
}
