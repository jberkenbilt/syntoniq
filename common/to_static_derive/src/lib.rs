// This module defines a derive macro to generate code for implementing ToStatic. All fields and
// enum arms must implement ToStatic. This is not intended to be general purposes. It makes a few
// assumptions:
// - If something has a lifetime parameter, it is `'s`
// - Types have no other generic parameters

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, GenericParam, Generics};
use syn::{DataEnum, parse_macro_input};

struct LtTokens {
    orig_lt: TokenStream2,
    static_lt: TokenStream2,
    fish_lt: TokenStream2,
}

#[proc_macro_derive(ToStatic)]
pub fn from_raw_directive_derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    match &derive_input.data {
        Data::Struct(data) => to_static_struct(&derive_input, data),
        Data::Enum(data) => to_static_enum(&derive_input, data),
        _ => syn::Error::new_spanned(&derive_input.ident, "only struct and enum is supported")
            .to_compile_error(),
    }
    .into()
}

fn to_static_struct(input: &DeriveInput, data: &DataStruct) -> proc_macro2::TokenStream {
    let top_name = &input.ident;
    let mut field_inits = Vec::new();

    for f in &data.fields {
        let field_name = f.ident.as_ref().unwrap();
        field_inits.push(quote! {
            #field_name: self.#field_name.to_static(arc_context),
        });
    }

    let LtTokens {
        orig_lt,
        static_lt,
        fish_lt,
    } = lt_tokens(&input.generics);
    quote! {
         impl<'s> crate::parsing::score_helpers::ToStatic<'s> for #top_name #orig_lt {
            type Static = #top_name #static_lt;
            fn to_static(&self, arc_context: &mut crate::parsing::score_helpers::ArcContext) -> Self::Static {
                #top_name #fish_lt {
                    #(#field_inits)*
                }
            }
         }
    }
}

fn to_static_enum(input: &DeriveInput, data: &DataEnum) -> proc_macro2::TokenStream {
    let top_name = &input.ident;
    let mut match_arms = Vec::new();
    let LtTokens {
        orig_lt,
        static_lt,
        fish_lt,
    } = lt_tokens(&input.generics);

    for v in &data.variants {
        let variant = &v.ident;
        match_arms.push(quote! {
            #top_name::#variant(x) => #top_name #fish_lt::#variant(x.to_static(arc_context)),
        });
    }

    quote! {
         impl<'s> crate::parsing::score_helpers::ToStatic<'s> for #top_name #orig_lt {
            type Static = #top_name #static_lt;
            fn to_static(&self, arc_context: &mut crate::parsing::score_helpers::ArcContext) -> Self::Static {
                match self {
                    #(#match_arms)*
                }
            }
         }
    }
}

fn lt_tokens(generics: &Generics) -> LtTokens {
    let mut has_lifetime = false;
    for p in &generics.params {
        if matches!(p, GenericParam::Lifetime(_)) {
            has_lifetime = true;
            break;
        }
    }

    let orig_lt = if has_lifetime {
        quote! { <'s> }
    } else {
        quote! {}
    };
    let static_lt = if has_lifetime {
        quote! { <'static> }
    } else {
        quote! {}
    };
    let fish_lt = if has_lifetime {
        quote! { ::<'static> }
    } else {
        quote! {}
    };
    LtTokens {
        orig_lt,
        static_lt,
        fish_lt,
    }
}
