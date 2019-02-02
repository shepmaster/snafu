extern crate proc_macro2;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

/// See the crate-level documentation for SNAFU which contains tested
/// examples of this macro.

#[cfg_attr(not(feature = "unstable_display_attribute"),
           proc_macro_derive(Snafu, attributes(snafu_display)))]
#[cfg_attr(feature = "unstable_display_attribute",
           proc_macro_derive(Snafu, attributes(snafu::display, snafu_display)))]
pub fn snafu_derive(input: TokenStream) -> TokenStream {
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
    Direct(Box<quote::ToTokens>),
    Stringified(Vec<Box<quote::ToTokens>>),
}

struct Field {
    name: syn::Ident,
    ty: syn::Type,
}

fn impl_hello_macro(ty: syn::DeriveInput) -> TokenStream {
    let info = parse_snafu_information(ty);
    generate_snafu(info).into()
}

fn parse_snafu_information(ty: syn::DeriveInput) -> EnumInfo {
    use syn::{Data, Fields, Meta};

    let enum_ = match ty.data {
        Data::Enum(enum_) => enum_,
        _ => panic!("Can only derive Error for an enum"),
    };

    let name = ty.ident;

    let variants = enum_.variants.into_iter().map(|variant| {
        let name = variant.ident;

        let display_format = variant.attrs.into_iter().map(|attr| {
            if is_snafu_display(&attr.path) {
                parse_snafu_display_beautiful(attr)
            } else if attr.path.is_ident("snafu_display") {
                let meta = attr.parse_meta().expect("`snafu_display` attribute is malformed");
                match meta {
                    Meta::List(list) => parse_snafu_display_nested(list),
                    Meta::NameValue(nv) => parse_snafu_display_nested_name_value(nv),
                    _ => panic!("Only supports a list"),
                }
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

fn is_snafu_display(p: &syn::Path) -> bool{
    let parts = ["snafu", "display"];
    p.segments.iter().zip(&parts).map(|(a, b)| a.ident == b).all(|b| b)
}

fn parse_snafu_display_beautiful(attr: syn::Attribute) -> DisplayFormat {
    use syn::Expr;

    let expr: Expr = syn::parse2(attr.tts).expect("Need expression");
    let expr: Box<quote::ToTokens> = match expr {
        Expr::Tuple(expr_tuple) => Box::new(expr_tuple.elems),
        Expr::Paren(expr_paren) => Box::new(expr_paren.expr),
        _ => panic!("Requires a parenthesized format string and optional values"),
    };
    DisplayFormat::Direct(expr)
}

fn parse_snafu_display_nested(meta: syn::MetaList) -> DisplayFormat {
    use syn::{Expr, NestedMeta, Lit};

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

    let fmt_str = nested.next().map(|x| Box::new(x) as Box<quote::ToTokens>);

    let fmt_args = nested.map(|nested| {
        nested.parse::<Expr>().expect("Strings after the first must be parsable as expressions")
    }).map(|x| Box::new(x) as Box<quote::ToTokens>);

    let nested = fmt_str.into_iter().chain(fmt_args).collect();

    DisplayFormat::Stringified(nested)
}

fn parse_snafu_display_nested_name_value(nv: syn::MetaNameValue) -> DisplayFormat {
    use syn::{Lit, Expr};

    let s = match nv.lit {
        Lit::Str(s) => s,
        _ => panic!("Only supports a litera strings"),
    };

    let expr = s.parse::<Expr>().expect("Must be a parsable as an expression");

    let expr: Box<quote::ToTokens> = match expr {
        Expr::Tuple(expr_tuple) => Box::new(expr_tuple.elems),
        Expr::Paren(expr_paren) => Box::new(expr_paren.expr),
        _ => panic!("Requires a parenthesized format string and optional values"),
    };

    DisplayFormat::Direct(expr)
}

fn generate_snafu(enum_info: EnumInfo) -> proc_macro2::TokenStream {
    use syn::Ident;
    use proc_macro2::Span;

    let enum_name = enum_info.name;

    let generated_variant_support = enum_info.variants.iter().map(|variant| {
        let VariantInfo { name: ref variant_name, ref source_field, ref user_fields, .. } = *variant;

        let generic_names: Vec<_> = (0..)
            .map(|i| Ident::new(&format!("T{}", i), Span::call_site()))
            .take(user_fields.len())
            .collect();
        let generic_names = &generic_names;

        let generics_list = quote! { <#(#generic_names),*> } ;
        let selector_name = quote! { #variant_name#generics_list };

        let names: Vec<_> = user_fields.iter().map(|f| f.name.clone()).collect();
        let names = &names;
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

        let where_clauses: Vec<_> = generic_names.iter().zip(user_fields).map(|(gen_ty, f)| {
            let Field { ref ty, .. } = *f;
            quote! { #gen_ty: std::convert::Into<#ty> }
        }).collect();
        let where_clauses = &where_clauses;

        let inherent_impl = if source_field.is_none() {
            let names2 = names;
            quote! {
                impl#generics_list #selector_name
                where
                    #(#where_clauses),*
                {
                    fn fail<T>(self) -> std::result::Result<T, #enum_name> {
                        let Self { #(#names),* } = self;
                        let error = #enum_name::#variant_name {
                            #( #names: std::convert::Into::into(#names2) ),*
                        };
                        std::result::Result::Err(error)
                    }
                }
            }
        } else {
            quote! {}
        };

        let enum_from_variant_selector_impl = match *source_field {
            Some(ref source_field) => {
                let Field { name: ref source_name, ty: ref source_ty } = *source_field;

                let other_ty = quote! {
                    snafu::Context<#source_ty, #selector_name>
                };

                let user_fields = user_fields.iter().map(|f| {
                    let Field { ref name, .. } = *f;
                    quote! { #name: other.context.#name.into() }
                });

                quote! {
                    impl#generics_list std::convert::From<#other_ty> for #enum_name
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
            #inherent_impl
            #enum_from_variant_selector_impl
        }
    });

    let variants = enum_info.variants.iter().map(|variant| {
        let VariantInfo { name: ref variant_name, ref user_fields, ref source_field, ref display_format, .. } = *variant;

        let format = match *display_format {
            Some(DisplayFormat::Stringified(ref fmt)) => {
                quote! { #(#fmt),* }
            }
            Some(DisplayFormat::Direct(ref fmt)) => {
                quote! { #fmt }
            }
            None => quote! { stringify!(#variant_name) },
        };


        let field_names = user_fields.iter().chain(source_field).map(|f| &f.name);
        let field_names = quote! { #(ref #field_names),* };

        quote! {
            #enum_name::#variant_name { #field_names } => {
                write!(f, #format)
            }
        }
    });

    let display_impl = quote! {
        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                #[allow(unused_variables)]
                match *self {
                    #(#variants)*
                }
            }
        }
    };

    let variants = enum_info.variants.iter().map(|variant| {
        let VariantInfo { name: ref variant_name, .. } = *variant;
        quote! {
            #enum_name::#variant_name { .. } => stringify!(#enum_name::#variant_name),
        }
    });

    let description_fn = quote! {
        fn description(&self) -> &str {
            match *self {
                #(#variants)*
            }
        }
    };

    let variants: Vec<_> = enum_info.variants.iter().map(|variant| {
        let VariantInfo { name: ref variant_name, ref source_field, .. } = *variant;

        match *source_field {
            Some(ref source_field) => {
                let Field { name: ref field_name, .. } = *source_field;
                quote! {
                    #enum_name::#variant_name { ref #field_name, .. } => {
                        Some(std::borrow::Borrow::borrow(#field_name))
                    }
                }
            }
            None => {
                quote! {
                    #enum_name::#variant_name { .. } => { None }
                }
            }
        }
    }).collect();
    let variants = &variants;

    let cause_fn = quote! {
        fn cause(&self) -> Option<&std::error::Error> {
            match *self {
                #(#variants)*
            }
        }
    };

    let source_fn = if cfg!(feature = "rust_1_30") {
        quote! {
            fn source(&self) -> Option<&(std::error::Error + 'static)> {
                match *self {
                    #(#variants)*
                }
            }
        }
    } else {
        quote! {}
    };

    let error_impl = quote! {
        impl std::error::Error for #enum_name {
            #description_fn
            #cause_fn
            #source_fn
        }
    };

    quote! {
        #(#generated_variant_support)*
        #display_impl
        #error_impl
    }
}
