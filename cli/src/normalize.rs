use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use syn::Lit;

/// Write tokens same way as `TokenStream::to_string` would do, but with normalization of doc
/// attributes into `///`.
pub fn write_tokens_normalized(
    f: &mut std::fmt::Formatter,
    tokens: TokenStream,
) -> std::fmt::Result {
    let mut tokens = tokens.into_iter().peekable();
    let mut joint = false;
    let mut first = true;
    let mut plain_comment = false;
    while let Some(tt) = tokens.next() {
        if !first && !joint {
            write!(f, " ")?;
        }
        first = false;
        joint = false;

        // Handle plain comments
        if plain_comment {
            if let TokenTree::Literal(lit) = tt {
                if let Lit::Str(lit) = Lit::new(lit) {
                    writeln!(f, "// {}", lit.value())?;
                }
            }
            plain_comment = false;
            continue;
        }
        if let Some(comment) = tokens
            .peek()
            .and_then(|lookahead| as_doc_comment(&tt, lookahead))
        {
            let _ignore = tokens.next();
            writeln!(f, "///{}", comment)?;
            continue;
        }
        match tt {
            TokenTree::Group(ref tt) => {
                let (start, end) = match tt.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::None => ("", ""),
                };
                if tt.stream().into_iter().next().is_none() {
                    write!(f, "{} {}", start, end)?
                } else {
                    write!(f, "{} ", start)?;
                    write_tokens_normalized(f, tt.stream())?;
                    write!(f, " {}", end)?
                }
            }
            TokenTree::Ident(ref tt) if tt == crate::tokens::MAGIC_NEWLINE_IDENT => {
                writeln!(f)?;
                writeln!(f)?;
            }
            TokenTree::Ident(ref tt) if tt == crate::tokens::MAGIC_COMMENT_IDENT => {
                plain_comment = true;
            }
            TokenTree::Ident(ref tt) => write!(f, "{}", tt)?,
            TokenTree::Punct(ref tt) => {
                write!(f, "{}", tt.as_char())?;
                match tt.spacing() {
                    Spacing::Alone => {}
                    Spacing::Joint => joint = true,
                }
            }
            TokenTree::Literal(ref tt) => write!(f, "{}", tt)?,
        }
    }
    Ok(())
}

fn as_doc_comment(first: &TokenTree, second: &TokenTree) -> Option<String> {
    match (first, second) {
        (TokenTree::Punct(first), TokenTree::Group(group))
            if first.as_char() == '#' && group.delimiter() == Delimiter::Bracket =>
        {
            let mut it = group.stream().into_iter();
            match (it.next(), it.next(), it.next()) {
                (
                    Some(TokenTree::Ident(ident)),
                    Some(TokenTree::Punct(punct)),
                    Some(TokenTree::Literal(lit)),
                ) => {
                    if ident == "doc" && punct.as_char() == '=' {
                        if let Lit::Str(lit) = Lit::new(lit) {
                            return Some(lit.value());
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    None
}
