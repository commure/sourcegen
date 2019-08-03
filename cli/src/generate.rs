use crate::error::{Location, SourcegenError, SourcegenErrorKind};
use crate::{GeneratorsMap, SourceGenerator};
use failure::ResultExt;
use proc_macro2::{LineColumn, TokenStream};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use syn::export::ToTokens;
use syn::spanned::Spanned;
use syn::{Attribute, AttributeArgs, Ident, Item, LitStr, Meta, NestedMeta};

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
    let mut file = syn::parse_file(&source)
        .with_context(|_| SourcegenErrorKind::ProcessFile(path.display().to_string()))?;
    let mut replacements = BTreeMap::new();
    handle_content(
        path,
        &source,
        &mut file.items,
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
    let formatter = crate::rustfmt::Formatter::new(basefile.parent().unwrap())?;

    let mut offset = 0;
    let is_cr_lf = is_cr_lf(source);
    for (region, replacement) in expansions {
        output += &source[offset..region.from];
        offset = region.to;
        let indent = format!("{:indent$}", "", indent = region.indent);
        if !replacement.is_empty() {
            let replacement = format!("// Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.\n{}", replacement);
            let formatted = formatter.format(basefile, &replacement)?;
            let mut first = true;
            for line in formatted.lines() {
                // We don't want newline on the last line (the captured region does not include the
                // one) and also we don't want an indent on the first line (we splice after it).
                if first {
                    first = false
                } else {
                    if is_cr_lf {
                        output.push('\r');
                    }
                    output.push('\n');
                    output += &indent;
                }
                output += line;
            }
        }
    }
    output += &source[offset..];
    Ok(output)
}

fn handle_content(
    path: &Path,
    source: &str,
    items: &mut [Item],
    generators: &GeneratorsMap,
    is_root: bool,
    replacements: &mut BTreeMap<Region, TokenStream>,
) -> Result<(), SourcegenError> {
    let mut item_idx = 0;
    while item_idx < items.len() {
        item_idx += 1;
        let (head, tail) = items.split_at_mut(item_idx);
        let item = head.last_mut().unwrap();

        let mut empty_attrs = Vec::new();
        let attrs = crate::region::item_attributes(item).unwrap_or(&mut empty_attrs);
        if let Some(invoke) = detect_invocation(path, attrs, generators)? {
            let context_location = invoke.context_location;
            let result = crate::region::invoke_generator(item, invoke.args, invoke.generator)
                .with_context(|_| SourcegenErrorKind::GeneratorError(context_location))?;
            if let Some(expansion) = result {
                let indent = invoke.sourcegen_attr.span().start().column;
                let from_loc = invoke.sourcegen_attr.bracket_token.span.end();
                let from = line_column_to_offset(source, from_loc)?;
                let from = from + skip_whitespaces(&source[from..]);

                let skip_count = (0..tail.len())
                    .find(|pos| {
                        !is_generated(
                            crate::region::item_attributes(&mut tail[*pos])
                                .unwrap_or(&mut empty_attrs),
                        )
                    })
                    .unwrap_or(tail.len());
                let to_span = if skip_count == 0 {
                    crate::region::item_end_span(item)
                } else {
                    // Skip consecutive items marked via `#[sourcegen::generated]`
                    item_idx += skip_count;
                    crate::region::item_end_span(&tail[skip_count - 1])
                };
                let to = line_column_to_offset(source, to_span.end())?;

                let region = Region { from, to, indent };
                replacements.insert(region, expansion);
                continue;
            }
        }

        if let Item::Mod(item) = item {
            if item.content.is_some() {
                let items = &mut item.content.as_mut().unwrap().1;
                handle_content(path, source, items, generators, false, replacements)?;
            } else {
                let mod_file = crate::mods::resolve_module(path, &item, is_root)?;
                process_source_file(&mod_file, generators, false)?;
            }
        }
    }
    Ok(())
}

fn is_generated(attrs: &[Attribute]) -> bool {
    let sourcegen_attr = attrs.iter().find(|attr| {
        attr.path
            .segments
            .first()
            .map_or(false, |segment| segment.value().ident == "sourcegen")
    });
    if let Some(sourcegen) = sourcegen_attr {
        sourcegen
            .path
            .segments
            .iter()
            .skip(1)
            .next()
            .map_or(false, |segment| segment.ident == "generated")
    } else {
        false
    }
}

/// Collect parameters from `#[sourcegen]` attribute.
fn detect_invocation<'a>(
    path: &Path,
    attrs: &mut Vec<Attribute>,
    generators: &'a GeneratorsMap,
) -> Result<Option<GeneratorInfo<'a>>, SourcegenError> {
    let sourcegen_attr = attrs.iter().position(|attr| {
        attr.path
            .segments
            .first()
            .map_or(false, |segment| segment.value().ident == "sourcegen")
    });
    if let Some(attr_pos) = sourcegen_attr {
        let sourcegen_attr = attrs.drain(0..attr_pos + 1).last().unwrap();
        let invoke = detect_generator(path, sourcegen_attr, generators)?;
        Ok(Some(invoke))
    } else {
        Ok(None)
    }
}

/// Map from the line number and column back to the offset.
fn line_column_to_offset(text: &str, lc: LineColumn) -> Result<usize, SourcegenError> {
    let mut line = lc.line as usize;

    assert_ne!(line, 0, "line number must be 1-indexed");

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

fn skip_whitespaces(text: &str) -> usize {
    let end = text.trim_start().as_ptr() as usize;
    let start = text.as_ptr() as usize;
    end - start
}

struct GeneratorInfo<'a> {
    /// Source generator to run
    generator: &'a dyn SourceGenerator,
    args: AttributeArgs,
    /// `#[sourcegen]` attribute itself
    sourcegen_attr: Attribute,
    /// Location for error reporting
    context_location: Location,
}

fn detect_generator<'a>(
    path: &Path,
    sourcegen_attr: Attribute,
    generators: &'a GeneratorsMap,
) -> Result<GeneratorInfo<'a>, SourcegenError> {
    let meta = parse_sourcegen_attr(path, &sourcegen_attr)?;

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
                sourcegen_attr,
                context_location,
            });
        }
    }

    let loc = Location::from_path_span(path, meta_span);
    Err(SourcegenErrorKind::MissingGeneratorAttribute(loc).into())
}

fn parse_sourcegen_attr(path: &Path, sourcegen_attr: &Attribute) -> Result<Meta, SourcegenError> {
    let loc = Location::from_path_span(path, sourcegen_attr.span());
    let mut tokens = TokenStream::new();
    // Fake `#[sourcegen(<attrs>)]` attribute as `parse_meta` does not like if we have
    // `#[sourcegen::sourcegen(<attrs>)]`
    Ident::new("sourcegen", sourcegen_attr.span()).to_tokens(&mut tokens);
    sourcegen_attr.tts.to_tokens(&mut tokens);
    let meta: Meta =
        syn::parse2(tokens).with_context(|_| SourcegenErrorKind::GeneratorError(loc.clone()))?;
    Ok(meta)
}

/// Look at the first newline and decide if we should use `\r\n` (Windows).
fn is_cr_lf(source: &str) -> bool {
    if let Some(pos) = source.find('\n') {
        source[..pos].ends_with('\r')
    } else {
        false
    }
}
