use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Token;
use syn::parse::Parse;
use syn::parse_macro_input;

#[proc_macro]
pub fn vecn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let shape = parse_macro_input!(input as NVecShape);
    quote::quote! {
        n_vec::new_array::<{ #shape }>()
    }
    .into()
}

enum NVecShape {
    Literal(syn::LitInt),
    ConstGeneric(syn::Ident),
}

impl Parse for NVecShape {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::LitInt) {
            Ok(NVecShape::Literal(input.parse()?))
        } else if lookahead.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            Ok(NVecShape::ConstGeneric(ident))
        } else {
            syn::Result::Err(syn::Error::new(
                input.span(),
                "Expected literal or const generic integer",
            ))
        }
    }
}

impl ToTokens for NVecShape {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NVecShape::Literal(lit) => {
                lit.to_tokens(tokens);
            }
            NVecShape::ConstGeneric(ident) => {
                ident.to_tokens(tokens);
            }
        }
    }
}
