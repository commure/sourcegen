use failure::Error;
use pretty_assertions::assert_eq as pretty_assert_eq;
use std::path::Path;

/// Assert that all files at `expected` path exist at `actual` path and the contents of the files
/// is the same.
pub fn assert_matches_expected(base: &Path, actual: &Path, expected: &Path) -> Result<(), Error> {
    assert_eq!(
        actual.is_file(),
        expected.is_file(),
        "actual '{}' is_file({}) != expected '{}' is_file({})",
        actual.display(),
        actual.is_file(),
        expected.display(),
        expected.is_file()
    );

    if actual.is_file() {
        let actual_content = std::fs::read_to_string(actual)?;
        let expected_content = std::fs::read_to_string(expected)?;
        pretty_assert_eq!(
            PrettyString(&actual_content),
            PrettyString(&expected_content),
            "contents of actual file '{}' differs from the contents of expected file '{}'",
            actual.strip_prefix(base)?.display(),
            expected.strip_prefix(base)?.display()
        );
    } else {
        let mut expected_it = std::fs::read_dir(expected)?;
        while let Some(expected_child) = expected_it.next() {
            let expected_child = expected_child?;
            let name = expected_child.file_name();

            let actual_child = actual.join(&name);

            assert!(
                actual_child.exists(),
                "expected file '{}' does not exists in actual output '{}'",
                Path::new(&name).display(),
                actual.strip_prefix(base)?.display()
            );

            assert_matches_expected(base, &actual_child, &expected_child.path())?;
        }

        // Make sure everything in `actual` also exists in `expected`
        let mut actual_it = std::fs::read_dir(actual)?;
        while let Some(actual_child) = actual_it.next() {
            let actual_child = actual_child?;
            let name = actual_child.file_name();
            let expected_child = expected.join(&name);

            assert!(
                expected_child.exists(),
                "actual file '{}' does not exists in expected output '{}'",
                Path::new(&name).display(),
                actual.strip_prefix(base)?.display()
            );
        }
    }

    Ok(())
}

/// Wrapper around string slice that makes debug output `{:?}` to print string same way as `{}`.
/// Used in different `assert*!` macros in combination with `pretty_assertions` crate to make
/// test failures to show nice diffs.
#[derive(PartialEq, Eq)]
#[doc(hidden)]
pub struct PrettyString<'a>(pub &'a str);

/// Make diff to display string as multi-line string
impl<'a> std::fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
