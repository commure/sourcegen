use crate::error::{SourcegenError, SourcegenErrorKind};
use std::path::PathBuf;
use syn::{Attribute, ItemMod, Lit, Meta};

// FIXME: support cfg_attr, too?
pub struct ModResolver {
    base: PathBuf,
}

impl ModResolver {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        ModResolver { base: base.into() }
    }

    /// Nested module -- append a new directory.
    pub fn push_module(&self, name: &str) -> Self {
        ModResolver {
            base: self.base.join(name),
        }
    }

    /// Resolve to a module file.
    pub fn resolve_module_file(&self, item: &ItemMod) -> Result<PathBuf, SourcegenError> {
        if let Some(path) = detect_mod_path(&item.attrs) {
            Ok(self.base.join(path))
        } else {
            let name = item.ident.to_string();
            let name = name.trim_start_matches("r#");
            let path = self.base.join(&format!("{}.rs", name));
            if path.is_file() {
                return Ok(path);
            }
            let path = self.base.join(&name).join("mod.rs");
            if path.is_file() {
                return Ok(path);
            }
            Err(SourcegenErrorKind::CannotResolveModule(
                path.display().to_string(),
                name.to_owned(),
            )
            .into())
        }
    }
}

fn detect_mod_path(attrs: &[Attribute]) -> Option<String> {
    let attr = attrs.iter().find(|attr| attr.path.is_ident("path"))?;
    let meta = attr.parse_meta().ok()?;
    if let Meta::NameValue(nv) = meta {
        if let Lit::Str(ref value) = nv.lit {
            return Some(value.value());
        }
    }
    None
}
