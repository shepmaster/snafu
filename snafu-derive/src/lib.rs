extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use std::iter;
use syn::parse::{Error as SynError, Result as SynResult};

/// See the crate-level documentation for SNAFU which contains tested
/// examples of this macro.

#[proc_macro_derive(Snafu, attributes(snafu))]
pub fn snafu_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Could not parse type to derive Error for");

    impl_snafu_macro(ast)
}

type UserInput = Box<quote::ToTokens>;

enum SnafuInfo {
    Enum(EnumInfo),
    Struct(StructInfo),
}

struct EnumInfo {
    name: syn::Ident,
    generics: syn::Generics,
    variants: Vec<VariantInfo>,
    default_visibility: UserInput,
}

struct VariantInfo {
    name: syn::Ident,
    source_field: Option<Field>,
    backtrace_field: Option<Field>,
    backtrace_delegate: Option<Field>,
    user_fields: Vec<Field>,
    display_format: Option<UserInput>,
    visibility: Option<UserInput>,
}

struct StructInfo {
    name: syn::Ident,
    generics: syn::Generics,
    inner_type: syn::Type,
}

#[derive(Clone)]
struct Field {
    name: syn::Ident,
    ty: syn::Type,
}

fn impl_snafu_macro(ty: syn::DeriveInput) -> TokenStream {
    match parse_snafu_information(ty) {
        Ok(info) => info.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn parse_snafu_information(ty: syn::DeriveInput) -> SynResult<SnafuInfo> {
    use syn::spanned::Spanned;
    use syn::Data;

    let span = ty.span();
    let syn::DeriveInput {
        ident,
        generics,
        data,
        attrs,
        ..
    } = ty;

    match data {
        Data::Enum(enum_) => parse_snafu_enum(enum_, ident, generics, attrs).map(SnafuInfo::Enum),
        Data::Struct(struct_) => {
            parse_snafu_struct(struct_, ident, generics, span).map(SnafuInfo::Struct)
        }
        _ => {
            return Err(SynError::new(
                span,
                "Can only derive `Snafu` for an enum or a newtype",
            ));
        }
    }
}

fn parse_snafu_enum(
    enum_: syn::DataEnum,
    name: syn::Ident,
    generics: syn::Generics,
    attrs: Vec<syn::Attribute>,
) -> SynResult<EnumInfo> {
    use syn::spanned::Spanned;
    use syn::Fields;

    let default_visibility = attributes_from_syn(attrs)?
        .into_iter()
        .flat_map(SnafuAttribute::into_visibility)
        .next()
        .unwrap_or_else(private_visibility);

    let variants: Result<_, SynError> = enum_
        .variants
        .into_iter()
        .map(|variant| {
            let name = variant.ident;

            let mut display_format = None;
            let mut visibility = None;

            for attr in attributes_from_syn(variant.attrs)? {
                match attr {
                    SnafuAttribute::Display(d) => display_format = Some(d),
                    SnafuAttribute::Visibility(v) => visibility = Some(v),
                    SnafuAttribute::Backtrace => { /* Report this isn't valid here? */ }
                }
            }

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
            let mut backtrace_delegates = Vec::new();

            for syn_field in fields {
                let span = syn_field.span();
                let name = syn_field
                    .ident
                    .ok_or_else(|| SynError::new(span, "Must have a named field"))?;
                let field = Field {
                    name,
                    ty: syn_field.ty,
                };

                let has_backtrace = attributes_from_syn(syn_field.attrs)?
                    .iter()
                    .any(SnafuAttribute::is_backtrace);

                if field.name == "source" {
                    if has_backtrace {
                        backtrace_delegates.push(field.clone());
                    }

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

            let backtrace_delegate = backtrace_delegates.pop();
            // Report a warning if there are multiple?
            // Report a warning if delegating and our own?

            Ok(VariantInfo {
                name,
                source_field,
                backtrace_field,
                backtrace_delegate,
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

fn parse_snafu_struct(
    struct_: syn::DataStruct,
    name: syn::Ident,
    generics: syn::Generics,
    span: proc_macro2::Span,
) -> SynResult<StructInfo> {
    use syn::Fields;

    let mut fields = match struct_.fields {
        Fields::Unnamed(f) => f,
        _ => {
            return Err(SynError::new(
                span,
                "Can only derive `Snafu` for tuple structs",
            ));
        }
    };

    fn one_field_error(span: proc_macro2::Span) -> SynError {
        SynError::new(
            span,
            "Can only derive `Snafu` for tuple structs with exactly one field",
        )
    }

    let inner = fields.unnamed.pop().ok_or_else(|| one_field_error(span))?;
    if !fields.unnamed.is_empty() {
        return Err(one_field_error(span));
    }

    let inner_type = inner.into_value().ty;

    Ok(StructInfo {
        name,
        inner_type,
        generics,
    })
}

enum MyMeta<T> {
    CompatParen(T),
    CompatDirect(T),
    Pretty(T),
    None,
}

impl<T> MyMeta<T> {
    fn into_option(self) -> Option<T> {
        match self {
            MyMeta::CompatParen(v) => Some(v),
            MyMeta::CompatDirect(v) => Some(v),
            MyMeta::Pretty(v) => Some(v),
            MyMeta::None => None,
        }
    }
}

impl<T> syn::parse::Parse for MyMeta<T>
where
    T: syn::parse::Parse,
{
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::token::{Eq, Paren};
        use syn::LitStr;

        let lookahead = input.lookahead1();

        if lookahead.peek(Paren) {
            let inside;
            parenthesized!(inside in input);
            let t: T = inside.parse()?;

            Ok(MyMeta::Pretty(t))
        } else if lookahead.peek(Eq) {
            let _: Eq = input.parse()?;
            let s: LitStr = input.parse()?;

            match s.parse::<MyParens<T>>() {
                Ok(t) => Ok(MyMeta::CompatParen(t.0)),
                Err(_) => match s.parse::<T>() {
                    Ok(t) => Ok(MyMeta::CompatDirect(t)),
                    Err(e) => Err(e),
                },
            }
        } else if input.is_empty() {
            Ok(MyMeta::None)
        } else {
            Err(lookahead.error())
        }
    }
}

struct MyParens<T>(T);

impl<T> syn::parse::Parse for MyParens<T>
where
    T: syn::parse::Parse,
{
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        let inside;
        parenthesized!(inside in input);
        inside.parse().map(MyParens)
    }
}

struct MyExprList(syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>);

impl syn::parse::Parse for MyExprList {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::punctuated::Punctuated;
        let exprs = Punctuated::parse_terminated(input)?;
        Ok(MyExprList(exprs))
    }
}

enum SnafuAttribute {
    Display(UserInput),
    Visibility(UserInput),
    Backtrace,
}

impl SnafuAttribute {
    fn into_visibility(self) -> Option<UserInput> {
        match self {
            SnafuAttribute::Visibility(v) => Some(v),
            _ => None,
        }
    }

    fn is_backtrace(&self) -> bool {
        match *self {
            SnafuAttribute::Backtrace => true,
            _ => false,
        }
    }
}

impl syn::parse::Parse for SnafuAttribute {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::{Ident, Visibility};

        let inside;
        parenthesized!(inside in input);
        let name: Ident = inside.parse()?;

        if name == "display" {
            let m: MyMeta<MyExprList> = inside.parse()?;
            let v = m.into_option().ok_or_else(|| {
                SynError::new(name.span(), "`snafu(display)` requires an argument")
            })?;
            let v = Box::new(v.0);
            Ok(SnafuAttribute::Display(v))
        } else if name == "visibility" {
            let m: MyMeta<Visibility> = inside.parse()?;
            let v = m
                .into_option()
                .map_or_else(private_visibility, |v| Box::new(v) as UserInput);
            Ok(SnafuAttribute::Visibility(v))
        } else if name == "backtrace" {
            let _: MyParens<Ident> = inside.parse()?;
            Ok(SnafuAttribute::Backtrace)
        } else {
            Err(SynError::new(
                name.span(),
                "expected `display`, `visibility`, or `backtrace`",
            ))
        }
    }
}

struct SnafuAttributeBody(Vec<SnafuAttribute>);

impl syn::parse::Parse for SnafuAttributeBody {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::punctuated::Punctuated;
        use syn::token::Comma;

        let parse_comma_list = Punctuated::<SnafuAttribute, Comma>::parse_terminated;
        let list = parse_comma_list(input)?;

        Ok(SnafuAttributeBody(
            list.into_pairs().map(|p| p.into_value()).collect(),
        ))
    }
}

fn attributes_from_syn(attrs: Vec<syn::Attribute>) -> SynResult<Vec<SnafuAttribute>> {
    use syn::parse2;

    let mut ours = Vec::new();

    let parsed_attrs = attrs
        .into_iter()
        .filter(|attr| attr.path.is_ident("snafu"))
        .map(|attr| {
            let body: SnafuAttributeBody = parse2(attr.tts)?;
            Ok(body.0)
        });

    for attr in parsed_attrs {
        match attr {
            Ok(v) => ours.extend(v),
            Err(e) => return Err(e),
        }
    }

    Ok(ours)
}

fn private_visibility() -> UserInput {
    Box::new(quote! {})
}

impl From<SnafuInfo> for proc_macro::TokenStream {
    fn from(other: SnafuInfo) -> proc_macro::TokenStream {
        match other {
            SnafuInfo::Enum(e) => e.into(),
            SnafuInfo::Struct(s) => s.into(),
        }
    }
}

impl From<EnumInfo> for proc_macro::TokenStream {
    fn from(other: EnumInfo) -> proc_macro::TokenStream {
        other.generate_snafu().into()
    }
}

impl From<StructInfo> for proc_macro::TokenStream {
    fn from(other: StructInfo) -> proc_macro::TokenStream {
        other.generate_snafu().into()
    }
}

trait GenericAwareNames {
    fn name(&self) -> &syn::Ident;

    fn generics(&self) -> &syn::Generics;

    fn parameterized_name(&self) -> UserInput {
        let enum_name = self.name();
        let original_generics = self.provided_generic_names();

        Box::new(quote! { #enum_name<#(#original_generics,)*> })
    }

    fn provided_generic_names(&self) -> Vec<proc_macro2::TokenStream> {
        use syn::{ConstParam, GenericParam, LifetimeDef, TypeParam};

        self.generics()
            .params
            .iter()
            .map(|p| match *p {
                GenericParam::Type(TypeParam { ref ident, .. }) => quote! { #ident },
                GenericParam::Lifetime(LifetimeDef { ref lifetime, .. }) => quote! { #lifetime },
                GenericParam::Const(ConstParam { ref ident, .. }) => quote! { #ident },
            })
            .collect()
    }

    fn provided_where_clauses(&self) -> Vec<proc_macro2::TokenStream> {
        self.generics()
            .where_clause
            .iter()
            .flat_map(|c| c.predicates.iter().map(|p| quote! { #p }))
            .collect()
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
}

impl GenericAwareNames for EnumInfo {
    fn name(&self) -> &syn::Ident {
        &self.name
    }

    fn generics(&self) -> &syn::Generics {
        &self.generics
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

        let parameterized_enum_name = &self.0.parameterized_name();

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
            .chain(self.0.provided_where_clauses())
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

                let format = match (display_format, source_field) {
                    (&Some(ref v), _) => quote! { #v },
                    (&None, &Some(ref f)) => {
                        let field_name = &f.name;
                        quote! { concat!(stringify!(#variant_name), ": {}"), #field_name }
                    }
                    (&None, &None) => quote! { stringify!(#variant_name)},
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
        let parameterized_enum_name = &self.0.parameterized_name();
        let where_clauses = &self.0.provided_where_clauses();

        let variants_to_display = &self.variants_to_display();

        stream.extend({
            quote! {
                impl#original_generics std::fmt::Display for #parameterized_enum_name
                where
                    #(#where_clauses),*
                {
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
                                std::option::Option::Some(std::borrow::Borrow::borrow(#field_name))
                            }
                        }
                    }
                    None => {
                        quote! {
                            #enum_name::#variant_name { .. } => { std::option::Option::None }
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
        let parameterized_enum_name = &self.0.parameterized_name();
        let where_clauses: &Vec<_> = &self.0.provided_where_clauses();

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
                    #(#where_clauses),*
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
                ref backtrace_delegate,
                ..
            } = *variant;


            if let Some(ref backtrace_delegate) = *backtrace_delegate {
                let Field {
                    name: ref field_name,
                    ..
                } = *backtrace_delegate;
                quote! {
                    #enum_name::#variant_name { ref #field_name, .. } => { snafu::ErrorCompat::backtrace(#field_name) }
                }

            } else if let Some(ref backtrace_field) = *backtrace_field  {
                let Field {
                    name: ref field_name,
                    ..
                } = *backtrace_field;
                quote! {
                    #enum_name::#variant_name { ref #field_name, .. } => { std::option::Option::Some(#field_name) }
                }
            } else {
                quote! {
                    #enum_name::#variant_name { .. } => { std::option::Option::None }
                }
            }
        }).collect()
    }
}

impl<'a> quote::ToTokens for ErrorCompatImpl<'a> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let original_generics = &self.0.generics;
        let parameterized_enum_name = &self.0.parameterized_name();
        let where_clauses = &self.0.provided_where_clauses();

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
                impl#original_generics snafu::ErrorCompat for #parameterized_enum_name
                where
                    #(#where_clauses),*
                {
                    #backtrace_fn
                }
            }
        })
    }
}

impl StructInfo {
    fn generate_snafu(self) -> proc_macro2::TokenStream {
        let parameterized_struct_name = self.parameterized_name();

        let StructInfo {
            inner_type,
            generics,
            name,
        } = self;

        let where_clauses: &Vec<_> = &generics
            .where_clause
            .iter()
            .flat_map(|c| c.predicates.iter().map(|p| quote! { #p }))
            .collect();

        let description_fn = quote! {
            fn description(&self) -> &str {
                std::error::Error::description(&self.0)
            }
        };

        let cause_fn = quote! {
            fn cause(&self) -> Option<&std::error::Error> {
                std::error::Error::cause(&self.0)
            }
        };

        let source_fn = if cfg!(feature = "rust_1_30") {
            quote! {
                fn source(&self) -> Option<&(std::error::Error + 'static)> {
                    std::error::Error::source(&self.0)
                }
            }
        } else {
            quote! {}
        };

        let backtrace_fn = if cfg!(feature = "backtraces") {
            quote! {
                fn backtrace(&self) -> Option<&snafu::Backtrace> {
                    snafu::ErrorCompat::backtrace(&self.0)
                }
            }
        } else {
            quote! {}
        };

        let error_impl = quote! {
            impl#generics std::error::Error for #parameterized_struct_name
            where
                #(#where_clauses),*
            {
                #description_fn
                #cause_fn
                #source_fn
            }
        };

        let error_compat_impl = quote! {
            impl#generics snafu::ErrorCompat for #parameterized_struct_name
            where
                #(#where_clauses),*
            {
                #backtrace_fn
            }
        };

        let display_impl = quote! {
            impl#generics std::fmt::Display for #parameterized_struct_name
            where
                #(#where_clauses),*
            {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    std::fmt::Display::fmt(&self.0, f)
                }
            }
        };

        let from_impl = quote! {
            impl#generics From<#inner_type> for #parameterized_struct_name
            where
                #(#where_clauses),*
            {
                fn from(other: #inner_type) -> Self {
                    #name(other)
                }
            }
        };

        quote! {
            #error_impl
            #error_compat_impl
            #display_impl
            #from_impl
        }
    }
}

impl GenericAwareNames for StructInfo {
    fn name(&self) -> &syn::Ident {
        &self.name
    }

    fn generics(&self) -> &syn::Generics {
        &self.generics
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
