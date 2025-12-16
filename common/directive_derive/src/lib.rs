// This module defines a derive macro to generate code for creating specific directives from
// structs that define their argument types. Additional requirements:
//   - Type must have a `span` field of type `Span`
//   - Type must have a validate() method.

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Data, DataStruct, DeriveInput, GenericArgument, PathArguments, Type, TypeGroup,
    TypeParen, TypePath,
};
use syn::{DataEnum, parse_macro_input};

// This function was initially AI-generated. Given a type, and wrapper type X, if the type is X<T>,
// it returns Some(T).
fn option_inner_type<'a>(outer: &'static str, ty: &'a Type) -> Option<&'a Type> {
    fn strip(ty: &Type) -> &Type {
        match ty {
            Type::Group(TypeGroup { elem, .. }) => strip(elem),
            Type::Paren(TypeParen { elem, .. }) => strip(elem),
            _ => ty,
        }
    }
    let ty = strip(ty);
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };
    let last = path.segments.last()?;
    if last.ident != outer {
        return None;
    }
    match &last.arguments {
        PathArguments::AngleBracketed(ab) => {
            if let Some(GenericArgument::Type(inner)) = ab.args.first() {
                Some(inner)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[proc_macro_derive(FromRawDirective)]
pub fn from_raw_directive_derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    // Dispatch based on whether this is a struct or an enum.
    match &derive_input.data {
        Data::Struct(data) => from_raw_struct(&derive_input, data),
        Data::Enum(data) => from_raw_enum(&derive_input, data),
        _ => syn::Error::new_spanned(&derive_input.ident, "only struct and enum is supported")
            .to_compile_error(),
    }
    .into()
}

fn from_raw_struct(input: &DeriveInput, data: &DataStruct) -> proc_macro2::TokenStream {
    // Iterate through all the fields in the structure. Fields of type Option are for optional
    // parameters. Fields of type Vec are for optional, repeatable parameters. Other fields are
    // for required parameters. The generated code uses the CheckType trait to test for the correct
    // argument. If the generated function is able to fully initialize the struct from the raw
    // directive, it returns a Some value. Otherwise, it returns None. This ensures that additional
    // semantic validation can safely operate on the fully-initialized type. As a special case,
    // fields with specified names that correspond to data blocks are handled as a special case:
    // if a data block field is present, require a data block to follow the directive. Otherwise,
    // it is an error if a data block follows the directive.

    enum DataBlockType {
        None,
        Scale,
        Layout,
    }

    let top_name = &input.ident;
    let directive_name = top_name.to_string().to_snake_case();
    let mut var_decls = Vec::new();
    let mut arg_checks = Vec::new();
    let mut required_checks = Vec::new();
    let mut inits = Vec::new();
    let mut help_statements = vec![quote! {
        write!(w, "\n*** '{}' ***\n", #directive_name)?;
    }];
    let mut wanted_data_block = DataBlockType::None;
    let mut top_doc_comment = String::new();
    get_doc_comment(&input.attrs, &mut top_doc_comment, "  ");
    if !top_doc_comment.is_empty() {
        help_statements.push(quote! {
            write!(w, "{}", #top_doc_comment)?;
        });
    }
    if !data.fields.is_empty() {
        help_statements.push(quote! {
            write!(w, "  ---\n")?;
        });
    }

    for f in &data.fields {
        let field_name = f.ident.as_ref().unwrap();
        if *field_name == "span" {
            // Special case
            continue;
        }
        if *field_name == "_s" {
            inits.push(quote! { _s: &(), });
            continue;
        }
        let field_type = &f.ty;
        if *field_name == "scale_block" || *field_name == "layout_block" {
            var_decls.push(quote! {
                let mut #field_name: Option<#field_type> = None;
            });
            inits.push(quote! { #field_name: #field_name? });
            if *field_name == "scale_block" {
                wanted_data_block = match wanted_data_block {
                    DataBlockType::None => DataBlockType::Scale,
                    _ => panic!("at most one data block field may appear"),
                };
                help_statements.push(quote! {
                    write!(w, "  This directive must be followed by a scale block.\n")?;
                });
            }
            if *field_name == "layout_block" {
                wanted_data_block = match wanted_data_block {
                    DataBlockType::None => DataBlockType::Layout,
                    _ => panic!("at most one data block field may appear"),
                };
                help_statements.push(quote! {
                    write!(w, "  This directive must be followed by a layout block.\n")?;
                });
            }
            continue;
        }
        let option_type = option_inner_type("Option", field_type);
        let vec_type = option_inner_type("Vec", field_type);
        let var_type = option_type.unwrap_or(field_type); // contains garbage if Vec
        let is_required = vec_type.is_none() && option_type.is_none();

        // Generate code fragments. These are in context of the generated function (at the end).

        let mut doc_comment = String::new();
        get_doc_comment(&f.attrs, &mut doc_comment, "    ");
        let field_name_str = field_name.to_string();
        let qualifier = if vec_type.is_some() {
            " (repeatable)"
        } else if option_type.is_some() {
            " (optional)"
        } else {
            ""
        };
        help_statements.push(quote! {
            write!(w, "  {}{}\n", #field_name_str, #qualifier)?;
        });
        if !doc_comment.is_empty() {
            help_statements.push(quote! {
                write!(w, "{}", #doc_comment)?;
            });
        }

        // Create a local variable that gets initialized if the argument is encountered.
        if let Some(inner) = vec_type {
            var_decls.push(quote! {
                let mut #field_name = Vec::<#inner>::new();
            });
        } else {
            var_decls.push(quote! {
                let mut #field_name: Option<#var_type> = None;
            });
        }

        let arg_check = if vec_type.is_some() {
            quote! {
                if let Some(x) = score_helpers::check_value(diags, &d.name.value, &p) {
                    #field_name.push(x);
                }
            }
        } else {
            quote! {
                if #field_name.is_some() {
                    diags.err(
                        code::DIRECTIVE_USAGE,
                        k.span,
                        format!(
                            "'{}': parameter '{}' is not repeatable",
                            d.name.value,
                            stringify!(#field_name),
                        ),
                    );
                }
                #field_name = score_helpers::check_value(diags, &d.name.value, &p);
            }
        };

        // Generate the code that checks the parameter type and initializes.
        arg_checks.push(quote! {
            if k.value == stringify!(#field_name) {
                params_seen.insert(k.value.to_string());
                handled = true;
                #arg_check
            }
        });

        // Generate code for each required field to ensure that the parameter was seen.
        let required_check = if is_required {
            quote! {
                if !params_seen.contains(stringify!(#field_name)) {
                    diags.err(
                        code::DIRECTIVE_USAGE,
                        d.name.span,
                        format!("'{}': missing parameter '{}'", d.name.value, stringify!(#field_name)),
                    );
                }
            }
        } else {
            quote! {}
        };
        required_checks.push(required_check);

        // Generate a code fragment that initializes the struct field from the local variable.
        // Using `?` for required types ensures we return `None` if any are missing. This is done
        // after missing parameters have already been reported.
        let field_init = if option_type.is_none() && vec_type.is_none() {
            quote! {: #field_name?}
        } else {
            quote! {}
        };
        inits.push(quote! {
            #field_name #field_init,
        });
    }

    let block_check = match wanted_data_block {
        DataBlockType::None => quote! {
            if let Some(block) = &d.block {
                diags.err(
                    code::DIRECTIVE_SYNTAX,
                    block.span,
                    "a data block is not expected here",
                );
            }
        },
        DataBlockType::Scale => quote! {
            if let Some(x) = d.block.clone() {
                match x.value {
                    DataBlock::Scale(s) => {
                        scale_block = Some(Spanned::new(x.span, s));
                    }
                    _ => {}
                }
            }
            if scale_block.is_none() {
                let mut diag = Diagnostic::new(
                    code::DIRECTIVE_SYNTAX,
                    span,
                    "this directive must be followed by a scale block",
                );
                if let Some(s) = d.block.as_ref().map(|x| x.span) {
                    diag = diag.with_context(s, "this is not a scale block");
                }
                diags.push(diag);
                return None;
            }
        },
        DataBlockType::Layout => quote! {
            if let Some(x) = d.block.clone() {
                match x.value {
                    DataBlock::Layout(s) => {
                        layout_block = Some(Spanned::new(x.span, s));
                    }
                    _ => {}
                }
            }
            if layout_block.is_none() {
                let mut diag = Diagnostic::new(
                    code::DIRECTIVE_SYNTAX,
                    span,
                    "this directive must be followed by a layout block",
                );
                if let Some(s) = d.block.as_ref().map(|x| x.span) {
                    diag = diag.with_context(s, "this is not a layout block");
                }
                diags.push(diag);
                return None;
            }
        },
    };
    required_checks.push(block_check);

    quote! {
        impl<'s> FromRawDirective<'s> for #top_name<'s> {
            fn from_raw(diags: &Diagnostics, span: Span, d: &RawDirective<'s>) -> Option<Self> {
                let mut params_seen = HashSet::new();
                #(#var_decls)*
                for p in &d.params {
                    let mut handled = false;
                    let k = &p.key;
                    let v = &p.value;
                    #(#arg_checks)*
                    if !handled {
                        diags.err(
                            code::UNKNOWN_DIRECTIVE_PARAM,
                            p.key.span,
                            format!("'{}': unknown parameter '{}'", d.name.value, p.key.value),
                        );
                    }
                }
                #(#required_checks)*
                let mut r = Self {
                    span: d.name.span,
                    #(#inits)*
                };
                let before = diags.num_errors();
                r.validate(diags);
                if diags.num_errors() > before {
                    None
                } else {
                    Some(r)
                }
            }

            fn show_help(w: &mut impl io::Write) -> io::Result<()> {
                #(#help_statements)*
                Ok(())
            }
        }
    }
}

fn from_raw_enum(input: &DeriveInput, data: &DataEnum) -> proc_macro2::TokenStream {
    // Iterate through all the enum variants. Generate code that calls the appropriated initializer
    // based on the name of the directive.

    let top_name = &input.ident;
    let mut match_arms = Vec::new();
    let mut help_calls = Vec::new();

    for v in &data.variants {
        let variant = &v.ident;
        let field = v.fields.iter().next().unwrap();
        let field_type = &field.ty;
        let directive_name = field_type
            .to_token_stream()
            .into_iter()
            .next()
            .unwrap()
            .to_string()
            .to_snake_case();
        match_arms.push(quote! {
            #directive_name => {
                Some(#top_name::#variant(<#field_type as FromRawDirective>::from_raw(diags, span, d)?))
            }
        });
        help_calls.push(quote! {
            <#field_type as FromRawDirective>::show_help(w)?;
        });
    }

    quote! {
        impl<'s> FromRawDirective<'s> for #top_name<'s> {
            fn from_raw(diags: &Diagnostics, span: Span, d: &RawDirective<'s>) -> Option<Self> {
                match d.name.value.as_ref() {
                    #(#match_arms)*
                    _ => {
                        diags.err(
                            code::UNKNOWN_DIRECTIVE,
                            d.name.span,
                            format!("unknown directive '{}'", d.name.value),
                        );
                        None
                    }
                }
            }

            fn show_help(w: &mut impl io::Write) -> io::Result<()> {
                #(#help_calls)*
                Ok(())
            }
        }
    }
}

fn get_doc_comment(attrs: &[Attribute], doc_comment: &mut String, indent: &'static str) {
    for attr in attrs {
        if attr.path().is_ident("doc")
            && let syn::Meta::NameValue(nv) = &attr.meta
            && let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &nv.value
        {
            let s_value = s.value();
            let mut value = s_value.trim_end();
            // Strip at most one space from the beginning to allow lists and indented structures
            // to generate correctly.
            if value.starts_with(' ') {
                value = &value[1..];
            }
            if !value.is_empty() {
                doc_comment.push_str(indent);
                doc_comment.push_str(value);
            }
            doc_comment.push('\n');
        }
    }
}
