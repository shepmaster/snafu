#![recursion_limit = "128"] // https://github.com/rust-lang/rust/issues/62059

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::collections::BTreeSet;

mod parse;
mod shared;

// The snafu crate re-exports this and adds useful documentation.
#[proc_macro_derive(Snafu, attributes(snafu))]
pub fn snafu_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Could not parse type to derive Error for");

    impl_snafu_macro(ast)
}

mod report;

#[proc_macro_attribute]
pub fn report(attr: TokenStream, item: TokenStream) -> TokenStream {
    report::body(attr, item)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Some arbitrary tokens we treat as a black box
type UserInput = Box<dyn quote::ToTokens>;

enum ModuleName {
    Default,
    Custom(syn::Ident),
}

enum SnafuInfo {
    Enum(EnumInfo),
    NamedStruct(NamedStructInfo),
    TupleStruct(TupleStructInfo),
}

struct EnumInfo {
    crate_root: UserInput,
    name: syn::Ident,
    generics: syn::Generics,
    variants: Vec<FieldContainer>,
    default_visibility: Option<UserInput>,
    default_suffix: SuffixKind,
    module: Option<ModuleName>,
}

/// A struct or enum variant, with named fields.
struct FieldContainer {
    name: syn::Ident,
    backtrace_field: Option<Field>,
    implicit_fields: Vec<Field>,
    selector_kind: ContextSelectorKind,
    display_format: Option<Display>,
    doc_comment: Option<DocComment>,
    visibility: Option<UserInput>,
    module: Option<ModuleName>,
    provides: Vec<Provide>,
    is_transparent: bool,
}

impl FieldContainer {
    fn user_fields(&self) -> &[Field] {
        self.selector_kind.user_fields()
    }

    fn provides(&self) -> &[Provide] {
        &self.provides
    }
}

struct Provide {
    is_opt: bool,
    is_ref: bool,
    ty: syn::Type,
    expr: syn::Expr,
}

enum ContextSelectorName {
    Provided(syn::Ident),
    Suffixed(SuffixKind),
}

impl ContextSelectorName {
    const SUFFIX_DEFAULT: Self = ContextSelectorName::Suffixed(SuffixKind::Default);

    fn resolve_name(&self, def: &SuffixKind, base_name: &syn::Ident) -> syn::Ident {
        match self {
            ContextSelectorName::Provided(ident) => ident.clone(),
            ContextSelectorName::Suffixed(suffix_kind) => suffix_kind.resolve_name(def, base_name),
        }
    }
}

#[derive(Default)]
enum SuffixKind {
    #[default]
    Default,
    None,
    Some(syn::Ident),
}

impl SuffixKind {
    const DEFAULT_SUFFIX: &'static str = "Snafu";

    fn resolve_name(&self, def: &Self, base_name: &syn::Ident) -> syn::Ident {
        let span = base_name.span();

        let base_name = base_name.to_string();
        let base_name = base_name.trim_end_matches("Error");

        let suffix = self
            .as_option()
            .or_else(|| def.as_option())
            .unwrap_or(&Self::DEFAULT_SUFFIX);

        quote::format_ident!("{}{}", base_name, suffix, span = span)
    }

    fn as_option(&self) -> Option<&dyn quote::IdentFragment> {
        match self {
            SuffixKind::Default => None,
            SuffixKind::None => Some(&""),
            SuffixKind::Some(s) => Some(s),
        }
    }
}

enum ContextSelectorKind {
    Context {
        selector_name: ContextSelectorName,
        source_field: Option<SourceField>,
        user_fields: Vec<Field>,
    },

    Whatever {
        source_field: Option<SourceField>,
        message_field: Field,
    },

    NoContext {
        source_field: SourceField,
    },
}

impl ContextSelectorKind {
    fn is_whatever(&self) -> bool {
        matches!(self, ContextSelectorKind::Whatever { .. })
    }

    fn user_fields(&self) -> &[Field] {
        match self {
            ContextSelectorKind::Context { user_fields, .. } => user_fields,
            ContextSelectorKind::Whatever { .. } => &[],
            ContextSelectorKind::NoContext { .. } => &[],
        }
    }

    fn source_field(&self) -> Option<&SourceField> {
        match self {
            ContextSelectorKind::Context { source_field, .. } => source_field.as_ref(),
            ContextSelectorKind::Whatever { source_field, .. } => source_field.as_ref(),
            ContextSelectorKind::NoContext { source_field } => Some(source_field),
        }
    }

    fn message_field(&self) -> Option<&Field> {
        match self {
            ContextSelectorKind::Context { .. } => None,
            ContextSelectorKind::Whatever { message_field, .. } => Some(message_field),
            ContextSelectorKind::NoContext { .. } => None,
        }
    }

    fn resolve_name(&self, def: &SuffixKind, base_name: &syn::Ident) -> syn::Ident {
        let selector_name = match self {
            ContextSelectorKind::Context { selector_name, .. } => selector_name,
            _ => &ContextSelectorName::SUFFIX_DEFAULT,
        };

        selector_name.resolve_name(def, base_name)
    }
}

struct NamedStructInfo {
    crate_root: UserInput,
    field_container: FieldContainer,
    generics: syn::Generics,
}

struct TupleStructInfo {
    crate_root: UserInput,
    name: syn::Ident,
    generics: syn::Generics,
    transformation: Transformation,
    provides: Vec<Provide>,
}

#[derive(Clone)]
pub(crate) struct Field {
    name: syn::Ident,
    ty: syn::Type,
    provide: bool,
    original: syn::Field,
}

impl quote::ToTokens for Field {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.original.to_tokens(tokens);
    }
}

impl Field {
    fn name(&self) -> &syn::Ident {
        &self.name
    }
}

struct SourceField {
    name: syn::Ident,
    transformation: Transformation,
    backtrace_delegate: bool,
    provide: bool,
}

impl SourceField {
    fn name(&self) -> &syn::Ident {
        &self.name
    }
}

enum Transformation {
    None {
        target_ty: syn::Type,
        from_is_generic: bool,
    },

    Transform {
        source_ty: syn::Type,
        target_ty: syn::Type,
        expr: syn::Expr,
    },
}

impl Transformation {
    fn source_ty(&self) -> &syn::Type {
        match self {
            Transformation::None { target_ty, .. } => target_ty,
            Transformation::Transform { source_ty, .. } => source_ty,
        }
    }

    fn target_ty(&self) -> &syn::Type {
        match self {
            Transformation::None { target_ty, .. } => target_ty,
            Transformation::Transform { target_ty, .. } => target_ty,
        }
    }

    fn transformation(&self) -> proc_macro2::TokenStream {
        match self {
            Transformation::None {
                from_is_generic: false,
                ..
            } => quote! { |v| v },

            Transformation::None {
                from_is_generic: true,
                ..
            } => quote! { ::core::convert::Into::into },

            Transformation::Transform { expr, .. } => quote! { #expr },
        }
    }

    fn is_generic(&self) -> bool {
        matches!(self, Transformation::None { from_is_generic, .. } if *from_is_generic)
    }
}

fn impl_snafu_macro(ty: syn::DeriveInput) -> TokenStream {
    match parse_snafu_information(ty) {
        Ok(info) => info.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn parse_snafu_information(ty: syn::DeriveInput) -> syn::Result<SnafuInfo> {
    use syn::Data;

    let syn::DeriveInput {
        ident,
        generics,
        data,
        attrs,
        ..
    } = &ty;

    match data {
        Data::Enum(enum_) => parse::parse_enum(enum_, ident, generics, attrs).map(SnafuInfo::Enum),
        Data::Struct(struct_) => parse_snafu_struct(struct_, ident, generics, attrs, &ty),
        _ => {
            let txt = "Can only derive `Snafu` for an enum or a newtype";
            Err(syn::Error::new_spanned(&ty, txt))
        }
    }
}

fn parse_snafu_struct(
    struct_: &syn::DataStruct,
    name: &syn::Ident,
    generics: &syn::Generics,
    attrs: &[syn::Attribute],
    span: impl quote::ToTokens,
) -> syn::Result<SnafuInfo> {
    use syn::Fields;

    match &struct_.fields {
        Fields::Named(f) => {
            let f = f.named.iter().collect::<Vec<_>>();
            parse::parse_named_struct(&f, name, generics, attrs, span).map(SnafuInfo::NamedStruct)
        }
        Fields::Unnamed(f) => {
            parse::parse_tuple_struct(f, name, generics, attrs, span).map(SnafuInfo::TupleStruct)
        }
        Fields::Unit => {
            parse::parse_named_struct(&[], name, generics, attrs, span).map(SnafuInfo::NamedStruct)
        }
    }
}

struct Display {
    exprs: Vec<syn::Expr>,
    shorthand_names: BTreeSet<syn::Ident>,
    assigned_names: BTreeSet<syn::Ident>,
}

#[derive(Default)]
struct DocComment {
    content: String,
    shorthand_names: BTreeSet<syn::Ident>,
}

fn private_visibility() -> UserInput {
    Box::new(quote! {})
}

// Private context selectors wouldn't be accessible outside the
// module, so we use `pub(super)`.
fn default_context_selector_visibility_in_module() -> proc_macro2::TokenStream {
    quote! { pub(super) }
}

impl From<SnafuInfo> for proc_macro::TokenStream {
    fn from(other: SnafuInfo) -> proc_macro::TokenStream {
        match other {
            SnafuInfo::Enum(e) => e.into(),
            SnafuInfo::NamedStruct(s) => s.into(),
            SnafuInfo::TupleStruct(s) => s.into(),
        }
    }
}

impl From<EnumInfo> for proc_macro::TokenStream {
    fn from(other: EnumInfo) -> proc_macro::TokenStream {
        other.generate_snafu().into()
    }
}

impl From<NamedStructInfo> for proc_macro::TokenStream {
    fn from(other: NamedStructInfo) -> proc_macro::TokenStream {
        other.generate_snafu().into()
    }
}

impl From<TupleStructInfo> for proc_macro::TokenStream {
    fn from(other: TupleStructInfo) -> proc_macro::TokenStream {
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
        use syn::{ConstParam, GenericParam, LifetimeParam, TypeParam};

        self.generics()
            .params
            .iter()
            .map(|p| match p {
                GenericParam::Type(TypeParam { ident, .. }) => quote! { #ident },
                GenericParam::Lifetime(LifetimeParam { lifetime, .. }) => quote! { #lifetime },
                GenericParam::Const(ConstParam { ident, .. }) => quote! { #ident },
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

        let context = match &self.module {
            None => quote! { #context_selectors },
            Some(module_name) => {
                use crate::shared::ContextModule;

                let context_module = ContextModule {
                    container_name: self.name(),
                    body: &context_selectors,
                    visibility: Some(&self.default_visibility),
                    module_name,
                };

                quote! { #context_module }
            }
        };

        quote! {
            #context
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

struct ContextSelector<'a>(&'a EnumInfo, &'a FieldContainer);

impl<'a> quote::ToTokens for ContextSelector<'a> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        use crate::shared::ContextSelector;

        let enum_name = &self.0.name;
        let default_suffix = &self.0.default_suffix;

        let FieldContainer {
            name: variant_name,
            selector_kind,
            ..
        } = self.1;

        let default_visibility;
        let selector_visibility = match (
            &self.1.visibility,
            &self.0.default_visibility,
            &self.0.module,
        ) {
            (Some(v), _, _) | (_, Some(v), _) => Some(&**v),
            (None, None, Some(_)) => {
                default_visibility = default_context_selector_visibility_in_module();
                Some(&default_visibility as _)
            }
            (None, None, None) => None,
        };

        let selector_doc_string = format!(
            "SNAFU context selector for the `{}::{}` variant",
            enum_name, variant_name,
        );

        let context_selector = ContextSelector {
            backtrace_field: self.1.backtrace_field.as_ref(),
            implicit_fields: &self.1.implicit_fields,
            crate_root: &self.0.crate_root,
            error_constructor_name: &quote! { #enum_name::#variant_name },
            original_generics_without_defaults: shared::GenericsWithoutDefaults::new(
                &self.0.generics,
            ),
            parameterized_error_name: &self.0.parameterized_name(),
            selector_doc_string: &selector_doc_string,
            selector_kind,
            selector_base_name: variant_name,
            user_fields: selector_kind.user_fields(),
            visibility: selector_visibility,
            where_clauses: &self.0.provided_where_clauses(),
            default_suffix,
        };

        stream.extend(quote! { #context_selector });
    }
}

struct DisplayImpl<'a>(&'a EnumInfo);

impl<'a> quote::ToTokens for DisplayImpl<'a> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        use self::shared::{Display, DisplayMatchArm};

        let enum_name = &self.0.name;

        let arms: Vec<_> = self
            .0
            .variants
            .iter()
            .map(|variant| {
                let FieldContainer {
                    display_format,
                    doc_comment,
                    name: variant_name,
                    selector_kind,
                    ..
                } = variant;

                let arm = DisplayMatchArm {
                    field_container: variant,
                    default_name: &variant_name,
                    display_format: display_format.as_ref(),
                    doc_comment: doc_comment.as_ref(),
                    pattern_ident: &quote! { #enum_name::#variant_name },
                    selector_kind,
                };

                quote! { #arm }
            })
            .collect();

        let display = Display {
            arms: &arms,
            original_generics: shared::GenericsWithoutDefaults::new(self.0.generics()),
            parameterized_error_name: &self.0.parameterized_name(),
            where_clauses: &self.0.provided_where_clauses(),
        };

        let display_impl = quote! { #display };

        stream.extend(display_impl)
    }
}

struct ErrorImpl<'a>(&'a EnumInfo);

impl<'a> quote::ToTokens for ErrorImpl<'a> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        use self::shared::{Error, ErrorProvideMatchArm, ErrorSourceMatchArm};

        let crate_root = &self.0.crate_root;

        let mut variants_to_source = Vec::with_capacity(self.0.variants.len());
        let mut variants_to_provide = Vec::with_capacity(self.0.variants.len());

        for field_container in &self.0.variants {
            let enum_name = &self.0.name;
            let variant_name = &field_container.name;
            let pattern_ident = &quote! { #enum_name::#variant_name };

            let error_source_match_arm = ErrorSourceMatchArm {
                field_container,
                pattern_ident,
            };
            let error_source_match_arm = quote! { #error_source_match_arm };

            let error_provide_match_arm = ErrorProvideMatchArm {
                crate_root,
                field_container,
                pattern_ident,
            };
            let error_provide_match_arm = quote! { #error_provide_match_arm };

            variants_to_source.push(error_source_match_arm);
            variants_to_provide.push(error_provide_match_arm);
        }

        let error_impl = Error {
            crate_root,
            parameterized_error_name: &self.0.parameterized_name(),
            source_arms: &variants_to_source,
            original_generics: shared::GenericsWithoutDefaults::new(&self.0.generics),
            where_clauses: &self.0.provided_where_clauses(),
            provide_arms: &variants_to_provide,
        };
        let error_impl = quote! { #error_impl };

        stream.extend(error_impl);
    }
}

struct ErrorCompatImpl<'a>(&'a EnumInfo);

impl<'a> quote::ToTokens for ErrorCompatImpl<'a> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        use self::shared::{ErrorCompat, ErrorCompatBacktraceMatchArm};

        let variants_to_backtrace: Vec<_> = self
            .0
            .variants
            .iter()
            .map(|field_container| {
                let crate_root = &self.0.crate_root;
                let enum_name = &self.0.name;
                let variant_name = &field_container.name;

                let match_arm = ErrorCompatBacktraceMatchArm {
                    field_container,
                    crate_root,
                    pattern_ident: &quote! { #enum_name::#variant_name },
                };

                quote! { #match_arm }
            })
            .collect();

        let error_compat_impl = ErrorCompat {
            crate_root: &self.0.crate_root,
            parameterized_error_name: &self.0.parameterized_name(),
            backtrace_arms: &variants_to_backtrace,
            original_generics: shared::GenericsWithoutDefaults::new(&self.0.generics),
            where_clauses: &self.0.provided_where_clauses(),
        };

        let error_compat_impl = quote! { #error_compat_impl };

        stream.extend(error_compat_impl);
    }
}

impl NamedStructInfo {
    fn generate_snafu(self) -> proc_macro2::TokenStream {
        let parameterized_struct_name = self.parameterized_name();
        let original_generics = shared::GenericsWithoutDefaults::new(&self.generics);
        let where_clauses = self.provided_where_clauses();

        let Self {
            crate_root,
            field_container:
                FieldContainer {
                    name,
                    selector_kind,
                    backtrace_field,
                    implicit_fields,
                    display_format,
                    doc_comment,
                    visibility,
                    module,
                    ..
                },
            ..
        } = &self;
        let field_container = &self.field_container;

        let user_fields = selector_kind.user_fields();

        use crate::shared::{Error, ErrorProvideMatchArm, ErrorSourceMatchArm};

        let pattern_ident = &quote! { Self };

        let error_source_match_arm = ErrorSourceMatchArm {
            field_container,
            pattern_ident,
        };
        let error_source_match_arm = quote! { #error_source_match_arm };

        let error_provide_match_arm = ErrorProvideMatchArm {
            crate_root: &crate_root,
            field_container,
            pattern_ident,
        };
        let error_provide_match_arm = quote! { #error_provide_match_arm };

        let error_impl = Error {
            crate_root: &crate_root,
            original_generics,
            parameterized_error_name: &parameterized_struct_name,
            provide_arms: &[error_provide_match_arm],
            source_arms: &[error_source_match_arm],
            where_clauses: &where_clauses,
        };
        let error_impl = quote! { #error_impl };

        use self::shared::{ErrorCompat, ErrorCompatBacktraceMatchArm};

        let match_arm = ErrorCompatBacktraceMatchArm {
            field_container,
            crate_root: &crate_root,
            pattern_ident: &quote! { Self },
        };
        let match_arm = quote! { #match_arm };

        let error_compat_impl = ErrorCompat {
            crate_root: &crate_root,
            parameterized_error_name: &parameterized_struct_name,
            backtrace_arms: &[match_arm],
            original_generics,
            where_clauses: &where_clauses,
        };

        use crate::shared::{Display, DisplayMatchArm};

        let arm = DisplayMatchArm {
            field_container,
            default_name: &name,
            display_format: display_format.as_ref(),
            doc_comment: doc_comment.as_ref(),
            pattern_ident: &quote! { Self },
            selector_kind,
        };
        let arm = quote! { #arm };

        let display_impl = Display {
            arms: &[arm],
            original_generics,
            parameterized_error_name: &parameterized_struct_name,
            where_clauses: &where_clauses,
        };

        use crate::shared::ContextSelector;

        let selector_doc_string = format!("SNAFU context selector for the `{}` error", name);

        let default_visibility;
        let selector_visibility = match (visibility, module) {
            (Some(v), _) => Some(&**v),
            (None, Some(_)) => {
                default_visibility = default_context_selector_visibility_in_module();
                Some(&default_visibility as _)
            }
            (None, None) => None,
        };

        let context_selector = ContextSelector {
            backtrace_field: backtrace_field.as_ref(),
            implicit_fields,
            crate_root: &crate_root,
            error_constructor_name: &name,
            original_generics_without_defaults: shared::GenericsWithoutDefaults::new(
                self.generics(),
            ),
            parameterized_error_name: &parameterized_struct_name,
            selector_doc_string: &selector_doc_string,
            selector_kind,
            selector_base_name: &field_container.name,
            user_fields,
            visibility: selector_visibility,
            where_clauses: &where_clauses,
            default_suffix: &SuffixKind::Default,
        };

        let context = match module {
            None => quote! { #context_selector },
            Some(module_name) => {
                use crate::shared::ContextModule;

                let context_module = ContextModule {
                    container_name: self.name(),
                    body: &context_selector,
                    visibility: visibility.as_ref().map(|x| &**x),
                    module_name,
                };

                quote! { #context_module }
            }
        };

        quote! {
            #error_impl
            #error_compat_impl
            #display_impl
            #context
        }
    }
}

impl GenericAwareNames for NamedStructInfo {
    fn name(&self) -> &syn::Ident {
        &self.field_container.name
    }

    fn generics(&self) -> &syn::Generics {
        &self.generics
    }
}

impl TupleStructInfo {
    fn generate_snafu(self) -> proc_macro2::TokenStream {
        let parameterized_struct_name = self.parameterized_name();

        let TupleStructInfo {
            crate_root,
            generics,
            name,
            provides,
            transformation,
        } = self;

        let where_clauses: Vec<_> = generics
            .where_clause
            .iter()
            .flat_map(|c| c.predicates.iter().map(|p| quote! { #p }))
            .collect();

        let generics = shared::GenericsWithoutDefaults::new(&generics);

        let description_fn = quote! {
            fn description(&self) -> &str {
                #crate_root::Error::description(&self.0)
            }
        };

        let cause_fn = quote! {
            fn cause(&self) -> ::core::option::Option<&dyn #crate_root::Error> {
                #crate_root::Error::cause(&self.0)
            }
        };

        let source_fn = quote! {
            fn source(&self) -> ::core::option::Option<&(dyn #crate_root::Error + 'static)> {
                #crate_root::Error::source(&self.0)
            }
        };

        let backtrace_fn = quote! {
            fn backtrace(&self) -> ::core::option::Option<&#crate_root::Backtrace> {
                #crate_root::ErrorCompat::backtrace(&self.0)
            }
        };

        let provide_fn = if cfg!(feature = "unstable-provider-api") {
            use shared::error::PROVIDE_ARG;

            let explicit_calls = shared::error::quote_provides(&provides);

            Some(quote! {
                fn provide<'a>(&'a self, #PROVIDE_ARG: &mut #crate_root::error::Request<'a>) {
                    match self {
                        Self(v) => {
                            #(#explicit_calls;)*
                        }
                    };
                }
            })
        } else {
            None
        };

        let error_impl = quote! {
            #[allow(single_use_lifetimes)]
            impl<#generics> #crate_root::Error for #parameterized_struct_name
            where
                #(#where_clauses),*
            {
                #description_fn
                #cause_fn
                #source_fn
                #provide_fn
            }
        };

        let error_compat_impl = quote! {
            #[allow(single_use_lifetimes)]
            impl<#generics> #crate_root::ErrorCompat for #parameterized_struct_name
            where
                #(#where_clauses),*
            {
                #backtrace_fn
            }
        };

        let display_impl = quote! {
            #[allow(single_use_lifetimes)]
            impl<#generics> ::core::fmt::Display for #parameterized_struct_name
            where
                #(#where_clauses),*
            {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Display::fmt(&self.0, f)
                }
            }
        };

        let tuple_field = quote! { 0 };
        let source_info = shared::SourceInfo::from_transformation(&tuple_field, &transformation);
        // FUTURE: Should we support implicit fields in opaque / tuple structs?
        let construct_implicit_fields_with_source = quote! {};

        let from_impl = shared::NoContextSelector {
            source_info,
            parameterized_error_name: &parameterized_struct_name,
            generics,
            where_clauses: &where_clauses,
            error_constructor_name: &name,
            construct_implicit_fields_with_source,
        };

        quote! {
            #error_impl
            #error_compat_impl
            #display_impl
            #from_impl
        }
    }
}

impl GenericAwareNames for TupleStructInfo {
    fn name(&self) -> &syn::Ident {
        &self.name
    }

    fn generics(&self) -> &syn::Generics {
        &self.generics
    }
}
