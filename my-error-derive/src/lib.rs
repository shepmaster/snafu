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
#[proc_macro_derive(MyError, attributes(my_error::display))]
pub fn my_error_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Could not parse type to derive Error for");

    impl_hello_macro(ast)
}

#[derive(Debug)]
struct EnumInfo {
    name: syn::Ident,
    variants: Vec<VariantInfo>,
}

#[derive(Debug)]
struct VariantInfo {
    name: syn::Ident,
    source_field: Option<Field>,
    user_fields: Vec<Field>,
    display_format: Option<syn::ExprTuple>,
}

#[derive(Debug)]
struct Field {
    name: syn::Ident,
    ty: syn::Type,
}

fn impl_hello_macro(ty: syn::DeriveInput) -> TokenStream {
    let info = parse_my_error_information(ty);
    generate_my_error(info).into()
}

fn parse_my_error_information(ty: syn::DeriveInput) -> EnumInfo {
    use syn::{Data, Fields};

    let enum_ = match ty.data {
        Data::Enum(enum_) => enum_,
        _ => panic!("Can only derive Error for an enum"),
    };

    let name = ty.ident;

    let variants = enum_.variants.into_iter().map(|variant| {
        let name = variant.ident;

        let display_format = variant.attrs.into_iter().map(|attr| {
            use syn::Expr;

            // my_error::display
            let expr: Expr = syn::parse2(attr.tts).expect("Need expression");
            match expr {
                Expr::Tuple(expr_tuple) => expr_tuple,
                _ => panic!("Require a tuple of format string and values"),
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

fn generate_my_error(enum_info: EnumInfo) -> proc_macro2::TokenStream {
    use syn::Ident;
    use proc_macro2::Span;

    let enum_name = enum_info.name;

    let generated_variant_support = enum_info.variants.iter().map(|variant| {
        let VariantInfo { name: variant_name, source_field, user_fields, display_format } = variant;

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

        let display_struct_name = format!("{}Display", variant_name);
        let display_struct_name = Ident::new(&display_struct_name, Span::call_site());
        let has_fields = !user_fields.is_empty() || !source_field.is_none();
        let lifetime = if has_fields { quote!{ <'a> } } else { quote!{} };

        let variant_display_struct = {
            let fields = user_fields.iter().chain(source_field).map(|f| {
                let Field { name, ty } = f;
                quote!{ #name: &'a #ty }
            });

            quote! {
                #[allow(dead_code)]
                struct #display_struct_name #lifetime {
                    #(#fields),*
                }
            }
        };

        let variant_display_impl = {
            let format = match display_format {
                Some(fmt) => {
                    let inner = &fmt.elems;
                    quote! { #inner }
                }
                None => quote! { stringify!(#variant_name) },
            };

            quote! {
                impl#lifetime core::fmt::Display for #display_struct_name#lifetime {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        write!(f, #format)
                    }
                }
            }
        };

        quote! {
            #variant_selector_struct
            #enum_from_variant_selector_impl
            #variant_display_struct
            #variant_display_impl
        }
    });

    let variants = enum_info.variants.iter().map(|variant| {
        let VariantInfo { name: variant_name, user_fields, source_field, .. } = variant;

        let display_struct_name = format!("{}Display", variant_name);
        let display_struct_name = Ident::new(&display_struct_name, Span::call_site());

        let field_names = user_fields.iter().chain(source_field).map(|f| &f.name);
        let field_names = quote! { #(#field_names),* };

        quote! {
            #enum_name::#variant_name { #field_names } => {
                #display_struct_name { #field_names }.fmt(f)
            }
        }
    });

    let display_impl = quote! {
        impl core::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
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
