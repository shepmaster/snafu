#![allow(unknown_lints, bare_trait_objects)]
#![recursion_limit = "128"] // https://github.com/rust-lang/rust/issues/62059

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use std::iter;
use syn::parse::Result as SynResult;

/// See the crate-level documentation for SNAFU which contains tested
/// examples of this macro.

#[proc_macro_derive(Snafu, attributes(snafu))]
pub fn snafu_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Could not parse type to derive Error for");

    impl_snafu_macro(ast)
}

type MultiSynResult<T> = std::result::Result<T, Vec<syn::Error>>;

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
    source_field: Option<SourceField>,
    backtrace_field: Option<Field>,
    backtrace_delegate: Option<Field>,
    user_fields: Vec<Field>,
    display_format: Option<UserInput>,
    doc_comment: String,
    visibility: Option<UserInput>,
}

struct StructInfo {
    name: syn::Ident,
    generics: syn::Generics,
    transformation: Transformation,
}

#[derive(Clone)]
struct Field {
    name: syn::Ident,
    ty: syn::Type,
}

impl Field {
    fn name(&self) -> &syn::Ident {
        &self.name
    }
}

struct SourceField {
    name: syn::Ident,
    transformation: Transformation,
}

impl SourceField {
    fn name(&self) -> &syn::Ident {
        &self.name
    }
}

enum Transformation {
    None { ty: syn::Type },
    Transform { ty: syn::Type, expr: syn::Expr },
}

impl Transformation {
    fn ty(&self) -> &syn::Type {
        match *self {
            Transformation::None { ref ty } => ty,
            Transformation::Transform { ref ty, .. } => ty,
        }
    }

    fn transformation(&self) -> proc_macro2::TokenStream {
        match *self {
            Transformation::None { .. } => quote! { |v| v },
            Transformation::Transform { ref expr, .. } => quote! { #expr },
        }
    }
}

fn impl_snafu_macro(ty: syn::DeriveInput) -> TokenStream {
    match parse_snafu_information(ty) {
        Ok(info) => info.into(),
        Err(e) => to_compile_errors(e).into(),
    }
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote! { #(#compile_errors)* }
}

fn parse_snafu_information(ty: syn::DeriveInput) -> MultiSynResult<SnafuInfo> {
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
            parse_snafu_struct(struct_, ident, generics, attrs, span).map(SnafuInfo::Struct)
        }
        _ => Err(vec![syn::Error::new(
            span,
            "Can only derive `Snafu` for an enum or a newtype",
        )]),
    }
}

fn parse_snafu_enum(
    enum_: syn::DataEnum,
    name: syn::Ident,
    generics: syn::Generics,
    attrs: Vec<syn::Attribute>,
) -> MultiSynResult<EnumInfo> {
    use syn::spanned::Spanned;
    use syn::Fields;

    let default_visibility = attributes_from_syn(attrs)?
        .into_iter()
        .flat_map(SnafuAttribute::into_visibility)
        .next()
        .unwrap_or_else(private_visibility);

    let variants: sponge::AllErrors<_, _> = enum_
        .variants
        .into_iter()
        .map(|variant| {
            let name = variant.ident;

            let mut display_format = None;
            let mut visibility = None;
            let mut doc_comment = String::new();
            let mut reached_end_of_doc_comment = false;

            for attr in attributes_from_syn(variant.attrs)? {
                match attr {
                    SnafuAttribute::Display(d) => display_format = Some(d),
                    SnafuAttribute::Visibility(v) => visibility = Some(v),
                    SnafuAttribute::Source(..) => { /* Report this isn't valid here? */ }
                    SnafuAttribute::Backtrace(..) => { /* Report this isn't valid here? */ }
                    SnafuAttribute::DocComment(doc_comment_line) => {
                        // We join all the doc comment attributes with a space,
                        // but end once the summary of the doc comment is
                        // complete, which is indicated by an empty line.
                        if !reached_end_of_doc_comment {
                            let trimmed = doc_comment_line.trim();
                            if trimmed.is_empty() {
                                reached_end_of_doc_comment = true;
                            } else {
                                if !doc_comment.is_empty() {
                                    doc_comment.push_str(" ");
                                }
                                doc_comment.push_str(trimmed);
                            }
                        }
                    }
                }
            }

            let fields = match variant.fields {
                Fields::Named(f) => f.named.into_iter().collect(),
                Fields::Unnamed(_) => {
                    return Err(vec![syn::Error::new(
                        variant.fields.span(),
                        "Only struct-like and unit enum variants are supported",
                    )]);
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
                    .ok_or_else(|| vec![syn::Error::new(span, "Must have a named field")])?;
                let field = Field {
                    name,
                    ty: syn_field.ty,
                };

                let mut has_backtrace = false;
                let mut is_source = None;
                let mut is_backtrace = None;
                let mut transformation = None;

                for attr in attributes_from_syn(syn_field.attrs)? {
                    match attr {
                        SnafuAttribute::Source(ss) => {
                            for s in ss {
                                match s {
                                    Source::Flag(v) => is_source = Some(v),
                                    Source::From(t, e) => transformation = Some((t, e)),
                                }
                            }
                        }
                        SnafuAttribute::Backtrace(b) => match b {
                            Backtrace::Flag(v) => is_backtrace = Some(v),
                            Backtrace::Delegate => has_backtrace = true,
                        },
                        SnafuAttribute::Visibility(_) => { /* Report this isn't valid here? */ }
                        SnafuAttribute::Display(_) => { /* Report this isn't valid here? */ }
                        SnafuAttribute::DocComment(_) => { /* Just a regular doc comment. */ }
                    }
                }

                let is_source = is_source.unwrap_or(field.name == "source");
                let is_backtrace = is_backtrace.unwrap_or(field.name == "backtrace");

                if is_source {
                    if has_backtrace {
                        backtrace_delegates.push(field.clone());
                    }

                    let Field { name, ty } = field;
                    let transformation = match transformation {
                        Some((ty, expr)) => Transformation::Transform { ty, expr },
                        None => Transformation::None { ty },
                    };

                    source_fields.push(SourceField {
                        name,
                        transformation,
                    });
                } else if is_backtrace {
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
                doc_comment,
                visibility,
            })
        })
        .collect();
    let variants = variants.into_result()?;

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
    attrs: Vec<syn::Attribute>,
    span: proc_macro2::Span,
) -> MultiSynResult<StructInfo> {
    use syn::Fields;

    let mut transformation = None;

    for attr in attributes_from_syn(attrs)? {
        match attr {
            SnafuAttribute::Display(..) => { /* Report this isn't valid here? */ }
            SnafuAttribute::Visibility(..) => { /* Report this isn't valid here? */ }
            SnafuAttribute::Source(ss) => {
                for s in ss {
                    match s {
                        Source::Flag(..) => { /* Report this isn't valid here? */ }
                        Source::From(t, e) => transformation = Some((t, e)),
                    }
                }
            }
            SnafuAttribute::Backtrace(..) => { /* Report this isn't valid here? */ }
            SnafuAttribute::DocComment(_) => { /* Just a regular doc comment. */ }
        }
    }

    let mut fields = match struct_.fields {
        Fields::Unnamed(f) => f,
        _ => {
            return Err(vec![syn::Error::new(
                span,
                "Can only derive `Snafu` for tuple structs",
            )]);
        }
    };

    fn one_field_error(span: proc_macro2::Span) -> syn::Error {
        syn::Error::new(
            span,
            "Can only derive `Snafu` for tuple structs with exactly one field",
        )
    }

    let inner = fields
        .unnamed
        .pop()
        .ok_or_else(|| vec![one_field_error(span)])?;
    if !fields.unnamed.is_empty() {
        return Err(vec![one_field_error(span)]);
    }

    let transformation = match transformation {
        Some((ty, expr)) => Transformation::Transform { ty, expr },
        None => Transformation::None {
            ty: inner.into_value().ty,
        },
    };

    Ok(StructInfo {
        name,
        generics,
        transformation,
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

struct List<T>(syn::punctuated::Punctuated<T, syn::token::Comma>);

impl<T> syn::parse::Parse for List<T>
where
    T: syn::parse::Parse,
{
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::punctuated::Punctuated;
        let exprs = Punctuated::parse_terminated(input)?;
        Ok(List(exprs))
    }
}

impl<T> List<T> {
    fn into_vec(self) -> Vec<T> {
        self.0.into_iter().collect()
    }
}

enum Source {
    Flag(bool),
    From(syn::Type, syn::Expr),
}

impl syn::parse::Parse for Source {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::token::Comma;
        use syn::{Expr, Ident, LitBool, Type};

        let lookahead = input.lookahead1();

        if lookahead.peek(LitBool) {
            let val: LitBool = input.parse()?;
            Ok(Source::Flag(val.value))
        } else if lookahead.peek(Ident) {
            let name: Ident = input.parse()?;

            if name == "from" {
                let inside;
                parenthesized!(inside in input);
                let ty: Type = inside.parse()?;
                let _: Comma = inside.parse()?;
                let expr: Expr = inside.parse()?;

                Ok(Source::From(ty, expr))
            } else {
                Err(syn::Error::new(
                    name.span(),
                    "expected `true`, `false`, or `from`",
                ))
            }
        } else {
            Err(lookahead.error())
        }
    }
}

enum Backtrace {
    Flag(bool),
    Delegate,
}

impl syn::parse::Parse for Backtrace {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::{Ident, LitBool};

        let lookahead = input.lookahead1();

        if lookahead.peek(LitBool) {
            let val: LitBool = input.parse()?;
            Ok(Backtrace::Flag(val.value))
        } else if lookahead.peek(Ident) {
            let name: Ident = input.parse()?;

            if name == "delegate" {
                Ok(Backtrace::Delegate)
            } else {
                Err(syn::Error::new(
                    name.span(),
                    "expected `true`, `false`, or `delegate`",
                ))
            }
        } else {
            Err(lookahead.error())
        }
    }
}

enum SnafuAttribute {
    Display(UserInput),
    Visibility(UserInput),
    Source(Vec<Source>),
    Backtrace(Backtrace),
    DocComment(String),
}

impl SnafuAttribute {
    fn into_visibility(self) -> Option<UserInput> {
        match self {
            SnafuAttribute::Visibility(v) => Some(v),
            _ => None,
        }
    }
}

impl syn::parse::Parse for SnafuAttribute {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::{Expr, Ident, Visibility};

        let inside;
        parenthesized!(inside in input);
        let name: Ident = inside.parse()?;

        if name == "display" {
            let m: MyMeta<List<Expr>> = inside.parse()?;
            let v = m.into_option().ok_or_else(|| {
                syn::Error::new(name.span(), "`snafu(display)` requires an argument")
            })?;
            let v = Box::new(v.0);
            Ok(SnafuAttribute::Display(v))
        } else if name == "visibility" {
            let m: MyMeta<Visibility> = inside.parse()?;
            let v = m
                .into_option()
                .map_or_else(private_visibility, |v| Box::new(v) as UserInput);
            Ok(SnafuAttribute::Visibility(v))
        } else if name == "source" {
            if inside.is_empty() {
                Ok(SnafuAttribute::Source(vec![Source::Flag(true)]))
            } else {
                let v: MyParens<List<Source>> = inside.parse()?;
                Ok(SnafuAttribute::Source(v.0.into_vec()))
            }
        } else if name == "backtrace" {
            if inside.is_empty() {
                Ok(SnafuAttribute::Backtrace(Backtrace::Flag(true)))
            } else {
                let v: MyParens<Backtrace> = inside.parse()?;
                Ok(SnafuAttribute::Backtrace(v.0))
            }
        } else {
            Err(syn::Error::new(
                name.span(),
                "expected `display`, `visibility`, `source`, or `backtrace`",
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

struct DocComment(SnafuAttribute);

impl syn::parse::Parse for DocComment {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::token::Eq;
        use syn::LitStr;

        let _: Eq = input.parse()?;
        let doc: LitStr = input.parse()?;

        Ok(DocComment(SnafuAttribute::DocComment(doc.value())))
    }
}

fn attributes_from_syn(attrs: Vec<syn::Attribute>) -> MultiSynResult<Vec<SnafuAttribute>> {
    use syn::parse2;

    let mut ours = Vec::new();
    let mut errs = Vec::new();

    let parsed_attrs = attrs.into_iter().flat_map(|attr| {
        if attr.path.is_ident("snafu") {
            Some(parse2::<SnafuAttributeBody>(attr.tts).map(|body| body.0))
        } else if attr.path.is_ident("doc") {
            Some(parse2::<DocComment>(attr.tts).map(|comment| vec![comment.0]))
        } else {
            None
        }
    });

    for attr in parsed_attrs {
        match attr {
            Ok(v) => ours.extend(v),
            Err(e) => errs.push(e),
        }
    }

    if errs.is_empty() {
        Ok(ours)
    } else {
        Err(errs)
    }
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
                    #[derive(Debug, Copy, Clone)]
                    #visibility struct #selector_name;
                }
            } else {
                let visibilities = iter::repeat(visibility);

                quote! {
                    #[derive(Debug, Copy, Clone)]
                    #visibility struct #selector_name {
                        #( #visibilities #names: #types ),*
                    }
                }
            }
        };

        let backtrace_field = match *backtrace_field {
            Some(ref field) => {
                let name = &field.name;
                quote! { #name: std::default::Default::default(), }
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
                quote! { #name: self.#name.into() }
            });

            let source_ty;
            let source_xfer_field;

            match *source_field {
                Some(ref source_field) => {
                    let SourceField {
                        name: ref source_name,
                        transformation: ref source_transformation,
                    } = *source_field;

                    let source_ty2 = source_transformation.ty();
                    let source_transformation = source_transformation.transformation();

                    source_ty = quote! { #source_ty2 };
                    source_xfer_field = quote! { #source_name: (#source_transformation)(error), };
                }
                None => {
                    source_ty = quote! { snafu::NoneError };
                    source_xfer_field = quote! {};
                }
            }

            quote! {
                impl#generics_list snafu::IntoError<#parameterized_enum_name> for #selector_name
                where
                    #parameterized_enum_name: std::error::Error + snafu::ErrorCompat,
                    #(#where_clauses),*
                {
                    type Source = #source_ty;

                    fn into_error(self, error: Self::Source) -> #parameterized_enum_name {
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
                    ref doc_comment,
                    ..
                } = *variant;

                let format = match (display_format, source_field) {
                    (&Some(ref v), _) => quote! { #v },
                    (&None, _) if !doc_comment.is_empty() => {
                        quote! { #doc_comment }
                    }
                    (&None, &Some(ref f)) => {
                        let field_name = &f.name;
                        quote! { concat!(stringify!(#variant_name), ": {}"), #field_name }
                    }
                    (&None, &None) => quote! { stringify!(#variant_name)},
                };

                let field_names = user_fields
                    .iter()
                    .chain(backtrace_field)
                    .map(Field::name)
                    .chain(source_field.as_ref().map(SourceField::name));

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
                        let SourceField {
                            name: ref field_name,
                            ..
                        } = *source_field;
                        quote! {
                            #enum_name::#variant_name { ref #field_name, .. } => {
                                std::option::Option::Some(#field_name.as_error_source())
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
                use snafu::AsErrorSource;
                match *self {
                    #(#variants_to_source)*
                }
            }
        };

        let source_fn = if cfg!(feature = "rust_1_30") {
            quote! {
                fn source(&self) -> Option<&(std::error::Error + 'static)> {
                    use snafu::AsErrorSource;
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
            generics,
            name,
            transformation,
        } = self;

        let inner_type = transformation.ty();
        let transformation = transformation.transformation();

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
            impl#generics std::convert::From<#inner_type> for #parameterized_struct_name
            where
                #(#where_clauses),*
            {
                fn from(other: #inner_type) -> Self {
                    #name((#transformation)(other))
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

mod sponge {
    use std::iter::FromIterator;

    pub struct AllErrors<T, E>(Result<T, Vec<E>>);

    impl<T, E> AllErrors<T, E> {
        pub fn into_result(self) -> Result<T, Vec<E>> {
            self.0
        }
    }

    impl<C, T, E> FromIterator<Result<C, Vec<E>>> for AllErrors<T, E>
    where
        T: FromIterator<C>,
    {
        fn from_iter<I>(i: I) -> Self
        where
            I: IntoIterator<Item = Result<C, Vec<E>>>,
        {
            let mut errors = Vec::new();

            let inner = i
                .into_iter()
                .flat_map(|v| match v {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        errors.extend(e);
                        Err(())
                    }
                })
                .collect();

            if errors.is_empty() {
                AllErrors(Ok(inner))
            } else {
                AllErrors(Err(errors))
            }
        }
    }
}
