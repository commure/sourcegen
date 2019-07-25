use crate::error::{SourcegenError, SourcegenErrorKind};
use std::path::{Path, PathBuf};
use syn::{Attribute, ItemMod, Lit, Meta};

// FIXME: support cfg_attr, too?
pub fn resolve_module(
    parent: &Path,
    item: &ItemMod,
    is_root: bool,
) -> Result<PathBuf, SourcegenError> {
    if let Some(path) = detect_mod_path(&item.attrs) {
        let parent_path = parent.parent().ok_or_else(|| {
            SourcegenErrorKind::CannotResolveModule(parent.display().to_string(), path.to_owned())
        })?;
        Ok(parent_path.join(path))
    } else {
        resolve_module_default(parent, &item.ident.to_string(), is_root)
    }
}

/// Resolve modules using default
fn resolve_module_default(
    parent: &Path,
    name: &str,
    is_root: bool,
) -> Result<PathBuf, SourcegenError> {
    let parent_path = parent.parent().ok_or_else(|| {
        SourcegenErrorKind::CannotResolveModule(parent.display().to_string(), name.to_owned())
    })?;

    let parent_file = parent.file_name().and_then(|s| s.to_str()).ok_or_else(|| {
        SourcegenErrorKind::CannotResolveModule(parent.display().to_string(), name.to_owned())
    })?;
    let parent_name = parent_file.trim_end_matches(".rs");
    let is_mod = parent_file == "mod.rs" || is_root;
    let base = if is_mod {
        parent_path.to_path_buf()
    } else {
        parent_path.join(parent_name)
    };
    let path = base.join(&format!("{}.rs", name));
    if path.is_file() {
        return Ok(path);
    }
    let path = base.join(name).join("mod.rs");
    if path.is_file() {
        return Ok(path);
    }
    Err(SourcegenErrorKind::CannotResolveModule(path.display().to_string(), name.to_owned()).into())
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
