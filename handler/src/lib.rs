use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Type};

#[proc_macro_attribute]
pub fn handler(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    if input.sig.asyncness.is_none() {
        panic!("supported only for async functions");
    }

    let args = &input.sig.inputs;
    assert_eq!(args.len(), 2, "invalid number of arguments expecting two");

    let data = match args.first() {
        None => {
            panic!("require 2 arguments");
        }
        Some(data) => match data {
            FnArg::Typed(ty) => ty,
            _ => {
                panic!("invalid argument type expected struct")
            }
        },
    };

    let p = match *data.ty {
        Type::Path(ref p) => p,
        _ => {
            panic!("expecting struct data");
        }
    };

    let name = &input.sig.ident;
    let d = &p.path;
    let out = quote! {

        #[allow(non_camel_case_types)]
        pub struct #name;

        #[async_trait::async_trait]
==== BASE ====
        impl Handler for #name {
            async fn call(&self, input: HandlerInput) -> Result<HandlerOutput> {
==== BASE ====
                #input

                #name(data, input).await
            }
        }

    };

    TokenStream::from(out)
}
