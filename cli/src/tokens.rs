use proc_macro2::{Ident, Span, TokenStream, TokenTree};

pub(crate) const MAGIC_COMMENT_IDENT: &str = "__SOURCEGEN_MAGIC_COMMENT__";

/// Token used to generate plain Rust comments in the output. Used as a marker in front of the
/// string literal to generate a plain comment. Usage:
///
/// ```rust
/// use sourcegen_cli::tokens::PlainComment;
/// let _output = quote::quote! {
///     #PlainComment "GeneratedComment"
///     struct Test;
/// };
/// ```
///
/// Generated output will contain a plain comment:
/// ```
/// // Generated comment
/// struct Test;
/// ```
pub struct PlainComment;

impl quote::ToTokens for PlainComment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(std::iter::once(TokenTree::Ident(Ident::new(
            MAGIC_COMMENT_IDENT,
            Span::call_site(),
        ))));
    }
}

pub(crate) const MAGIC_NEWLINE_IDENT: &str = "__SOURCEGEN_MAGIC_NEWLINE__";

/// Token used to generate a newline in the output. Used as a marker. Usage:
///
/// ```rust
/// use sourcegen_cli::tokens::NewLine;
/// let _output = quote::quote! {
///     struct Frist;
///     #NewLine
///     struct Second;
/// };
/// ```
///
/// Generated output will contain a plain comment:
/// ```
/// struct First;
///
/// struct Second;
/// ```
pub struct NewLine;

impl quote::ToTokens for NewLine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(std::iter::once(TokenTree::Ident(Ident::new(
            MAGIC_NEWLINE_IDENT,
            Span::call_site(),
        ))));
    }
}
