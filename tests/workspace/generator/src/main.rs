use failure::Error;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use sourcegen_cli::{SourceGenerator, SourcegenParameters};
use syn::{AttributeArgs, Ident, Meta, NestedMeta};

/// Entry point -- call into `source-expand` crate with our generator as a parameter.
pub fn main() {
    let parameters = SourcegenParameters {
        generators: &[
            ("generate-enum", &GenerateEnum),
            ("generate-struct", &GenerateStruct),
            ("generate-mod", &GenerateMod),
        ],
        ..Default::default()
    };
    if let Err(err) = sourcegen_cli::run_sourcegen(&parameters) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

struct GenerateEnum;

/// Enum source generator -- generates X amount of literals in enum where X is provided
/// via `count = X` attribute argument.
impl SourceGenerator for GenerateEnum {
    fn generate_enum(
        &self,
        args: syn::AttributeArgs,
        item: &syn::ItemEnum,
    ) -> Result<Option<TokenStream>, Error> {
        let count = find_count(&args);
        let lits = (0..count).map(|idx| {
            let lit = Ident::new(&format!("Literal{}", idx), Span::call_site());
            quote!(#lit)
        });
        let vis = &item.vis;
        let ident = &item.ident;
        Ok(Some(quote! {
            /// This comment is generated
            #vis enum #ident {
                #(#lits),*
            }
        }))
    }
}

struct GenerateStruct;

/// Struct source generator -- generates X amount of fields in struct where X is provided
/// via `count = X` attribute argument.
impl SourceGenerator for GenerateStruct {
    fn generate_struct(
        &self,
        args: syn::AttributeArgs,
        item: &syn::ItemStruct,
    ) -> Result<Option<TokenStream>, Error> {
        let count = find_count(&args);
        let lits = (0..count).map(|idx| {
            let lit = Ident::new(&format!("field{}", idx), Span::call_site());
            quote!(#lit)
        });
        let vis = &item.vis;
        let ident = &item.ident;
        Ok(Some(quote! {
            /// This comment is generated
            #vis struct #ident {
                #(pub #lits: usize),*
            }
        }))
    }
}

struct GenerateMod;

/// Mod source generator -- generates X empty structs where X is provided via `count = X`
/// attribute argument.
impl SourceGenerator for GenerateMod {
    fn generate_mod(
        &self,
        args: syn::AttributeArgs,
        item: &syn::ItemMod,
    ) -> Result<Option<TokenStream>, Error> {
        let count = find_count(&args);
        let lits = (0..count).map(|idx| {
            let lit = Ident::new(&format!("Struct{}", idx), Span::call_site());
            quote!(#lit)
        });
        let vis = &item.vis;
        let ident = &item.ident;
        Ok(Some(quote! {
            /// This comment is generated
            #vis mod #ident {
                #(pub struct #lits;)*
            }
        }))
    }
}

/// Helper function to find `count = X` in attribute arguments.
fn find_count(args: &AttributeArgs) -> usize {
    for item in args {
        match item {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.ident == "count" => {
                if let syn::Lit::Int(ref value) = nv.lit {
                    return value.value() as usize;
                }
            }
            _ => {}
        }
    }
    0
}
