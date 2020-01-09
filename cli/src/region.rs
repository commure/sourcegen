use crate::SourceGenerator;
use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;
use syn::{Attribute, AttributeArgs, Item};

pub fn item_attributes(item: &mut syn::Item) -> Option<&mut Vec<Attribute>> {
    Some(match item {
        Item::ExternCrate(item) => &mut item.attrs,
        Item::Use(item) => &mut item.attrs,
        Item::Static(item) => &mut item.attrs,
        Item::Const(item) => &mut item.attrs,
        Item::Fn(item) => &mut item.attrs,
        Item::Mod(item) => &mut item.attrs,
        Item::ForeignMod(item) => &mut item.attrs,
        Item::Type(item) => &mut item.attrs,
        Item::Struct(item) => &mut item.attrs,
        Item::Enum(item) => &mut item.attrs,
        Item::Union(item) => &mut item.attrs,
        Item::Trait(item) => &mut item.attrs,
        Item::TraitAlias(item) => &mut item.attrs,
        Item::Impl(item) => &mut item.attrs,
        Item::Macro(item) => &mut item.attrs,
        Item::Macro2(item) => &mut item.attrs,
        _ => return None,
    })
}

pub fn item_end_span(item: &Item) -> Span {
    match item {
        Item::ExternCrate(item) => item.semi_token.span,
        Item::Use(item) => item.semi_token.span,
        Item::Static(item) => item.semi_token.span,
        Item::Const(item) => item.semi_token.span,
        Item::Fn(item) => item.block.span(),
        Item::Mod(item) => {
            if let Some(semi) = item.semi {
                semi.span()
            } else if let Some(ref content) = item.content {
                content.0.span
            } else {
                item.ident.span()
            }
        }
        Item::ForeignMod(item) => item.brace_token.span,
        Item::Type(item) => item.semi_token.span(),
        Item::Struct(item) => {
            if let Some(semi) = item.semi_token {
                semi.span()
            } else {
                item.fields.span()
            }
        }
        Item::Enum(item) => item.brace_token.span,
        Item::Union(item) => item.fields.span(),
        Item::Trait(item) => item.brace_token.span,
        Item::TraitAlias(item) => item.semi_token.span,
        Item::Impl(item) => item.brace_token.span,
        Item::Macro(item) => {
            if let Some(semi) = item.semi_token {
                semi.span()
            } else {
                item.mac.span()
            }
        }
        Item::Macro2(item) => item.rules.span(),
        _ => unreachable!(),
    }
}

pub fn invoke_generator(
    item: &Item,
    args: AttributeArgs,
    generator: &dyn SourceGenerator,
) -> Result<Option<TokenStream>, anyhow::Error> {
    match item {
        //        ExternCrate(ItemExternCrate),
        //        Use(ItemUse),
        //        Static(ItemStatic),
        //        Const(ItemConst),
        //        Fn(ItemFn),
        Item::Mod(item) => generator.generate_mod(args, item),
        //        ForeignMod(ItemForeignMod),
        //        Type(ItemType),
        Item::Struct(item) => generator.generate_struct(args, item),
        Item::Enum(item) => generator.generate_enum(args, item),
        //        Union(ItemUnion),
        Item::Trait(item) => generator.generate_trait(args, item),
        //        Impl(ItemImpl),
        //        Macro(ItemMacro),
        //        Macro2(ItemMacro2),
        //        Verbatim(ItemVerbatim),
        // FIXME: support other?
        _ => return Ok(None),
    }
}
