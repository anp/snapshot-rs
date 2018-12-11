#![recursion_limit = "128"]

// Still need this extern in 2018 (https://github.com/rust-lang/rust/pull/54116)
extern crate proc_macro;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::Item;

#[proc_macro_attribute]
pub fn snapshot(_: TokenStream, function: TokenStream) -> TokenStream {
    let mut inner_fn: Item = syn::parse(function.into()).unwrap();

    // swap the inner/outer function names in the Item
    let mut fn_item = match inner_fn {
        Item::Fn(ref mut item) => item,
        _ => panic!("#[snapshot] can only be applied to functions"),
    };

    // TODO check for generics, input variables, etc.

    let outer_fn_token = fn_item.ident.clone();
    let outer_fn_name = outer_fn_token.to_string();
    let inner_fn_name = format!("__snapshot_inner_{}", outer_fn_token);
    let inner_fn_token = syn::Ident::new(&inner_fn_name, fn_item.ident.span());

    fn_item.ident = inner_fn_token.clone();

    let output = quote! {
        #[test]
        fn #outer_fn_token() {
            #inner_fn

            // run the user's snapshot test first, in case it panics
            let recorded_value = #inner_fn_token();

            let file = file!().to_owned();
            let module_path = module_path!().to_owned();
            let test_function = (#outer_fn_name).to_owned();

            let snapshot = ::snapshot::Snapshot::new(
                file, module_path, test_function, recorded_value,
            );

            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            if let Ok(_) = ::std::env::var("UPDATE_SNAPSHOTS") {
                snapshot.update_snapshot(manifest_dir);
            } else {
                snapshot.check_snapshot(manifest_dir);
            }
        }
    };

    output.into()
}
