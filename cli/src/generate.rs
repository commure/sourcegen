use crate::error::{Location, SourcegenError, SourcegenErrorKind};
use crate::mods::ModResolver;
use crate::{GeneratorsMap, SourceGenerator};
use failure::ResultExt;
use proc_macro2::{LineColumn, TokenStream};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use syn::spanned::Spanned;
use syn::{Attribute, AttributeArgs, File, Item, LitStr, Meta, NestedMeta};

static ITEM_COMMENT: &str =
    "// Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.";
static FILE_COMMENT: &str = "// Generated. All manual edits below this line will be discarded.";

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
struct Region {
    from: usize,
    to: usize,
    indent: usize,
}

pub fn process_source_file(
    path: &Path,
    generators: &HashMap<&str, &dyn SourceGenerator>,
    mod_resolver: &ModResolver,
) -> Result<(), SourcegenError> {
    let source = std::fs::read_to_string(path)
        .with_context(|_| SourcegenErrorKind::ProcessFile(path.display().to_string()))?;
    let mut file = syn::parse_file(&source)
        .with_context(|_| SourcegenErrorKind::ProcessFile(path.display().to_string()))?;

    let output = if let Some(invoke) = detect_file_invocation(path, &mut file, generators)? {
        if !invoke.is_file {
            // Remove all attributes in front of the `#![sourcegen]` attribute
            file.attrs.drain(0..invoke.sourcegen_attr_index + 1);
        }

        // Handle full file generation
        let context_location = invoke.context_location;
        let result = invoke
            .generator
            .generate_file(invoke.args, &file)
            .with_context(|_| SourcegenErrorKind::GeneratorError(context_location))?;
        if let Some(expansion) = result {
            let from_loc = if invoke.is_file {
                crate::region::item_end_span(&file.items[0]).end()
            } else {
                invoke.sourcegen_attr.bracket_token.span.end()
            };
            let from = line_column_to_offset(&source, from_loc)?;
            let from = from + skip_whitespaces(&source[from..]);
            let region = Region {
                from,
                to: source.len(),
                indent: 0,
            };

            // Replace the whole file
            let mut replacements = BTreeMap::new();
            replacements.insert(region, expansion);
            render_expansions(path, &source, &replacements, FILE_COMMENT)?
        } else {
            // Nothing to replace
            return Ok(());
        }
    } else {
        let mut replacements = BTreeMap::new();
        handle_content(
            path,
            &source,
            &mut file.items,
            &generators,
            &mut replacements,
            &mod_resolver,
        )?;
        render_expansions(path, &source, &replacements, ITEM_COMMENT)?
    };

    if source != output {
        std::fs::write(path, output)
            .with_context(|_| SourcegenErrorKind::ProcessFile(path.display().to_string()))?;
    }
    Ok(())
}

/// Render given list of replacements into the source file. `basefile` is used to determine base
/// directory to run `rustfmt` in (so it can use local overrides for formatting rules).
///
/// `comment` is the warning comment that will be added in front of each generated block.
fn render_expansions(
    basefile: &Path,
    source: &str,
    expansions: &BTreeMap<Region, TokenStream>,
    comment: &str,
) -> Result<String, SourcegenError> {
    let mut output = String::with_capacity(source.len());
    let formatter = crate::rustfmt::Formatter::new(basefile.parent().unwrap())?;

    let mut offset = 0;
    let is_cr_lf = is_cr_lf(source);
    for (region, tokens) in expansions {
        output += &source[offset..region.from];
        offset = region.to;
        let indent = format!("{:indent$}", "", indent = region.indent);
        if !tokens.is_empty() {
            let replacement = Replacement {
                comment,
                is_cr_lf,
                tokens,
            };
            let formatted = formatter.format(basefile, replacement)?;
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
    // Insert newline at the end of the file!
    if offset == source.len() {
        if is_cr_lf {
            output.push('\r');
        }
        output.push('\n');
    }
    output += &source[offset..];
    Ok(output)
}

fn handle_content(
    path: &Path,
    source: &str,
    items: &mut [Item],
    generators: &GeneratorsMap,
    replacements: &mut BTreeMap<Region, TokenStream>,
    mod_resolver: &ModResolver,
) -> Result<(), SourcegenError> {
    let mut item_idx = 0;
    while item_idx < items.len() {
        item_idx += 1;
        let (head, tail) = items.split_at_mut(item_idx);
        let item = head.last_mut().unwrap();

        let mut empty_attrs = Vec::new();
        let attrs = crate::region::item_attributes(item).unwrap_or(&mut empty_attrs);
        if let Some(invoke) = detect_invocation(path, attrs, generators)? {
            // Remove all attributes in front of the `#[sourcegen]` attribute
            attrs.drain(0..invoke.sourcegen_attr_index + 1);
            let context_location = invoke.context_location;
            let result = crate::region::invoke_generator(item, invoke.args, invoke.generator)
                .with_context(|_| SourcegenErrorKind::GeneratorError(context_location))?;
            if let Some(expansion) = result {
                let indent = invoke.sourcegen_attr.span().start().column;
                let from_loc = invoke.sourcegen_attr.bracket_token.span.end();
                let from = line_column_to_offset(source, from_loc)?;
                let from = from + skip_whitespaces(&source[from..]);

                // Find the first item that is not marked as "generated"
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
            let nested_mod_resolved = mod_resolver.push_module(&item.ident.to_string());
            if item.content.is_some() {
                let items = &mut item.content.as_mut().unwrap().1;
                handle_content(
                    path,
                    source,
                    items,
                    generators,
                    replacements,
                    &nested_mod_resolved,
                )?;
            } else {
                let mod_file = mod_resolver.resolve_module_file(item)?;
                process_source_file(&mod_file, generators, &nested_mod_resolved)?;
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
            .map_or(false, |segment| segment.ident == "sourcegen")
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

fn detect_file_invocation<'a>(
    path: &Path,
    file: &mut File,
    generators: &'a GeneratorsMap,
) -> Result<Option<GeneratorInfo<'a>>, SourcegenError> {
    if let Some(mut invoke) = detect_invocation(path, &mut file.attrs, generators)? {
        // This flag should only be set when we are processing a special workaround
        invoke.is_file = false;
        return Ok(Some(invoke));
    }

    if let Some(item) = file.items.iter_mut().next() {
        // Special case: if first item in the file has `sourcegen::sourcegen` attribute with `file` set
        // to `true`, we treat it as file sourcegen.
        let mut empty_attrs = Vec::new();
        let attrs = crate::region::item_attributes(item).unwrap_or(&mut empty_attrs);
        if let Some(invoke) = detect_invocation(path, &mut attrs.clone(), generators)? {
            if invoke.is_file {
                return Ok(Some(invoke));
            }
        }
    }
    Ok(None)
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
            .map_or(false, |segment| segment.ident == "sourcegen")
    });
    if let Some(attr_pos) = sourcegen_attr {
        let invoke = detect_generator(path, attrs, attr_pos, generators)?;
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
    /// Index of `#[sourcegen]` attribute
    sourcegen_attr_index: usize,
    /// Location for error reporting
    context_location: Location,
    /// If this invocation should regenerate the whole block up to the end.
    /// (this is used as a workaround for attributes not allowed on modules)
    is_file: bool,
}

fn detect_generator<'a>(
    path: &Path,
    attrs: &[Attribute],
    sourcegen_attr_index: usize,
    generators: &'a GeneratorsMap,
) -> Result<GeneratorInfo<'a>, SourcegenError> {
    let sourcegen_attr = attrs[sourcegen_attr_index].clone();

    let loc = Location::from_path_span(path, sourcegen_attr.span());
    let meta = sourcegen_attr
        .parse_meta()
        .with_context(|_| SourcegenErrorKind::GeneratorError(loc.clone()))?;

    let meta_span = meta.span();
    if let Meta::List(list) = meta {
        let mut name: Option<&LitStr> = None;
        let mut is_file = false;
        for item in &list.nested {
            match item {
                NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("generator") => {
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
                NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("file") => {
                    if let syn::Lit::Bool(ref value) = nv.lit {
                        is_file = value.value;
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
                sourcegen_attr_index,
                sourcegen_attr,
                context_location,
                is_file,
            });
        }
    }

    let loc = Location::from_path_span(path, meta_span);
    Err(SourcegenErrorKind::MissingGeneratorAttribute(loc).into())
}

/// Look at the first newline and decide if we should use `\r\n` (Windows).
fn is_cr_lf(source: &str) -> bool {
    if let Some(pos) = source.find('\n') {
        source[..pos].ends_with('\r')
    } else {
        false
    }
}

/// Struct used to generate replacement code directly into stdin of `rustfmt`.
struct Replacement<'a> {
    comment: &'a str,
    is_cr_lf: bool,
    tokens: &'a TokenStream,
}

impl std::fmt::Display for Replacement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Write;

        f.write_str(self.comment)?;
        if self.is_cr_lf {
            f.write_char('\r')?;
        }
        f.write_char('\n')?;
        write!(f, "{}", self.tokens)
    }
}
