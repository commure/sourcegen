extern crate proc_macro;

/// Does nothing (returns item as-is). Needed to remove the attribute that is handled by source generator.
#[proc_macro_attribute]
pub fn sourcegen(
    _attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

#[proc_macro_attribute]
pub fn generated(
    _attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}
