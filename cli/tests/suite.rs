use failure::Error;
use sourcegen_cli::SourcegenParameters;
use std::path::Path;

pub mod generators;
pub mod helpers;

fn main() -> Result<(), Error> {
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
            helpers::install_rustfmt(&path)?;
            run_test_dir(&path)?;
        }
    }

    Ok(())
}

fn parameters(manifest: &Path) -> SourcegenParameters {
    SourcegenParameters {
        manifest: Some(manifest),
        generators: &[
            ("write-back", &self::generators::WriteBack),
            ("generate-impls", &self::generators::GenerateImpls),
            ("generate-simple", &self::generators::GenerateSimple),
            (
                "generate-doc-comments",
                &self::generators::GenerateDocComments,
            ),
            ("generate-file", &self::generators::GenerateFile),
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
