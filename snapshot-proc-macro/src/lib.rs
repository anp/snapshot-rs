#![feature(proc_macro)]
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use syn::*;

#[proc_macro_attribute]
pub fn snapshot(_: TokenStream, function: TokenStream) -> TokenStream {
    let src = function.to_string();
    let function = proc_macro2::TokenStream::from(function);
    let mut inner_fn: Item = function.into();

    // swap the inner/outer function names in the Item
    let (outer_fn_token, outer_fn_name, inner_fn_name, inner_fn_token) = {
        let mut fn_item = match inner_fn.node {
            ItemKind::Fn(ref mut item) => item,
            _ => panic!("#[snapshot] can only be applied to functions"),
        };

        // TODO swap the inner and outer fn names
        let outer_fn_token = fn_item.ident.clone();
        let outer_fn_name = outer_fn_token.to_string();
        let inner_fn_name = format!("__snapshot_inner_{}", outer_fn_token);
        let inner_fn_token = syn::Ident::from(inner_fn_name.clone());

        fn_item.ident = inner_fn_token.clone();

        (outer_fn_token, outer_fn_name, inner_fn_name, inner_fn_token)
    };

    let output = quote! {
        #[test]
        fn #outer_fn_token() {
            #inner_fn

            // run the user's snapshot test first, in case it panics
            let current_result = #inner_fn_token();

            use ::snapshot::Snapable;

            let file = file!();
            let module_path = module_path!();
            let test_function = #outer_fn_name;

            let metadata = ::snapshot::Metadata {
                file, module_path, test_function,
            };

            let (mut snap_path, snap_file) = metadata.path(env!("CARGO_MANIFEST_DIR"));

            snap_path.push(snap_file);
            panic!("{:?}", snap_path);

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
