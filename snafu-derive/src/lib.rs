#![allow(unknown_lints, bare_trait_objects)]
#![recursion_limit = "128"] // https://github.com/rust-lang/rust/issues/62059

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use std::collections::VecDeque;
use std::fmt;
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
    backtrace_delegate: bool,
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

/// SyntaxErrors is a convenience wrapper for a list of syntax errors discovered while parsing
/// something that derives Snafu.  It makes it easier for developers to add and return syntax
/// errors while walking through the parse tree.
#[derive(Debug, Default)]
struct SyntaxErrors {
    inner: Vec<syn::Error>,
}

impl SyntaxErrors {
    /// Adds a new syntax error.  The given description will be used in the compile error pointing
    /// to the given span.  Helper structs are available to format common descriptions, e.g.
    /// OnlyValidOn and DuplicateAttribute.
    fn add<D, T>(&mut self, tokens: T, description: D)
    where
        D: fmt::Display,
        T: quote::ToTokens,
    {
        self.inner
            .push(syn::Error::new_spanned(tokens, description));
    }

    /// Adds the given list of errors.
    fn extend(&mut self, errors: Vec<syn::Error>) {
        self.inner.extend(errors);
    }

    #[allow(dead_code)]
    /// Returns the number of errors that have been added.
    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Consume the SyntaxErrors, returning Ok if there were no syntax errors added, or Err(list)
    /// if there were syntax errors.
    fn finish(self) -> MultiSynResult<()> {
        if self.inner.is_empty() {
            Ok(())
        } else {
            Err(self.inner)
        }
    }
}

/// Helper structure to simplify parameters to SyntaxErrors.add, handling cases where an attribute
/// was used on an element where it's not valid.
#[derive(Debug)]
struct OnlyValidOn {
    /// The name of the attribute that was misused.
    attribute: &'static str,
    /// A description of where that attribute is valid.
    valid_on: &'static str,
    /// A description of where the attribute was incorrectly used.
    not_on: &'static str,
}

impl fmt::Display for OnlyValidOn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` attribute is only valid on {}, not on {}",
            self.attribute, self.valid_on, self.not_on
        )
    }
}

/// Helper structure to simplify parameters to SyntaxErrors.add, handling cases where two
/// incompatible attributes were specified on the same element.
#[derive(Debug)]
struct IncompatibleAttributes {
    attributes: &'static [&'static str],
    location: &'static str,
}

impl fmt::Display for IncompatibleAttributes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let attrs_string = self
            .attributes
            .iter()
            .map(|attr| format!("`{}`", attr))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "Incompatible attributes [{}] specified on {}",
            attrs_string, self.location,
        )
    }
}

/// Helper structure to simplify parameters to SyntaxErrors.add, handling cases where an attribute
/// was incorrectly used multiple times on the same element.
#[derive(Debug)]
struct DuplicateAttribute {
    attribute: &'static str,
    location: &'static str,
}

impl fmt::Display for DuplicateAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Multiple `{}` attributes are not supported {}",
            self.attribute, self.location,
        )
    }
}

/// AtMostOne is a helper to track attributes seen during parsing.  If more than one item is added,
/// it's added to a list of DuplicateAttribute errors, using the given `name` and `location` as
/// descriptors.
///
/// When done parsing a structure, call `finish` to get first attribute found, if any, and the list
/// of errors, or call `finish_with_location` to get the attribute and the token tree where it was
/// found, which can be useful for error reporting.
#[derive(Debug)]
struct AtMostOne<T, U>
where
    U: quote::ToTokens,
{
    name: &'static str,
    location: &'static str,
    // We store all the values we've seen to allow for `iter`, which helps the `AtMostOne` be
    // useful for additional manual error checking.
    values: VecDeque<(T, U)>,
    errors: SyntaxErrors,
}

impl<T, U> AtMostOne<T, U>
where
    U: quote::ToTokens + Clone,
{
    /// Creates an AtMostOne to track an attribute with the given `name` on the given `location`.
    /// `location` is a string describing where the attribute can be specified, often referencing a
    /// parent element, for example "on a field".
    fn new(name: &'static str, location: &'static str) -> Self {
        Self {
            name,
            location,
            values: VecDeque::new(),
            errors: SyntaxErrors::default(),
        }
    }

    /// Add an occurence of the attribute found at the given token tree `tokens`.
    fn add(&mut self, item: T, tokens: U) {
        if !self.values.is_empty() {
            self.errors.add(
                tokens.clone(),
                DuplicateAttribute {
                    attribute: self.name,
                    location: self.location,
                },
            );
        }
        self.values.push_back((item, tokens));
    }

    #[allow(dead_code)]
    /// Returns the number of elements that have been added.
    fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if no elements have been added, otherwise false.
    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns an iterator over all values that have been added.
    ///
    /// This can help with additional manual error checks beyond the duplication checks that
    /// `AtMostOne` handles for you.
    fn iter(&self) -> std::collections::vec_deque::Iter<(T, U)> {
        self.values.iter()
    }

    /// Consumes the AtMostOne, returning the first item added, if any, and the list of errors
    /// representing any items added beyond the first.
    fn finish(self) -> (Option<T>, Vec<syn::Error>) {
        let (value, errors) = self.finish_with_location();
        (value.map(|(val, _location)| val), errors)
    }

    /// Like `finish` but also returns the location of the first item added.  Useful when you have
    /// to do additional, manual error checking on the first item added, and you'd like to report
    /// an accurate location for it in case of errors.
    fn finish_with_location(mut self) -> (Option<(T, U)>, Vec<syn::Error>) {
        let errors = match self.errors.finish() {
            Ok(()) => Vec::new(),
            Err(vec) => vec,
        };
        (self.values.pop_front(), errors)
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
    use quote::ToTokens;
    use syn::spanned::Spanned;
    use syn::Fields;

    let mut errors = SyntaxErrors::default();

    let mut default_visibilities = AtMostOne::new("visibility", "on an error enum");

    for attr in attributes_from_syn(attrs)? {
        match attr {
            SnafuAttribute::Visibility(tokens, v) => {
                default_visibilities.add(v, tokens);
            }
            SnafuAttribute::Display(tokens, ..) => {
                errors.add(
                    tokens,
                    OnlyValidOn {
                        attribute: "display",
                        valid_on: "variants of an error enum",
                        not_on: "an enum",
                    },
                );
            }
            SnafuAttribute::Source(tokens, ss) => {
                for s in ss {
                    match s {
                        Source::Flag(..) => {
                            errors.add(
                                tokens.clone(),
                                OnlyValidOn {
                                    attribute: "source(bool)",
                                    valid_on: "fields of an error variant",
                                    not_on: "an enum",
                                },
                            );
                        }
                        Source::From(_t, _e) => {
                            errors.add(
                                tokens.clone(),
                                OnlyValidOn {
                                    attribute: "source(from)",
                                    valid_on: "fields of an error variant",
                                    not_on: "an enum",
                                },
                            );
                        }
                    }
                }
            }
            SnafuAttribute::Backtrace(tokens, ..) => {
                errors.add(
                    tokens,
                    OnlyValidOn {
                        attribute: "backtrace",
                        valid_on: "fields of an error variant",
                        not_on: "an enum",
                    },
                );
            }
            SnafuAttribute::DocComment(..) => { /* Just a regular doc comment. */ }
        }
    }

    let (maybe_default_visibility, errs) = default_visibilities.finish();
    let default_visibility = maybe_default_visibility.unwrap_or_else(private_visibility);
    errors.extend(errs);

    let variants: sponge::AllErrors<_, _> = enum_
        .variants
        .into_iter()
        .map(|variant| {
            let name = variant.ident;

            let mut display_formats = AtMostOne::new("display", "on an error variant");
            let mut visibilities = AtMostOne::new("visibility", "on an error variant");
            let mut doc_comment = String::new();
            let mut reached_end_of_doc_comment = false;

            for attr in attributes_from_syn(variant.attrs)? {
                match attr {
                    SnafuAttribute::Display(tokens, d) => display_formats.add(d, tokens),
                    SnafuAttribute::Visibility(tokens, v) => visibilities.add(v, tokens),
                    SnafuAttribute::Source(tokens, ..) => {
                        errors.add(
                            tokens,
                            OnlyValidOn {
                                attribute: "source",
                                valid_on: "fields of an error variant",
                                not_on: "a variant",
                            },
                        );
                    }
                    SnafuAttribute::Backtrace(tokens, ..) => {
                        errors.add(
                            tokens,
                            OnlyValidOn {
                                attribute: "backtrace",
                                valid_on: "fields of an error variant",
                                not_on: "a variant",
                            },
                        );
                    }
                    SnafuAttribute::DocComment(_tts, doc_comment_line) => {
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
            let mut source_fields = AtMostOne::new("source", "within an error variant");
            let mut backtrace_fields = AtMostOne::new("backtrace", "within an error variant");

            for syn_field in fields {
                let span = syn_field.span();
                let name = syn_field
                    .ident
                    .as_ref()
                    .ok_or_else(|| vec![syn::Error::new(span, "Must have a named field")])?;
                let field = Field {
                    name: name.clone(),
                    ty: syn_field.ty.clone(),
                };

                // Check whether we have multiple source/backtrace attributes on this field.
                // We can't just add to source_fields/backtrace_fields from inside the attribute
                // loop because source and backtrace are connected and require a bit of special
                // logic after the attribute loop.  For example, we need to know whether there's a
                // source transformation before we record a source field, but it might be on a
                // later attribute.  We use the data field of `source_attrs` to track any
                // transformations in case it was a `source(from(...))`, but for backtraces we
                // don't need any more data.
                let mut source_attrs = AtMostOne::new("source", "on a field");
                let mut backtrace_attrs = AtMostOne::new("backtrace", "on a field");

                // Keep track of the negative markers so we can check for inconsistencies and
                // exclude fields even if they have the "source" or "backtrace" name.
                let mut source_opt_out = false;
                let mut backtrace_opt_out = false;

                for attr in attributes_from_syn(syn_field.attrs.clone())? {
                    match attr {
                        SnafuAttribute::Source(tokens, ss) => {
                            static INCOMPATIBLE: &[&str] = &["source(false)", "source(from)"];
                            for s in ss {
                                match s {
                                    Source::Flag(v) => {
                                        // If we've seen a `source(from)` then there will be a
                                        // `Some` value in `source_attrs`.
                                        let seen_source_from = source_attrs
                                            .iter()
                                            .map(|(val, _location)| val)
                                            .any(Option::is_some);
                                        if !v && seen_source_from {
                                            errors.add(
                                                tokens.clone(),
                                                IncompatibleAttributes {
                                                    attributes: INCOMPATIBLE,
                                                    location: "a field",
                                                },
                                            );
                                        }
                                        if v {
                                            source_attrs.add(None, tokens.clone());
                                        } else if name == "source" {
                                            source_opt_out = true;
                                        } else {
                                            errors.add(
                                                tokens.clone(),
                                                OnlyValidOn {
                                                    attribute: "source(false)",
                                                    valid_on: "a field named \"source\"",
                                                    not_on: "other fields",
                                                },
                                            );
                                        }
                                    }
                                    Source::From(t, e) => {
                                        if source_opt_out {
                                            errors.add(
                                                tokens.clone(),
                                                IncompatibleAttributes {
                                                    attributes: INCOMPATIBLE,
                                                    location: "a field",
                                                },
                                            );
                                        }
                                        source_attrs.add(Some((t, e)), tokens.clone());
                                    }
                                }
                            }
                        }
                        SnafuAttribute::Backtrace(tokens, v) => {
                            if v {
                                backtrace_attrs.add((), tokens);
                            } else if name == "backtrace" {
                                backtrace_opt_out = true;
                            } else {
                                errors.add(
                                    tokens,
                                    OnlyValidOn {
                                        attribute: "backtrace(false)",
                                        valid_on: "a field named \"backtrace\"",
                                        not_on: "other fields",
                                    },
                                );
                            }
                        }
                        SnafuAttribute::Visibility(tokens, ..) => {
                            errors.add(
                                tokens,
                                OnlyValidOn {
                                    attribute: "visibility",
                                    valid_on: "an error enum and its variants",
                                    not_on: "a field",
                                },
                            );
                        }
                        SnafuAttribute::Display(tokens, ..) => {
                            errors.add(
                                tokens,
                                OnlyValidOn {
                                    attribute: "display",
                                    valid_on: "variants of an error enum",
                                    not_on: "a field",
                                },
                            );
                        }
                        SnafuAttribute::DocComment(..) => { /* Just a regular doc comment. */ }
                    }
                }

                // Add errors for any duplicated attributes on this field.
                let (source_attr, errs) = source_attrs.finish_with_location();
                errors.extend(errs);
                let (backtrace_attr, errs) = backtrace_attrs.finish_with_location();
                errors.extend(errs);

                let source_attr = source_attr.or_else(|| {
                    if field.name == "source" && !source_opt_out {
                        Some((None, syn_field.clone().into_token_stream()))
                    } else {
                        None
                    }
                });

                let backtrace_attr = backtrace_attr.or_else(|| {
                    if field.name == "backtrace" && !backtrace_opt_out {
                        Some(((), syn_field.clone().into_token_stream()))
                    } else {
                        None
                    }
                });

                if let Some((maybe_transformation, location)) = source_attr {
                    let Field { name, ty } = field;
                    let transformation = maybe_transformation
                        .map(|(ty, expr)| Transformation::Transform { ty, expr })
                        .unwrap_or_else(|| Transformation::None { ty });

                    source_fields.add(
                        SourceField {
                            name,
                            transformation,
                            // Specifying `backtrace` on a source field is how you request
                            // delegation of the backtrace to the source error type.
                            backtrace_delegate: backtrace_attr.is_some(),
                        },
                        location,
                    );
                } else if let Some((_, location)) = backtrace_attr {
                    backtrace_fields.add(field, location);
                } else {
                    user_fields.push(field);
                }
            }

            let (source, errs) = source_fields.finish_with_location();
            errors.extend(errs);

            let (backtrace, errs) = backtrace_fields.finish_with_location();
            errors.extend(errs);

            match (&source, &backtrace) {
                (Some(source), Some(backtrace)) if source.0.backtrace_delegate => {
                    let source_location = source.1.clone();
                    let backtrace_location = backtrace.1.clone();
                    errors.add(
                        source_location,
                        "Cannot have `backtrace` field and `backtrace` attribute on a source field in the same error variant",
                    );
                    errors.add(
                        backtrace_location,
                        "Cannot have `backtrace` field and `backtrace` attribute on a source field in the same error variant",
                    );
                }
                _ => {} // no conflict
            }

            let (display_format, errs) = display_formats.finish();
            errors.extend(errs);

            let (visibility, errs) = visibilities.finish();
            errors.extend(errs);

            Ok(VariantInfo {
                name,
                source_field: source.map(|(val, _tts)| val),
                backtrace_field: backtrace.map(|(val, _tts)| val),
                user_fields,
                display_format,
                doc_comment,
                visibility,
            })
        })
        .collect();

    errors.finish()?;

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

    let mut transformations = AtMostOne::new("source(from)", "on an error struct");

    let mut errors = SyntaxErrors::default();

    for attr in attributes_from_syn(attrs)? {
        match attr {
            SnafuAttribute::Display(tokens, ..) => {
                errors.add(
                    tokens,
                    OnlyValidOn {
                        attribute: "display",
                        valid_on: "variants of an error enum",
                        not_on: "a struct",
                    },
                );
            }
            SnafuAttribute::Visibility(tokens, ..) => {
                errors.add(
                    tokens,
                    OnlyValidOn {
                        attribute: "visibility",
                        valid_on: "an error enum and its variants",
                        not_on: "a struct",
                    },
                );
            }
            SnafuAttribute::Source(tokens, ss) => {
                for s in ss {
                    match s {
                        Source::Flag(..) => {
                            errors.add(
                                tokens.clone(),
                                OnlyValidOn {
                                    attribute: "source(bool)",
                                    valid_on: "fields of an error variant",
                                    not_on: "a struct",
                                },
                            );
                        }
                        Source::From(t, e) => transformations.add((t, e), tokens.clone()),
                    }
                }
            }
            SnafuAttribute::Backtrace(tokens, ..) => {
                errors.add(
                    tokens,
                    OnlyValidOn {
                        attribute: "backtrace",
                        valid_on: "fields of an error variant",
                        not_on: "a struct",
                    },
                );
            }
            SnafuAttribute::DocComment(..) => { /* Just a regular doc comment. */ }
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

    let (maybe_transformation, errs) = transformations.finish();
    let transformation = maybe_transformation
        .map(|(ty, expr)| Transformation::Transform { ty, expr })
        .unwrap_or_else(|| Transformation::None {
            ty: inner.into_value().ty,
        });
    errors.extend(errs);

    errors.finish()?;

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

// Having a Backtrace newtype and implementing Parse gives us a way to handle careful parsing
// errors outside of `impl Parse for SnafuAttribute`
struct Backtrace(bool);

impl syn::parse::Parse for Backtrace {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::{Ident, LitBool};

        let lookahead = input.lookahead1();

        if lookahead.peek(LitBool) {
            let val: LitBool = input.parse()?;
            Ok(Backtrace(val.value))
        } else if lookahead.peek(Ident) {
            let name: Ident = input.parse()?;

            if name == "delegate" {
                Err(syn::Error::new(
                    name.span(),
                    "`backtrace(delegate)` has been removed; use `backtrace` on a source field",
                ))
            } else {
                Err(syn::Error::new(name.span(), "expected `true` or `false`"))
            }
        } else {
            Err(lookahead.error())
        }
    }
}

/// A SnafuAttribute represents one SNAFU-specific attribute inside of `#[snafu(...)]`.  For
/// example, in `#[snafu(visibility(pub), display("hi"))]`, `visibility(pub)` and `display("hi")`
/// are each a SnafuAttribute.
///
/// We store the location in the source where we found the attribute (as a `TokenStream`) along
/// with the data.  The location can be used to give accurate error messages in case there was a
/// problem with the use of the attribute.
enum SnafuAttribute {
    Display(proc_macro2::TokenStream, UserInput),
    Visibility(proc_macro2::TokenStream, UserInput),
    Source(proc_macro2::TokenStream, Vec<Source>),
    Backtrace(proc_macro2::TokenStream, bool),
    DocComment(proc_macro2::TokenStream, String),
}

impl syn::parse::Parse for SnafuAttribute {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::token::{Comma, Paren};
        use syn::{Expr, Ident, Visibility};

        let input_tts = input.cursor().token_stream();
        let name: Ident = input.parse()?;
        if name == "display" {
            let m: MyMeta<List<Expr>> = input.parse()?;
            let v = m.into_option().ok_or_else(|| {
                syn::Error::new(name.span(), "`snafu(display)` requires an argument")
            })?;
            let v = Box::new(v.0);
            Ok(SnafuAttribute::Display(input_tts, v))
        } else if name == "visibility" {
            let m: MyMeta<Visibility> = input.parse()?;
            let v = m
                .into_option()
                .map_or_else(private_visibility, |v| Box::new(v) as UserInput);
            Ok(SnafuAttribute::Visibility(input_tts, v))
        } else if name == "source" {
            let lookahead = input.lookahead1();
            if input.is_empty() || lookahead.peek(Comma) {
                Ok(SnafuAttribute::Source(input_tts, vec![Source::Flag(true)]))
            } else if lookahead.peek(Paren) {
                let v: MyParens<List<Source>> = input.parse()?;
                Ok(SnafuAttribute::Source(input_tts, v.0.into_vec()))
            } else {
                Err(lookahead.error())
            }
        } else if name == "backtrace" {
            let lookahead = input.lookahead1();
            if input.is_empty() || lookahead.peek(Comma) {
                Ok(SnafuAttribute::Backtrace(input_tts, true))
            } else if lookahead.peek(Paren) {
                let v: MyParens<Backtrace> = input.parse()?;
                let backtrace = v.0;
                Ok(SnafuAttribute::Backtrace(input_tts, backtrace.0))
            } else {
                Err(lookahead.error())
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

        // Remove the parentheses from `#[snafu(...)]` so SnafuAttribute only has to deal with the
        // tokens inside.
        let inside;
        parenthesized!(inside in input);

        let parse_comma_list = Punctuated::<SnafuAttribute, Comma>::parse_terminated;
        let list = parse_comma_list(&inside)?;

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
        let tokens = input.cursor().token_stream();
        let doc: LitStr = input.parse()?;

        Ok(DocComment(SnafuAttribute::DocComment(tokens, doc.value())))
    }
}

fn attributes_from_syn(attrs: Vec<syn::Attribute>) -> MultiSynResult<Vec<SnafuAttribute>> {
    use syn::parse2;

    let mut ours = Vec::new();
    let mut errs = Vec::new();

    let parsed_attrs = attrs.into_iter().flat_map(|attr| {
        if attr.path.is_ident("snafu") {
            Some(parse2::<SnafuAttributeBody>(attr.tokens).map(|body| body.0))
        } else if attr.path.is_ident("doc") {
            // Ignore any errors that occur while parsing the doc
            // comment. This isn't our attribute so we shouldn't
            // assume that we know what values are acceptable.
            parse2::<DocComment>(attr.tokens)
                .ok()
                .map(|comment| Ok(vec![comment.0]))
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

    fn provided_generic_types_without_defaults(&self) -> Vec<proc_macro2::TokenStream> {
        use syn::TypeParam;
        self.generics()
            .type_params()
            .map(|t: &TypeParam| {
                let &TypeParam {
                    ref attrs,
                    ref ident,
                    ref colon_token,
                    ref bounds,
                    ..
                } = t;
                quote! {
                    #(#attrs)*
                    #ident
                    #colon_token
                    #bounds
                }
            })
            .collect()
    }

    fn provided_generics_without_defaults(&self) -> Vec<proc_macro2::TokenStream> {
        self.provided_generic_lifetimes()
            .into_iter()
            .chain(self.provided_generic_types_without_defaults().into_iter())
            .collect()
    }

    fn provided_generic_lifetimes(&self) -> Vec<proc_macro2::TokenStream> {
        use syn::{GenericParam, LifetimeDef};

        self.generics()
            .params
            .iter()
            .flat_map(|p| match *p {
                GenericParam::Lifetime(LifetimeDef { ref lifetime, .. }) => {
                    Some(quote! { #lifetime })
                }
                _ => None,
            })
            .collect()
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
        let original_lifetimes = self.0.provided_generic_lifetimes();
        let original_generic_types_without_defaults =
            self.0.provided_generic_types_without_defaults();
        let original_generics_without_defaults = self.0.provided_generics_without_defaults();

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

        let generics_list = quote! { <#(#original_lifetimes,)* #(#generic_names,)* #(#original_generic_types_without_defaults,)*> };
        let selector_name = quote! { #variant_name<#(#generic_names,)*> };

        let names: &Vec<_> = &user_fields.iter().map(|f| f.name.clone()).collect();

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
                        #( #visibilities #names: #generic_names ),*
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
            quote! {
                impl<#(#generic_names,)*> #selector_name
                {
                    #visibility fn fail<#(#original_generics_without_defaults,)* __T>(self) -> std::result::Result<__T, #parameterized_enum_name>
                    where
                        #(#where_clauses),*
                    {
                        let Self { #(#names),* } = self;
                        let error = #enum_name::#variant_name {
                            #backtrace_field
                            #( #names: std::convert::Into::into(#names) ),*
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
                        ..
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
                    (Some(ref v), _) => quote! { #v },
                    (None, _) if !doc_comment.is_empty() => {
                        quote! { #doc_comment }
                    }
                    (None, Some(ref f)) => {
                        let field_name = &f.name;
                        quote! { concat!(stringify!(#variant_name), ": {}"), #field_name }
                    }
                    (None, None) => quote! { stringify!(#variant_name)},
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
        let original_generics = self.0.provided_generics_without_defaults();
        let parameterized_enum_name = &self.0.parameterized_name();
        let where_clauses = &self.0.provided_where_clauses();

        let variants_to_display = &self.variants_to_display();

        stream.extend({
            quote! {
                impl<#(#original_generics),*> std::fmt::Display for #parameterized_enum_name
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
        let original_generics = self.0.provided_generics_without_defaults();
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

        let source_fn = quote! {
            fn source(&self) -> Option<&(std::error::Error + 'static)> {
                use snafu::AsErrorSource;
                match *self {
                    #(#variants_to_source)*
                }
            }
        };

        stream.extend({
            quote! {
                impl<#(#original_generics),*> std::error::Error for #parameterized_enum_name
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
                ref source_field,
                ref backtrace_field,
                ..
            } = *variant;


            match (source_field, backtrace_field) {
                (Some(ref source_field), _) if source_field.backtrace_delegate => {
                    let SourceField {
                        name: ref field_name,
                        ..
                    } = *source_field;
                    quote! {
                        #enum_name::#variant_name { ref #field_name, .. } => { snafu::ErrorCompat::backtrace(#field_name) }
                    }
                },
                (_, &Some(ref backtrace_field)) => {
                    let Field {
                        name: ref field_name,
                        ..
                    } = *backtrace_field;
                    quote! {
                        #enum_name::#variant_name { ref #field_name, .. } => { std::option::Option::Some(#field_name) }
                    }
                }
                _ => {
                    quote! {
                        #enum_name::#variant_name { .. } => { std::option::Option::None }
                    }
                }
            }
        }).collect()
    }
}

impl<'a> quote::ToTokens for ErrorCompatImpl<'a> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let original_generics = self.0.provided_generics_without_defaults();
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
                impl<#(#original_generics),*> snafu::ErrorCompat for #parameterized_enum_name
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

        let source_fn = quote! {
            fn source(&self) -> Option<&(std::error::Error + 'static)> {
                std::error::Error::source(&self.0)
            }
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
