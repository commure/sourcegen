use crate::error::{Location, SourcegenError, SourcegenErrorKind};
use crate::{GeneratorsMap, SourceGenerator};
use failure::ResultExt;
use proc_macro2::{LineColumn, TokenStream};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use syn::export::ToTokens;
use syn::spanned::Spanned;
use syn::{
    Attribute, AttributeArgs, Ident, Item, ItemEnum, ItemStruct, LitStr, Meta, NestedMeta,
    Visibility,
};

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
struct Region {
    from: usize,
    to: usize,
    indent: usize,
}

pub fn process_source_file(
    path: &Path,
    generators: &HashMap<&str, &dyn SourceGenerator>,
    is_root: bool,
) -> Result<(), SourcegenError> {
    let source = std::fs::read_to_string(path)
        .with_context(|_| SourcegenErrorKind::ProcessFile(path.display().to_string()))?;
    let file = syn::parse_file(&source)
        .with_context(|_| SourcegenErrorKind::ProcessFile(path.display().to_string()))?;
    let mut replacements = BTreeMap::new();
    handle_content(
        path,
        &source,
        &file.items,
        &generators,
        is_root,
        &mut replacements,
    )?;

    let output = render_expansions(path, &source, &replacements)?;

    if source != output {
        std::fs::write(path, output)
            .with_context(|_| SourcegenErrorKind::ProcessFile(path.display().to_string()))?;
    }
    Ok(())
}

/// `basefile` is used to tell `rustfmt` which configuration to use.
fn render_expansions(
    basefile: &Path,
    source: &str,
    expansions: &BTreeMap<Region, TokenStream>,
) -> Result<String, SourcegenError> {
    let mut output = String::with_capacity(source.len());
    let formatter = crate::rustfmt::Formatter::new()?;

    let mut offset = 0;
    for (region, replacement) in expansions {
        output += &source[offset..region.from];
        offset = region.to;
        let indent = format!("{:indent$}", "", indent = region.indent);
        let formatted = formatter.format(basefile, &replacement.to_string())?;

        let mut first = true;
        for line in formatted.lines() {
            // We don't want newline on the last line (the captured region does not include the
            // one) and also we don't want an indent on the first line (we splice after it).
            if first {
                first = false
            } else {
                output.push('\n');
                output += &indent;
            }
            output += line;
        }
    }
    output += &source[offset..];
    Ok(output)
}

fn handle_content(
    path: &Path,
    source: &str,
    items: &[Item],
    generators: &GeneratorsMap,
    is_root: bool,
    replacements: &mut BTreeMap<Region, TokenStream>,
) -> Result<(), SourcegenError> {
    for item in items {
        match item {
            Item::Enum(item) => {
                if let Some(invoke) = detect_invocation(path, &item.attrs, generators)? {
                    let context_location = invoke.context_location;
                    if let Some(expansion) = invoke
                        .generator
                        .generate_enum(invoke.args, item)
                        .with_context(|_| SourcegenErrorKind::GeneratorError(context_location))?
                    {
                        let region = enum_region(source, item, invoke.expand_attr_pos)?;
                        replacements.insert(region, expansion);
                    }
                }
            }

            Item::Struct(item) => {
                if let Some(invoke) = detect_invocation(path, &item.attrs, generators)? {
                    let context_location = invoke.context_location;
                    if let Some(expansion) = invoke
                        .generator
                        .generate_struct(invoke.args, item)
                        .with_context(|_| SourcegenErrorKind::GeneratorError(context_location))?
                    {
                        let region = struct_region(source, item, invoke.expand_attr_pos)?;
                        replacements.insert(region, expansion);
                    }
                }
            }

            Item::Mod(item) if item.content.is_some() => {
                let items = &item.content.as_ref().unwrap().1;
                handle_content(path, source, items, generators, false, replacements)?;
            }
            Item::Mod(item) => {
                let mod_file = crate::mods::resolve_module(path, &item, is_root)?;
                process_source_file(&mod_file, generators, false)?;
            }
            _ => {
                // What else do we want to support?
            }
        }
    }
    Ok(())
}

/// Detect the working region for the enums. The area starts after the `#[sourcegen]` attribute.
fn enum_region(source: &str, item: &ItemEnum, attr_pos: usize) -> Result<Region, SourcegenError> {
    let from_span = if attr_pos + 1 < item.attrs.len() {
        item.attrs[attr_pos + 1].span()
    } else if item.vis != Visibility::Inherited {
        item.vis.span()
    } else {
        item.enum_token.span()
    };
    let to_span = item.brace_token.span;

    let from = line_column_to_offset(source, from_span.start())?;
    let to = line_column_to_offset(source, to_span.end())?;
    Ok(Region {
        from,
        to,
        indent: from_span.start().column,
    })
}

/// Detect the working region for the structs. The area starts after the `#[sourcegen]` attribute.
fn struct_region(
    source: &str,
    item: &ItemStruct,
    attr_pos: usize,
) -> Result<Region, SourcegenError> {
    let from_span = if attr_pos + 1 < item.attrs.len() {
        item.attrs[attr_pos + 1].span()
    } else if item.vis != Visibility::Inherited {
        item.vis.span()
    } else {
        item.struct_token.span()
    };
    let to_span = if let Some(semi) = item.semi_token {
        semi.span()
    } else {
        item.fields.span()
    };

    let from = line_column_to_offset(source, from_span.start())?;
    let to = line_column_to_offset(source, to_span.end())?;
    Ok(Region {
        from,
        to,
        indent: from_span.start().column,
    })
}

/// Collect parameters from `#[sourcegen]` attribute.
fn detect_invocation<'a>(
    path: &Path,
    attrs: &[Attribute],
    generators: &'a GeneratorsMap,
) -> Result<Option<GeneratorInfo<'a>>, SourcegenError> {
    let sourcegen_attr = attrs.iter().position(|attr| {
        attr.path
            .segments
            .first()
            .map_or(false, |segment| segment.value().ident == "sourcegen")
    });
    if let Some(attr_pos) = sourcegen_attr {
        let loc = Location::from_path_span(path, attrs[attr_pos].span());
        let mut tokens = TokenStream::new();
        // Fake `#[sourcegen(<attrs>)]` attribute as `parse_meta` does not like if we have
        // `#[sourcegen::sourcegen(<attrs>)]`
        Ident::new("sourcegen", attrs[attr_pos].span()).to_tokens(&mut tokens);
        attrs[attr_pos].tts.to_tokens(&mut tokens);
        let meta: Meta = syn::parse2(tokens)
            .with_context(|_| SourcegenErrorKind::GeneratorError(loc.clone()))?;
        let mut invoke = detect_generator(path, meta, generators)?;
        invoke.expand_attr_pos = attr_pos;

        Ok(Some(invoke))
    } else {
        Ok(None)
    }
}

/// Map from the line number and column back to the offset.
fn line_column_to_offset(text: &str, lc: LineColumn) -> Result<usize, SourcegenError> {
    let mut line = lc.line as usize;

    assert!(line != 0, "line number must be 1-indexed");

    let mut offset = 0;
    for (idx, ch) in text.char_indices() {
        offset = idx;
        if line == 1 {
            break;
        }
        if ch == '\n' {
            line -= 1;
        }
    }
    offset += lc.column;
    Ok(offset.min(text.len()))
}

struct GeneratorInfo<'a> {
    /// Source generator to run
    generator: &'a dyn SourceGenerator,
    args: AttributeArgs,
    /// Used for figuring out the location to splice the code
    expand_attr_pos: usize,
    /// Location for error reporting
    context_location: Location,
}

fn detect_generator<'a>(
    path: &Path,
    meta: Meta,
    generators: &'a GeneratorsMap,
) -> Result<GeneratorInfo<'a>, SourcegenError> {
    let meta_span = meta.span();
    if let Meta::List(list) = meta {
        let mut name: Option<&LitStr> = None;
        for item in &list.nested {
            match item {
                NestedMeta::Meta(Meta::NameValue(nv)) if nv.ident == "generator" => {
                    if let syn::Lit::Str(ref value) = nv.lit {
                        if name.is_some() {
                            let loc = Location::from_path_span(path, item.span());
                            return Err(SourcegenErrorKind::MultipleGeneratorAttributes(loc).into());
                        }
                        name = Some(value);
                    } else {
                        let loc = Location::from_path_span(path, item.span());
                        return Err(SourcegenErrorKind::GeneratorAttributeMustBeString(loc).into());
                    }
                }
                _ => {}
            }
        }
        if let Some(name) = name {
            let name_span = name.span();
            let name = name.value();
            let args = list.nested.into_iter().collect::<Vec<_>>();
            let context_location = Location::from_path_span(path, meta_span);
            let generator = *generators.get(name.as_str()).ok_or_else(|| {
                SourcegenErrorKind::GeneratorNotFound(
                    Location::from_path_span(path, name_span),
                    name,
                )
            })?;
            return Ok(GeneratorInfo {
                generator,
                args,
                // We set it later
                expand_attr_pos: 0,
                context_location,
            });
        }
    }

    let loc = Location::from_path_span(path, meta_span);
    Err(SourcegenErrorKind::MissingGeneratorAttribute(loc).into())
}
