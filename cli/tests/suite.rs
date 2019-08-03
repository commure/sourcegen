use failure::Error;
use sourcegen_cli::SourcegenParameters;
use std::path::Path;
use std::process::Command;

pub mod generators;
pub mod helpers;

fn main() -> Result<(), Error> {
    install_rustfmt()?;

    let temp = tempfile::tempdir()?;
    let root = temp.path().join("root");
    copy_dir::copy_dir("tests/test_data", &root)?;
    for entry in std::fs::read_dir(&root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .map_or(true, |name| name != "fake_sourcegen")
        {
            eprintln!("running test for '{}'", path.strip_prefix(&root)?.display());
            run_test_dir(&path)?;
        }
    }

    Ok(())
}

fn install_rustfmt() -> Result<(), Error> {
    let output = Command::new("rustup")
        .arg("component")
        .arg("add")
        .arg("rustfmt")
        .output()?;

    // Ignore status, but print to the console
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        eprintln!(
            "Warning: failed to install rust fmt (exit code {}): {}",
            output.status.code().unwrap_or(0),
            err
        );
    }
    Ok(())
}

fn parameters(manifest: &Path) -> SourcegenParameters {
    SourcegenParameters {
        manifest: Some(manifest),
        generators: &[
            ("write-back", &self::generators::WriteBack),
            ("generate-impls", &self::generators::GenerateImpls),
        ],
        ..Default::default()
    }
}

fn run_test_dir(dir: &Path) -> Result<(), failure::Error> {
    let manifest = dir.join("input").join("Cargo.toml");
    sourcegen_cli::run_sourcegen(&parameters(&manifest))?;

    self::helpers::assert_matches_expected(dir, &dir.join("input"), &dir.join("expected"))?;
    Ok(())
}
