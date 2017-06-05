#![feature(proc_macro)]
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use std::str::FromStr;
use proc_macro::TokenStream;
use syn::*;

#[proc_macro_attribute]
pub fn snapshot(attribute: TokenStream, function: TokenStream) -> TokenStream {
    let src = function.to_string();
    let function = proc_macro2::TokenStream::from(function);
    let Item { node, .. } = function.into();
    let ItemFn { ident: outer_fn_name, .. } = match node {
        ItemKind::Fn(item) => item,
        _ => panic!("#[snapshot] can only be applied to functions"),
    };

    // TODO swap the inner and outer fn names
    let inner_fn_name = format!("__snapshot_inner_{}", &outer_fn_name.to_string());
    let inner_fn_token = syn::Ident::from(inner_fn_name.clone());
    let output = quote! {
        #[test]
        fn #outer_fn_name() {
            let file = file!();
            let module_path = module_path!();
            let test_function = #inner_fn_name;

            let metadata = ::snapshot::Metadata {
                file, module_path, test_function,
            };

            let current_result = #inner_fn_token();

            if let Ok(_) = ::std::env::var("UPDATE_SNAPSHOTS") {
                // TODO update snapshots
            } else {
                // TODO check snapshots
            }
        }
    };

    let output: proc_macro2::TokenStream = output.into();
    output.into()
}
