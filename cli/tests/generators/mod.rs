use failure::Error;
use proc_macro2::TokenStream;
use quote::quote;
use sourcegen_cli::SourceGenerator;

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
            impl Boo for #ident {}
        }))
    }
}
