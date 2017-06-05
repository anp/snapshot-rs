#![feature(proc_macro)]
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use std::str::FromStr;
use proc_macro::TokenStream;
use syn::*;

#[proc_macro_attribute]
pub fn snapshot(attribute: TokenStream, function: TokenStream) -> TokenStream {
    let src = function.to_string();
    let function = proc_macro2::TokenStream::from(function);
    let Item { attrs, node } = function.into();
    let ItemFn {
        ident, // TODO get the function name out of this bad boy
        unsafety,
        constness,
        abi,
        block,
        decl,
        ..
    } = match node {
        ItemKind::Fn(item) => item,
        _ => panic!("#[snapshot] can only be applied to functions"),
    };
    let FnDecl {
        inputs,
        output,
        variadic,
        generics,
        ..
    } = {
        *decl
    };

    // TODO get the function name!!!!
    panic!("haha we're finding the function name");


    <TokenStream as FromStr>::from_str(&src).unwrap()
}
