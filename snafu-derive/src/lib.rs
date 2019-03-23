extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use std::iter;
use syn::parse::{Error as SynError, Result as SynResult};

/// See the crate-level documentation for SNAFU which contains tested
/// examples of this macro.

#[proc_macro_derive(Snafu, attributes(snafu_visibility, snafu_display))]
pub fn snafu_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Could not parse type to derive Error for");

    impl_hello_macro(ast)
}

struct EnumInfo {
    name: syn::Ident,
    generics: syn::Generics,
    variants: Vec<VariantInfo>,
    default_visibility: Box<quote::ToTokens>,
}

struct VariantInfo {
    name: syn::Ident,
    source_field: Option<Field>,
    backtrace_field: Option<Field>,
    user_fields: Vec<Field>,
    display_format: Option<DisplayFormat>,
    visibility: Option<Box<quote::ToTokens>>,
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
    match parse_snafu_information(ty) {
        Ok(info) => info.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn parse_snafu_information(ty: syn::DeriveInput) -> SynResult<EnumInfo> {
    use syn::spanned::Spanned;
    use syn::{Data, Fields};

    let span = ty.span();

    let default_visibility = parse_snafu_visibility(&ty.attrs)?;
    let default_visibility = default_visibility.unwrap_or_else(|| private_visibility());

    let enum_ = match ty.data {
        Data::Enum(enum_) => enum_,
        _ => {
            return Err(SynError::new(span, "Can only derive `Snafu` for an enum"));
        }
    };

    let name = ty.ident;

    let generics = ty.generics;

    let variants: Result<_, SynError> = enum_
        .variants
        .into_iter()
        .map(|variant| {
            let name = variant.ident;

            let display_format = parse_snafu_display(&variant.attrs)?;
            let visibility = parse_snafu_visibility(&variant.attrs)?;

            let fields = match variant.fields {
                Fields::Named(f) => f.named.into_iter().collect(),
                Fields::Unnamed(_) => {
                    return Err(SynError::new(
                        variant.fields.span(),
                        "Only struct-like and unit enum variants are supported",
                    ));
                }
                Fields::Unit => vec![],
            };

            let mut user_fields = Vec::new();
            let mut source_fields = Vec::new();
            let mut backtrace_fields = Vec::new();

            for field in fields {
                let span = field.span();
                let name = field
                    .ident
                    .ok_or_else(|| SynError::new(span, "Must have a named field"))?;
                let field = Field { name, ty: field.ty };

                if field.name == "source" {
                    source_fields.push(field);
                } else if field.name == "backtrace" {
                    backtrace_fields.push(field);
                } else {
                    user_fields.push(field);
                }
            }

            let source_field = source_fields.pop();
            // Report a warning if there are multiple?

            let backtrace_field = backtrace_fields.pop();
            // Report a warning if there are multiple?

            Ok(VariantInfo {
                name,
                source_field,
                backtrace_field,
                user_fields,
                display_format,
                visibility,
            })
        })
        .collect();
    let variants = variants?;

    Ok(EnumInfo {
        name,
        generics,
        variants,
        default_visibility,
    })
}

fn parse_snafu_visibility(attrs: &[syn::Attribute]) -> SynResult<Option<Box<quote::ToTokens>>> {
    use syn::spanned::Spanned;
    use syn::Meta;

    attrs
        .into_iter()
        .flat_map(|attr| {
            if attr.path.is_ident("snafu_visibility") {
                let meta = match attr.parse_meta() {
                    Ok(meta) => meta,
                    Err(e) => return Some(Err(e)),
                };
                match meta {
                    Meta::Word(_) => Some(Ok(private_visibility())),
                    Meta::NameValue(nv) => Some(parse_snafu_visibility_nested_name_value(nv)),
                    meta => Some(Err(SynError::new(
                        meta.span(),
                        "`snafu_visibility` either has no argument or uses an equal sign",
                    ))),
                }
            } else {
                None
            }
        })
        .next()
        .my_transpose()
}

fn private_visibility() -> Box<quote::ToTokens> {
    Box::new(quote! {})
}

fn parse_snafu_visibility_nested_name_value(
    nv: syn::MetaNameValue,
) -> SynResult<Box<quote::ToTokens>> {
    use syn::Visibility;

    let s = unpack_nv_string_literal(nv)?;
    let v = s.parse::<Visibility>()?;

    Ok(Box::new(v))
}

fn parse_snafu_display(attrs: &[syn::Attribute]) -> SynResult<Option<DisplayFormat>> {
    use syn::spanned::Spanned;
    use syn::Meta;

    attrs
        .into_iter()
        .flat_map(|attr| {
            if attr.path.is_ident("snafu_display") {
                let meta = match attr.parse_meta() {
                    Ok(meta) => meta,
                    Err(e) => return Some(Err(e)),
                };
                match meta {
                    Meta::List(list) => Some(parse_snafu_display_nested(list)),
                    Meta::NameValue(nv) => Some(parse_snafu_display_nested_name_value(nv)),
                    meta => Some(Err(SynError::new(
                        meta.span(),
                        "`snafu_display` requires an argument",
                    ))),
                }
            } else {
                // These are ignored, hopefully they belong to
                // someone else...
                None
            }
        })
        .next()
        .my_transpose()
}

fn parse_snafu_display_nested(meta: syn::MetaList) -> SynResult<DisplayFormat> {
    use syn::spanned::Spanned;
    use syn::{Expr, Lit, NestedMeta};

    let mut nested = meta.nested.into_iter().map(|nested| {
        let span = nested.span();
        let non_literal = || Err(SynError::new(span, "A list of string literals is expected"));

        let nested = match nested {
            NestedMeta::Literal(lit) => lit,
            _ => return non_literal(),
        };
        match nested {
            Lit::Str(s) => Ok(s),
            _ => return non_literal(),
        }
    });

    let fmt_str = nested
        .next()
        .map(|x| x.map(|x| Box::new(x) as Box<quote::ToTokens>));

    let fmt_args = nested
        .map(|x| x.and_then(|x| x.parse::<Expr>()))
        .map(|x| x.map(|x| Box::new(x) as Box<quote::ToTokens>));

    let nested: Result<_, _> = fmt_str.into_iter().chain(fmt_args).collect();
    let nested = nested?;

    Ok(DisplayFormat::Stringified(nested))
}

fn parse_snafu_display_nested_name_value(nv: syn::MetaNameValue) -> SynResult<DisplayFormat> {
    use syn::Expr;

    let s = unpack_nv_string_literal(nv)?;

    let expr: Box<quote::ToTokens> = match s.parse::<Expr>()? {
        Expr::Tuple(expr_tuple) => Box::new(expr_tuple.elems),
        Expr::Paren(expr_paren) => Box::new(expr_paren.expr),
        _ => {
            return Err(SynError::new(
                s.span(),
                "A parenthesized format string with optional values is expected",
            ));
        }
    };

    Ok(DisplayFormat::Direct(expr))
}

fn unpack_nv_string_literal(nv: syn::MetaNameValue) -> SynResult<syn::LitStr> {
    use syn::spanned::Spanned;
    use syn::Lit;

    match nv.lit {
        Lit::Str(s) => Ok(s),
        _ => Err(SynError::new(nv.lit.span(), "A string literal is expected")),
    }
}

impl From<EnumInfo> for proc_macro::TokenStream {
    fn from(other: EnumInfo) -> proc_macro::TokenStream {
        other.generate_snafu().into()
    }
}

impl EnumInfo {
    fn generate_snafu(self) -> proc_macro2::TokenStream {
        let context_selectors = ContextSelectors(&self);
        let display_impl = DisplayImpl(&self);
        let error_impl = ErrorImpl(&self);
        let error_compat_impl = ErrorCompatImpl(&self);

        quote! {
            #context_selectors
            #display_impl
            #error_impl
            #error_compat_impl
        }
    }

    fn parameterized_enum_name(&self) -> Box<quote::ToTokens> {
        let enum_name = &self.name;
        let original_generics = self.generics.params.iter();

        Box::new(quote! { #enum_name<#(#original_generics,)*> })
    }
}

struct ContextSelectors<'a>(&'a EnumInfo);

impl<'a> quote::ToTokens for ContextSelectors<'a> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let context_selectors = self
            .0
            .variants
            .iter()
            .map(|variant| ContextSelector(self.0, variant));

        stream.extend({
            quote! {
                #(#context_selectors)*
            }
        })
    }
}

struct ContextSelector<'a>(&'a EnumInfo, &'a VariantInfo);

impl<'a> quote::ToTokens for ContextSelector<'a> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        use proc_macro2::Span;
        use syn::Ident;

        let enum_name = &self.0.name;
        let original_generics: &Vec<_> = &self.0.generics.params.iter().collect();

        let parameterized_enum_name = &self.0.parameterized_enum_name();

        let VariantInfo {
            name: ref variant_name,
            ref source_field,
            ref backtrace_field,
            ref user_fields,
            ..
        } = *self.1;

        let generic_names: &Vec<_> = &(0..user_fields.len())
            .map(|i| Ident::new(&format!("__T{}", i), Span::call_site()))
            .collect();

        let visibility = self
            .1
            .visibility
            .as_ref()
            .unwrap_or(&self.0.default_visibility);

        let generics_list = quote! { <#(#original_generics,)* #(#generic_names,)*> };
        let selector_name = quote! { #variant_name<#(#generic_names,)*> };

        let names: &Vec<_> = &user_fields.iter().map(|f| f.name.clone()).collect();
        let types = generic_names;

        let variant_selector_struct = {
            if user_fields.is_empty() {
                quote! {
                    #visibility struct #selector_name;
                }
            } else {
                let visibilities = iter::repeat(visibility);

                quote! {
                    #visibility struct #selector_name {
                        #( #visibilities #names: #types ),*
                    }
                }
            }
        };

        let backtrace_field = match *backtrace_field {
            Some(_) => {
                quote! { backtrace: std::default::Default::default(), }
            }
            None => quote! {},
        };

        let where_clauses: &Vec<_> = &generic_names
            .iter()
            .zip(user_fields)
            .map(|(gen_ty, f)| {
                let Field { ref ty, .. } = *f;
                quote! { #gen_ty: std::convert::Into<#ty> }
            })
            .collect();

        let inherent_impl = if source_field.is_none() {
            let names2 = names;
            quote! {
                impl<#(#generic_names,)*> #selector_name
                {
                    #visibility fn fail<#(#original_generics,)* __T>(self) -> std::result::Result<__T, #parameterized_enum_name>
                    where
                        #(#where_clauses),*
                    {
                        let Self { #(#names),* } = self;
                        let error = #enum_name::#variant_name {
                            #backtrace_field
                            #( #names: std::convert::Into::into(#names2) ),*
                        };
                        std::result::Result::Err(error)
                    }
                }
            }
        } else {
            quote! {}
        };

        let enum_from_variant_selector_impl = {
            let user_fields = user_fields.iter().map(|f| {
                let Field { ref name, .. } = *f;
                quote! { #name: other.context.#name.into() }
            });

            let other_ty;
            let source_xfer_field;

            match *source_field {
                Some(ref source_field) => {
                    let Field {
                        name: ref source_name,
                        ty: ref source_ty,
                    } = *source_field;

                    other_ty = quote! { snafu::Context<#source_ty, #selector_name> };
                    source_xfer_field = quote! { #source_name: other.error, };
                }
                None => {
                    other_ty = quote! { snafu::Context<snafu::NoneError, #selector_name> };
                    source_xfer_field = quote! {};
                }
            }

            quote! {
                impl#generics_list std::convert::From<#other_ty> for #parameterized_enum_name
                where
                    #(#where_clauses),*
                {
                    fn from(other: #other_ty) -> Self {
                        #enum_name::#variant_name {
                            #source_xfer_field
                            #backtrace_field
                            #(#user_fields),*
                        }
                    }
                }
            }
        };

        stream.extend({
            quote! {
                #variant_selector_struct
                #inherent_impl
                #enum_from_variant_selector_impl
            }
        })
    }
}

struct DisplayImpl<'a>(&'a EnumInfo);

impl<'a> DisplayImpl<'a> {
    fn variants_to_display(&self) -> Vec<proc_macro2::TokenStream> {
        let enum_name = &self.0.name;

        self.0
            .variants
            .iter()
            .map(|variant| {
                let VariantInfo {
                    name: ref variant_name,
                    ref user_fields,
                    ref source_field,
                    ref backtrace_field,
                    ref display_format,
                    ..
                } = *variant;

                let format = match *display_format {
                    Some(DisplayFormat::Stringified(ref fmt)) => {
                        quote! { #(#fmt),* }
                    }
                    Some(DisplayFormat::Direct(ref fmt)) => {
                        quote! { #fmt }
                    }
                    None => quote! { stringify!(#variant_name) },
                };

                let field_names = user_fields
                    .iter()
                    .chain(source_field)
                    .chain(backtrace_field)
                    .map(|f| &f.name);
                let field_names = quote! { #(ref #field_names),* };

                quote! {
                    #enum_name::#variant_name { #field_names } => {
                        write!(f, #format)
                    }
                }
            })
            .collect()
    }
}

impl<'a> quote::ToTokens for DisplayImpl<'a> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let original_generics = &self.0.generics;
        let parameterized_enum_name = &self.0.parameterized_enum_name();

        let variants_to_display = &self.variants_to_display();

        stream.extend({
            quote! {
                impl#original_generics std::fmt::Display for #parameterized_enum_name {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        #[allow(unused_variables)]
                        match *self {
                            #(#variants_to_display)*
                        }
                    }
                }
            }
        })
    }
}

struct ErrorImpl<'a>(&'a EnumInfo);

impl<'a> ErrorImpl<'a> {
    fn variants_to_description(&self) -> Vec<proc_macro2::TokenStream> {
        let enum_name = &self.0.name;
        self.0
            .variants
            .iter()
            .map(|variant| {
                let VariantInfo {
                    name: ref variant_name,
                    ..
                } = *variant;
                quote! {
                    #enum_name::#variant_name { .. } => stringify!(#enum_name::#variant_name),
                }
            })
            .collect()
    }

    fn variants_to_source(&self) -> Vec<proc_macro2::TokenStream> {
        let enum_name = &self.0.name;
        self.0
            .variants
            .iter()
            .map(|variant| {
                let VariantInfo {
                    name: ref variant_name,
                    ref source_field,
                    ..
                } = *variant;

                match *source_field {
                    Some(ref source_field) => {
                        let Field {
                            name: ref field_name,
                            ..
                        } = *source_field;
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
            })
            .collect()
    }
}

impl<'a> quote::ToTokens for ErrorImpl<'a> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let original_generics = &self.0.generics;
        let parameterized_enum_name = &self.0.parameterized_enum_name();

        let variants_to_description = &self.variants_to_description();

        let description_fn = quote! {
            fn description(&self) -> &str {
                match *self {
                    #(#variants_to_description)*
                }
            }
        };

        let variants_to_source = &self.variants_to_source();

        let cause_fn = quote! {
            fn cause(&self) -> Option<&std::error::Error> {
                match *self {
                    #(#variants_to_source)*
                }
            }
        };

        let source_fn = if cfg!(feature = "rust_1_30") {
            quote! {
                fn source(&self) -> Option<&(std::error::Error + 'static)> {
                    match *self {
                        #(#variants_to_source)*
                    }
                }
            }
        } else {
            quote! {}
        };

        stream.extend({
            quote! {
                impl#original_generics std::error::Error for #parameterized_enum_name
                where
                    Self: std::fmt::Debug + std::fmt::Display,
                {
                    #description_fn
                    #cause_fn
                    #source_fn
                }
            }
        })
    }
}

struct ErrorCompatImpl<'a>(&'a EnumInfo);

impl<'a> ErrorCompatImpl<'a> {
    fn variants_to_backtrace(&self) -> Vec<proc_macro2::TokenStream> {
        let enum_name = &self.0.name;
        self.0.variants.iter().map(|variant| {
            let VariantInfo {
                name: ref variant_name,
                ref backtrace_field,
                ..
            } = *variant;

            match *backtrace_field {
                Some(ref backtrace_field) => {
                    let Field {
                        name: ref field_name,
                        ..
                    } = *backtrace_field;
                    quote! {
                        #enum_name::#variant_name { ref #field_name, .. } => { Some(#field_name) }
                    }
                }
                None => {
                    quote! {
                        #enum_name::#variant_name { .. } => { None }
                    }
                }
            }
        }).collect()
    }
}

impl<'a> quote::ToTokens for ErrorCompatImpl<'a> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let original_generics = &self.0.generics;
        let parameterized_enum_name = &self.0.parameterized_enum_name();

        let variants = &self.variants_to_backtrace();

        let backtrace_fn = if cfg!(feature = "backtraces") {
            quote! {
                fn backtrace(&self) -> Option<&snafu::Backtrace> {
                    match *self {
                        #(#variants),*
                    }
                }
            }
        } else {
            quote! {}
        };

        stream.extend({
            quote! {
                impl#original_generics snafu::ErrorCompat for #parameterized_enum_name {
                    #backtrace_fn
                }
            }
        })
    }
}

trait Transpose<T, E> {
    fn my_transpose(self) -> Result<Option<T>, E>;
}

impl<T, E> Transpose<T, E> for Option<Result<T, E>> {
    fn my_transpose(self) -> Result<Option<T>, E> {
        match self {
            Some(Ok(v)) => Ok(Some(v)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}
