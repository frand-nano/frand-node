use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::*;

mod node;

#[proc_macro_derive(Node)]
pub fn node(item: TokenStream) -> TokenStream {   
    let state = parse_macro_input!(item as ItemStruct);

    let node = node::expand(&state)
    .unwrap_or_else(Error::into_compile_error);

    quote! { 
        #node
    }.into()
}

#[proc_macro_derive(NodeMacro)]
pub fn node_macro(item: TokenStream) -> TokenStream {
    let node: proc_macro2::TokenStream = node(item.clone()).into();
    let state = parse_macro_input!(item as ItemStruct);

    let macro_name = Ident::new(
        &format!("{}_node", state.ident.to_string()).to_case(Case::Snake), 
        state.ident.span(),
    );
    
    #[cfg(debug_assertions)]
    let result = quote::quote! { 
        #[allow(unused_macros)]
        macro_rules! #macro_name {
            () => { #node }
        }
    }.into();

    #[cfg(not(debug_assertions))]
    let result = quote::quote! { 
        #[allow(unused_macros)]
        macro_rules! #macro_name {
            () => { }
        }
        #node
    }.into();

    result
}