use failure::Error;
use proc_macro2::TokenStream;
use quote::quote;
use sourcegen_cli::tokens::{NewLine, PlainComment};
use sourcegen_cli::SourceGenerator;

/// Writes back the input without any changes
pub struct WriteBack;

impl SourceGenerator for WriteBack {
    fn generate_struct(
        &self,
        _args: syn::AttributeArgs,
        item: &syn::ItemStruct,
    ) -> Result<Option<TokenStream>, Error> {
        Ok(Some(quote! {
            #item
        }))
    }

    fn generate_enum(
        &self,
        _args: syn::AttributeArgs,
        item: &syn::ItemEnum,
    ) -> Result<Option<TokenStream>, Error> {
        Ok(Some(quote! {
            #item
        }))
    }

    fn generate_mod(
        &self,
        _args: syn::AttributeArgs,
        item: &syn::ItemMod,
    ) -> Result<Option<TokenStream>, Error> {
        Ok(Some(quote! {
            #item
        }))
    }
}

/// Generate some impls along with the struct itself
pub struct GenerateImpls;

impl SourceGenerator for GenerateImpls {
    fn generate_struct(
        &self,
        _args: syn::AttributeArgs,
        item: &syn::ItemStruct,
    ) -> Result<Option<TokenStream>, Error> {
        let vis = &item.vis;
        let ident = &item.ident;
        Ok(Some(quote! {
            #vis struct #ident;

            #[sourcegen::generated]
            impl #ident {}
        }))
    }
}

/// Generate one field in the struct
pub struct GenerateSimple;

impl SourceGenerator for GenerateSimple {
    fn generate_struct(
        &self,
        _args: syn::AttributeArgs,
        item: &syn::ItemStruct,
    ) -> Result<Option<TokenStream>, Error> {
        let vis = &item.vis;
        let ident = &item.ident;
        Ok(Some(quote! {
            #vis struct #ident {
                pub hello: String,
            }
        }))
    }
}

/// Generates a struct with a doc comment
pub struct GenerateDocComments;

impl SourceGenerator for GenerateDocComments {
    fn generate_struct(
        &self,
        _args: syn::AttributeArgs,
        item: &syn::ItemStruct,
    ) -> Result<Option<TokenStream>, Error> {
        let vis = &item.vis;
        let ident = &item.ident;
        Ok(Some(quote! {
            /// Some generated comment here
            #vis struct #ident {
                pub hello: String,
            }
        }))
    }
}

/// Generate full file
pub struct GenerateFile;

impl SourceGenerator for GenerateFile {
    fn generate_file(
        &self,
        _args: syn::AttributeArgs,
        _file: &syn::File,
    ) -> Result<Option<TokenStream>, Error> {
        Ok(Some(quote! {
            #[doc = r" Some generated comment here"]
            struct Hello {
                pub hello: String,
            }
        }))
    }
}

/// Generates a struct with regular comments
pub struct GeneratePlainComments;

impl SourceGenerator for GeneratePlainComments {
    fn generate_struct(
        &self,
        _args: syn::AttributeArgs,
        item: &syn::ItemStruct,
    ) -> Result<Option<TokenStream>, Error> {
        let vis = &item.vis;
        let ident = &item.ident;
        Ok(Some(quote! {
            #PlainComment "This is some struct!"
            #vis struct #ident {
                #PlainComment "This is some field!"
                pub hello: String,
            }
        }))
    }
}

/// Generates a struct with a newline between struct and impl
pub struct GenerateNewLine;

impl SourceGenerator for GenerateNewLine {
    fn generate_struct(
        &self,
        _args: syn::AttributeArgs,
        item: &syn::ItemStruct,
    ) -> Result<Option<TokenStream>, Error> {
        let vis = &item.vis;
        let ident = &item.ident;
        Ok(Some(quote! {
            #vis struct #ident;
            #NewLine
            impl #ident {}
        }))
    }
}

/// Writes back the input without any changes
pub struct GenerateTrait;

impl SourceGenerator for GenerateTrait {
    fn generate_trait(
        &self,
        _args: syn::AttributeArgs,
        item: &syn::ItemTrait,
    ) -> Result<Option<TokenStream>, Error> {
        let vis = &item.vis;
        let ident = &item.ident;
        Ok(Some(quote! {
            /// Some generated comment here
            #vis trait #ident {
                fn hello();
            }
        }))
    }
}
