extern crate proc_macro;
extern crate proc_macro2;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;

/// ```rust
/// #[derive(MyError)]
/// enum Error {
///     #[my_error::source(io::Error)]
///     //#[my_error::display("Could not open config at {}: {}", self.filename.display(), self.source)]
///     OpenConfig { filename: PathBuf },
///     #[my_error::source(io::Error)]
///     SaveConfig {},
///     MissingUser,
/// }
///
/// # Terminology
/// - "selector"
/// ```
#[proc_macro_derive(MyError, attributes(my_error::display, my_error_display_compat))]
pub fn my_error_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Could not parse type to derive Error for");

    impl_hello_macro(ast)
}

struct EnumInfo {
    name: syn::Ident,
    variants: Vec<VariantInfo>,
}

struct VariantInfo {
    name: syn::Ident,
    source_field: Option<Field>,
    user_fields: Vec<Field>,
    display_format: Option<DisplayFormat>,
}

enum DisplayFormat {
    Direct(Box<dyn quote::ToTokens>),
    Stringified(Vec<Box<dyn quote::ToTokens>>),
}

struct Field {
    name: syn::Ident,
    ty: syn::Type,
}

fn impl_hello_macro(ty: syn::DeriveInput) -> TokenStream {
    let info = parse_my_error_information(ty);
    generate_my_error(info).into()
}

fn parse_my_error_information(ty: syn::DeriveInput) -> EnumInfo {
    use syn::{Data, Fields, Expr, Meta, NestedMeta, Lit};

    let enum_ = match ty.data {
        Data::Enum(enum_) => enum_,
        _ => panic!("Can only derive Error for an enum"),
    };

    let name = ty.ident;

    let variants = enum_.variants.into_iter().map(|variant| {
        let name = variant.ident;

        let display_format = variant.attrs.into_iter().map(|attr| {
            if is_my_error_display(&attr.path) {
                let expr: Expr = syn::parse2(attr.tts).expect("Need expression");
                let expr: Box<dyn quote::ToTokens> = match expr {
                    Expr::Tuple(expr_tuple) => Box::new(expr_tuple.elems),
                    Expr::Paren(expr_paren) => Box::new(expr_paren.expr),
                    _ => panic!("Requires a parenthesized format string and optional values"),
                };
                DisplayFormat::Direct(expr)
            } else if attr.path.is_ident("my_error_display_compat") {
                let meta = attr.parse_meta().expect("Improperly formed attribute");
                let meta = match meta {
                    Meta::List(list) => list,
                    _ => panic!("Only supports a list"),
                };
                let mut nested = meta.nested.into_iter().map(|nested| {
                    let nested = match nested {
                        NestedMeta::Literal(lit) => lit,
                        _ => panic!("Only supports a list of literals"),
                    };
                    match nested {
                        Lit::Str(s) => s,
                        _ => panic!("Only supports a list of literal strings"),
                    }
                });

                let fmt_str = nested.next().map(|x| Box::new(x) as Box<dyn quote::ToTokens>);

                let fmt_args = nested.map(|nested| {
                    nested.parse::<Expr>().expect("Strings after the first must be parsable as expressions")
                }).map(|x| Box::new(x) as Box<dyn quote::ToTokens>);


                let nested = fmt_str.into_iter().chain(fmt_args).collect();

                DisplayFormat::Stringified(nested)
            } else {
                panic!("Unknown attribute type");
            }
        }).next();

        let fields = match variant.fields {
            Fields::Named(f) => f.named.into_iter().collect(),
            Fields::Unnamed(_) => panic!("Tuple variants are not supported"),
            Fields::Unit => vec![],
        };

        let mut user_fields = Vec::new();
        let mut source_fields = Vec::new();

        for field in fields {
            let name = field.ident.expect("Must have a named field");
            let field = Field { name, ty: field.ty };

            if field.name == "source" {
                source_fields.push(field);
            } else {
                user_fields.push(field);
            }
        }

        let source_field = source_fields.pop();
        // Report a warning if there are multiple?

        VariantInfo { name, source_field, user_fields, display_format }
    }).collect();

    EnumInfo { name, variants }
}

fn is_my_error_display(p: &syn::Path) -> bool{
    let parts = ["my_error", "display"];
    p.segments.iter().zip(&parts).map(|(a, b)| a.ident == b).all(|b| b)
}


fn generate_my_error(enum_info: EnumInfo) -> proc_macro2::TokenStream {
    use syn::Ident;
    use proc_macro2::Span;

    let enum_name = enum_info.name;

    let generated_variant_support = enum_info.variants.iter().map(|variant| {
        let VariantInfo { name: variant_name, source_field, user_fields, .. } = variant;

        let generic_names: Vec<_> = (0..)
            .map(|i| Ident::new(&format!("T{}", i), Span::call_site()))
            .take(user_fields.len())
            .collect();
        let generic_names = &generic_names;

        let generics_list = quote! { <#(#generic_names),*> } ;
        let selector_name = quote! { #variant_name#generics_list };

        let names = user_fields.iter().map(|f| f.name.clone());
        let types = generic_names;

        let variant_selector_struct = {
            if user_fields.is_empty() {
                quote! {
                    struct #selector_name;
                }
            } else {
                quote! {
                    struct #selector_name {
                        #( #names: #types ),*
                    }
                }
            }
        };

        let enum_from_variant_selector_impl = match source_field {
            Some(source_field) => {
                let Field { name: source_name, ty: source_ty } = source_field;

                let other_ty = quote! {
                    my_error::Context<#source_ty, #selector_name>
                };

                let where_clauses = generic_names.iter().zip(user_fields).map(|(gen_ty, f)| {
                    let Field { ty, .. } = f;
                    quote! { #gen_ty: core::convert::Into<#ty> }
                });

                let user_fields = user_fields.iter().map(|f| {
                    let Field { name, .. } = f;
                    quote! { #name: other.context.#name.into() }
                });

                quote! {
                    impl#generics_list core::convert::From<#other_ty> for #enum_name
                    where
                        #(#where_clauses),*
                    {
                        fn from(other: #other_ty) -> Self {
                            #enum_name::#variant_name {
                                #source_name: other.error,
                                #(#user_fields),*
                            }
                        }
                    }
                }
            }
            None => {
                quote! {}
            }
        };

        quote! {
            #variant_selector_struct
            #enum_from_variant_selector_impl
        }
    });

    let variants = enum_info.variants.iter().map(|variant| {
        let VariantInfo { name: variant_name, user_fields, source_field, display_format, .. } = variant;

        let format = match display_format {
            Some(DisplayFormat::Stringified(fmt)) => {
                quote! { #(#fmt),* }
            }
            Some(DisplayFormat::Direct(fmt)) => {
                quote! { #fmt }
            }
            None => quote! { stringify!(#variant_name) },
        };


        let field_names = user_fields.iter().chain(source_field).map(|f| &f.name);
        let field_names = quote! { #(#field_names),* };

        quote! {
            #enum_name::#variant_name { #field_names } => {
                write!(f, #format)
            }
        }
    });

    let display_impl = quote! {
        impl core::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                #[allow(unused_variables)]
                match self {
                    #(#variants)*
                }
            }
        }
    };

    let variants = enum_info.variants.iter().map(|variant| {
        let VariantInfo { name: variant_name, source_field, .. } = variant;

        match source_field {
            Some(source_field) => {
                let Field { name: field_name, .. } = source_field;
                quote! {
                    #enum_name::#variant_name { #field_name, .. } => { Some(#field_name) }
                }
            }
            None => {
                quote! {
                    #enum_name::#variant_name { .. } => { None }
                }
            }
        }
    });

    let error_impl = quote! {
        impl std::error::Error for #enum_name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    #(#variants)*
                }
            }
        }
    };

    quote! {
        #(#generated_variant_support)*
        #display_impl
        #error_impl
    }
}
