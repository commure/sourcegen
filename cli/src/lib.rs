//! Source generator command-line utility.
//!
//! Utility to do in-place source generation for Rust code. Takes a list of source generators to
//! run and applies them to all crates that have [`sourcegen`] dependency.
//!
//! [`sourcegen`]: http://crates.io/crates/sourcegen
use crate::error::{SourcegenError, SourcegenErrorKind};
use failure::{Error, ResultExt};
use proc_macro2::TokenStream;
use std::collections::HashMap;
use std::path::Path;

mod error;
mod generate;
mod mods;
mod rustfmt;

/// Trait to be implemented by source generators.
pub trait SourceGenerator {
    /// Expand struct definition. Return `None` if no changes are necessary.
    fn generate_struct(
        &self,
        _args: syn::AttributeArgs,
        _item: &syn::ItemStruct,
    ) -> Result<Option<TokenStream>, Error> {
        Ok(None)
    }

    /// Expand enum definition. Return `None` if no changes are necessary.
    fn generate_enum(
        &self,
        _args: syn::AttributeArgs,
        _item: &syn::ItemEnum,
    ) -> Result<Option<TokenStream>, Error> {
        Ok(None)
    }
}

pub(crate) type GeneratorsMap<'a> = HashMap<&'a str, &'a dyn SourceGenerator>;

/// Parameters for the source generation tool
#[derive(Default, Clone)]
pub struct SourcegenParameters<'a> {
    /// Root cargo manifest file to start from. If not given, the default is to use `Cargo.toml` in
    /// the current directory.
    pub manifest: Option<&'a Path>,
    /// List of generators to run. Each entry is a pair of generator name and trait object
    /// implementing the generator.
    pub generators: &'a [(&'a str, &'a dyn SourceGenerator)],

    #[doc(hidden)]
    pub __must_use_default: (),
}

/// Main entry point to the source generator toolkit.
pub fn run_sourcegen(parameters: &SourcegenParameters) -> Result<(), SourcegenError> {
    let generators = parameters
        .generators
        .iter()
        .cloned()
        .collect::<GeneratorsMap>();

    let mut cmd = cargo_metadata::MetadataCommand::new();
    if let Some(manifest) = parameters.manifest {
        cmd.manifest_path(manifest);
    } else {
        let path = std::env::current_dir().context(SourcegenErrorKind::MetadataError)?;
        let manifest = path.join("Cargo.toml");
        cmd.manifest_path(&manifest);
    }
    let metadata = cmd.exec().context(SourcegenErrorKind::MetadataError)?;

    let packages = metadata
        .packages
        .into_iter()
        .filter(|p| p.source.is_none())
        // FIXME: should we look at "rename", too?
        .filter(|p| p.dependencies.iter().any(|dep| dep.name == "sourcegen"));

    for package in packages {
        eprintln!("Generating source code in crate '{}'", package.name);
        for target in &package.targets {
            self::generate::process_source_file(&target.src_path, &generators, true)?;
        }
    }
    Ok(())
}