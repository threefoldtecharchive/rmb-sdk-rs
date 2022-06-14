use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn handler(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    if input.sig.asyncness.is_none() {
        panic!("supported only for async functions");
    }

    let name = &input.sig.ident;
    let out = quote! {
        #[allow(non_camel_case_types)]
        struct #name;

        #[async_trait::async_trait]
        impl Handler for #name {
            async fn call(&self, input: HandlerInput) -> Result<HandlerOutput> {
                #input

                #name(input).await
            }
        }
    };

    TokenStream::from(out)
}
