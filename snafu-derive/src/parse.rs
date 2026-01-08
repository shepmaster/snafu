use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::{collections::BTreeSet, fmt, mem};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token, Expr, Ident, Lit, LitBool, LitStr, Path, Type,
};

use crate::{ModuleName, SuffixKind, Transformation, UserInput};

macro_rules! join_syn_error {
    ($r1:expr, $r2:expr) => {
        match ($r1, $r2) {
            (Err(mut e1), Err(e2)) => {
                e1.combine(e2);
                Err(e1)
            }
            (Err(e), _) | (_, Err(e)) => Err(e),
            (Ok(v1), Ok(v2)) => Ok((v1, v2)),
        }
    };
}

mod attr;
mod enum_impl;
mod field_container_impl;
mod field_impl;
mod named_struct_impl;
mod tuple_struct_field_impl;
mod tuple_struct_impl;
mod variant_impl;

pub(crate) use enum_impl::parse_enum;
pub(crate) use named_struct_impl::parse_named_struct;
pub(crate) use tuple_struct_impl::parse_tuple_struct;

use attr::{ErrorForLocation as _, ErrorLocation};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(backtrace);
    custom_keyword!(context);
    custom_keyword!(crate_root);
    custom_keyword!(display);
    custom_keyword!(implicit);
    custom_keyword!(module);
    custom_keyword!(provide);
    custom_keyword!(source);
    custom_keyword!(transparent);
    custom_keyword!(visibility);
    custom_keyword!(whatever);

    custom_keyword!(from);
    custom_keyword!(exact);
    custom_keyword!(generic);

    custom_keyword!(name);
    custom_keyword!(suffix);

    custom_keyword!(opt);
}

#[derive(Default)]
struct SynErrors(Option<syn::Error>);

impl SynErrors {
    fn push(&mut self, e: syn::Error) {
        if let Some(prev_e) = &mut self.0 {
            prev_e.combine(e);
        } else {
            self.0 = Some(e);
        }
    }

    fn push_new(&mut self, span: impl quote::ToTokens, txt: impl fmt::Display) {
        let e = syn::Error::new_spanned(span, txt);
        self.push(e);
    }

    fn push_invalid<A>(&mut self, attr: A, location: ErrorLocation)
    where
        A: AttributeMeta,
    {
        self.push_new(attr, <A::Meta as attr::Attribute>::INVALID.on(location));
    }

    fn push_invalid_flag<A>(&mut self, attr: A, location: ErrorLocation)
    where
        A: FlagAttribute,
        A::Meta: attr::FlagAttribute,
    {
        let i = if attr.has_arg() {
            <A::Meta as attr::Attribute>::INVALID
        } else {
            <A::Meta as attr::FlagAttribute>::BASE_INVALID
        };

        self.push_new(attr, i.on(location));
    }

    fn finish<T>(self, value: T) -> syn::Result<T> {
        match self.0 {
            Some(e) => Err(e),
            None => Ok(value),
        }
    }

    // FUTURE: It'd be nice if we could avoid this by knowing that we
    // just pushed an error...
    fn assume_failed<T>(self) -> syn::Result<T> {
        match self.0 {
            Some(e) => Err(e),
            None => unreachable!("No error recorded"),
        }
    }
}

#[derive(Default)]
struct DocCommentBuilder {
    reached_end_of_doc_comment: bool,
    content: String,
}

impl DocCommentBuilder {
    fn push(&mut self, line: &str) {
        // We join all the doc comment attributes with a space,
        // but end once the summary of the doc comment is
        // complete, which is indicated by an empty line.
        if self.reached_end_of_doc_comment {
            return;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            self.reached_end_of_doc_comment = true;
            return;
        }

        if !self.content.is_empty() {
            self.content.push(' ');
        }

        self.content.push_str(trimmed);
    }

    fn finish(self) -> Option<crate::DocComment> {
        let Self { content, .. } = self;
        if content.is_empty() {
            None
        } else {
            let shorthand_names = extract_field_names(&content)
                .map(|n| quote::format_ident!("{}", n))
                .collect();

            Some(crate::DocComment {
                content,
                shorthand_names,
            })
        }
    }
}

enum Attribute {
    Backtrace(Backtrace),
    ContextFlag(ContextFlag),
    ContextName(ContextName),
    ContextSuffix(ContextSuffix),
    CrateRoot(CrateRoot),
    Display(Display),
    DocComment(DocComment),
    Implicit(Implicit),
    Module(Module),
    ProvideFlag(ProvideFlag),
    ProvideExpression(ProvideExpression),
    SourceFlag(SourceFlag),
    SourceFrom(SourceFrom),
    Transparent(Transparent),
    Visibility(Visibility),
    Whatever(Whatever),
}

fn syn_attrs(
    attrs: &[syn::Attribute],
    errors: &mut SynErrors,
    mut f: impl FnMut(&mut SynErrors, Attribute),
) {
    for attr in attrs {
        if attr.path().is_ident("snafu") {
            let attr_list = <Punctuated<NestedAttribute, token::Comma>>::parse_terminated;
            let a = match attr.parse_args_with(attr_list) {
                Ok(a) => a,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            };

            let mut f = |a| f(errors, a);

            for pair in a.into_pairs() {
                match pair.into_value() {
                    NestedAttribute::Backtrace(a) => f(Attribute::Backtrace(a)),
                    NestedAttribute::Context(a) => match a {
                        Context::Flag(a) => f(Attribute::ContextFlag(a)),
                        Context::Name(a) => f(Attribute::ContextName(a)),
                        Context::Suffix(a) => f(Attribute::ContextSuffix(a)),
                    },
                    NestedAttribute::CrateRoot(a) => f(Attribute::CrateRoot(a)),
                    NestedAttribute::Display(a) => f(Attribute::Display(a)),
                    NestedAttribute::Implicit(a) => f(Attribute::Implicit(a)),
                    NestedAttribute::Module(a) => f(Attribute::Module(a)),
                    NestedAttribute::Provide(a) => match a {
                        Provide::Flag(a) => f(Attribute::ProvideFlag(a)),
                        Provide::Expression(a) => f(Attribute::ProvideExpression(a)),
                    },
                    NestedAttribute::Source(a) => a.flatten(|a| match a {
                        Source::Flag(a) => f(Attribute::SourceFlag(a)),
                        Source::From(a) => f(Attribute::SourceFrom(a)),
                    }),
                    NestedAttribute::Transparent(a) => f(Attribute::Transparent(a)),
                    NestedAttribute::Visibility(a) => f(Attribute::Visibility(a)),
                    NestedAttribute::Whatever(a) => f(Attribute::Whatever(a)),
                }
            }
        } else if attr.path().is_ident("doc") {
            // Ignore any errors that occur while parsing the doc
            // comment. This isn't our attribute so we shouldn't
            // assume that we know what values are acceptable.
            if let Ok(comment) = syn::parse2::<DocComment>(attr.meta.to_token_stream()) {
                f(errors, Attribute::DocComment(comment));
            }
        }
    }
}

struct AtMostOne<T, D> {
    inner: AtMostOneInner<T>,
    message: D,
}

impl<T, D> AtMostOne<T, D>
where
    D: fmt::Display,
{
    fn new(message: D) -> Self {
        Self {
            inner: Default::default(),
            message,
        }
    }

    fn push(&mut self, value: T)
    where
        T: quote::ToTokens,
    {
        let inner = mem::take(&mut self.inner);
        self.inner = match inner {
            AtMostOneInner::Empty => AtMostOneInner::One(value),

            AtMostOneInner::One(_old_value) => {
                // FUTURE: consider reporting the *original* location as well
                let new_error = syn::Error::new_spanned(value, &self.message);
                AtMostOneInner::Err(new_error)
            }

            AtMostOneInner::Err(mut error) => {
                let new_error = syn::Error::new_spanned(value, &self.message);
                error.combine(new_error);
                AtMostOneInner::Err(error)
            }
        };
    }

    fn finish(self) -> syn::Result<Option<T>> {
        match self.inner {
            AtMostOneInner::Empty => Ok(None),
            AtMostOneInner::One(value) => Ok(Some(value)),
            AtMostOneInner::Err(error) => Err(error),
        }
    }

    /// When an error occurs, it's added to the `SynErrors`
    /// and `None` is returned.
    fn finish_default(self, errors: &mut SynErrors) -> Option<T> {
        match self.finish() {
            Ok(v) => v,
            Err(e) => {
                errors.push(e);
                None
            }
        }
    }

    fn finish_exactly_one(self, span: impl quote::ToTokens) -> syn::Result<T> {
        match self.inner {
            AtMostOneInner::Empty => Err(syn::Error::new_spanned(span, &self.message)),

            AtMostOneInner::One(value) => Ok(value),

            AtMostOneInner::Err(error) => Err(error),
        }
    }
}

impl<T> AtMostOne<T, attr::ErrorWithLocation<attr::DuplicateAttribute>> {
    fn attribute<A>(_attr: A, location: ErrorLocation) -> Self
    where
        A: attr::Attribute,
    {
        Self::new(A::DUPLICATE.on(location))
    }
}

impl<T, D> Extend<T> for AtMostOne<T, D>
where
    T: quote::ToTokens,
    D: fmt::Display,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for value in iter {
            self.push(value);
        }
    }
}

enum AtMostOneInner<T> {
    Empty,
    One(T),
    Err(syn::Error),
}

impl<T> Default for AtMostOneInner<T> {
    fn default() -> Self {
        AtMostOneInner::Empty
    }
}

/// Allows attaching span information to an arbitrary piece of data,
/// enabling better error reporting.
struct Sidecar<S, T>(S, T);

impl<S, T> quote::ToTokens for Sidecar<S, T>
where
    S: quote::ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}

enum NestedAttribute {
    Backtrace(Backtrace),
    Context(Context),
    CrateRoot(CrateRoot),
    Display(Display),
    Implicit(Implicit),
    Module(Module),
    Provide(Provide),
    Source(NestedSource),
    Transparent(Transparent),
    Visibility(Visibility),
    Whatever(Whatever),
}

impl Parse for NestedAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::backtrace) {
            input.parse().map(NestedAttribute::Backtrace)
        } else if lookahead.peek(kw::context) {
            input.parse().map(NestedAttribute::Context)
        } else if lookahead.peek(kw::crate_root) {
            input.parse().map(NestedAttribute::CrateRoot)
        } else if lookahead.peek(kw::display) {
            input.parse().map(NestedAttribute::Display)
        } else if lookahead.peek(kw::implicit) {
            input.parse().map(NestedAttribute::Implicit)
        } else if lookahead.peek(kw::module) {
            input.parse().map(NestedAttribute::Module)
        } else if lookahead.peek(kw::provide) {
            input.parse().map(NestedAttribute::Provide)
        } else if lookahead.peek(kw::source) {
            input.parse().map(NestedAttribute::Source)
        } else if lookahead.peek(kw::transparent) {
            input.parse().map(NestedAttribute::Transparent)
        } else if lookahead.peek(kw::visibility) {
            input.parse().map(NestedAttribute::Visibility)
        } else if lookahead.peek(kw::whatever) {
            input.parse().map(NestedAttribute::Whatever)
        } else {
            Err(lookahead.error())
        }
    }
}

struct Backtrace {
    backtrace_token: kw::backtrace,
    arg: MaybeArg<LitBool>,
}

impl Parse for Backtrace {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            backtrace_token: input.parse()?,
            arg: input.parse()?,
        })
    }
}

impl ToTokens for Backtrace {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.backtrace_token.to_tokens(tokens);
        self.arg.to_tokens(tokens);
    }
}

enum Context {
    Flag(ContextFlag),
    Name(ContextName),
    Suffix(ContextSuffix),
}

impl Parse for Context {
    fn parse(input: ParseStream) -> Result<Self> {
        let context_token = input.parse()?;
        let arg = input.parse::<MaybeArg<ContextArg>>()?;

        Ok(match arg {
            MaybeArg::None => Context::Flag(ContextFlag {
                context_token,
                arg: MaybeArg::None,
            }),

            MaybeArg::Some {
                paren_token,
                content,
            } => match content {
                ContextArg::Flag { value } => Context::Flag(ContextFlag {
                    context_token,
                    arg: MaybeArg::Some {
                        paren_token,
                        content: value,
                    },
                }),

                ContextArg::Name {
                    name_token,
                    paren_token: inner_paren_token,
                    name,
                } => Context::Name(ContextName {
                    context_token,
                    paren_token,
                    name_token,
                    inner_paren_token,
                    name,
                }),

                ContextArg::Suffix {
                    suffix_token,
                    paren_token: inner_paren_token,
                    suffix,
                } => Context::Suffix(ContextSuffix {
                    context_token,
                    paren_token,
                    suffix_token,
                    inner_paren_token,
                    suffix,
                }),
            },
        })
    }
}

struct ContextFlag {
    context_token: kw::context,
    arg: MaybeArg<LitBool>,
}

impl ToTokens for ContextFlag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.context_token.to_tokens(tokens);
        self.arg.to_tokens(tokens);
    }
}

struct ContextName {
    context_token: kw::context,
    paren_token: token::Paren,
    name_token: kw::name,
    inner_paren_token: token::Paren,
    name: Ident,
}

impl ToTokens for ContextName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.context_token.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.name_token.to_tokens(tokens);
            self.inner_paren_token.surround(tokens, |tokens| {
                self.name.to_tokens(tokens);
            });
        });
    }
}

struct ContextSuffix {
    context_token: kw::context,
    paren_token: token::Paren,
    suffix_token: kw::suffix,
    inner_paren_token: token::Paren,
    suffix: SuffixArg,
}

impl ToTokens for ContextSuffix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.context_token.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.suffix_token.to_tokens(tokens);
            self.inner_paren_token.surround(tokens, |tokens| {
                self.suffix.to_tokens(tokens);
            });
        });
    }
}

enum ContextArg {
    Flag {
        value: LitBool,
    },
    Name {
        name_token: kw::name,
        paren_token: token::Paren,
        name: Ident,
    },
    Suffix {
        suffix_token: kw::suffix,
        paren_token: token::Paren,
        suffix: SuffixArg,
    },
}

impl Parse for ContextArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitBool) {
            Ok(ContextArg::Flag {
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::suffix) {
            let content;
            Ok(ContextArg::Suffix {
                suffix_token: input.parse()?,
                paren_token: parenthesized!(content in input),
                suffix: content.parse()?,
            })
        } else if lookahead.peek(kw::name) {
            let content;
            Ok(ContextArg::Name {
                name_token: input.parse()?,
                paren_token: parenthesized!(content in input),
                name: content.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

enum SuffixArg {
    Flag { value: LitBool },
    Suffix { suffix: Ident },
}

impl SuffixArg {
    fn into_suffix_kind(self) -> SuffixKind {
        match self {
            SuffixArg::Flag { value } => {
                if value.value {
                    SuffixKind::Default
                } else {
                    SuffixKind::None
                }
            }
            SuffixArg::Suffix { suffix } => SuffixKind::Some(suffix),
        }
    }
}

impl Parse for SuffixArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitBool) {
            Ok(SuffixArg::Flag {
                value: input.parse()?,
            })
        } else if lookahead.peek(Ident) {
            Ok(SuffixArg::Suffix {
                suffix: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for SuffixArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SuffixArg::Flag { value } => {
                value.to_tokens(tokens);
            }
            SuffixArg::Suffix { suffix } => {
                suffix.to_tokens(tokens);
            }
        }
    }
}

struct CrateRoot {
    crate_root_token: kw::crate_root,
    paren_token: token::Paren,
    arg: Path,
}

fn into_crate_root(crate_root: Option<CrateRoot>) -> UserInput {
    match crate_root {
        Some(cr) => Box::new(cr.arg),
        None => Box::new(quote! { ::snafu }),
    }
}

impl Parse for CrateRoot {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            crate_root_token: input.parse()?,
            paren_token: parenthesized!(content in input),
            arg: content.parse()?,
        })
    }
}

impl ToTokens for CrateRoot {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.crate_root_token.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.arg.to_tokens(tokens);
        });
    }
}

struct Display {
    display_token: kw::display,
    paren_token: token::Paren,
    args: Punctuated<Expr, token::Comma>,
}

impl Display {
    fn into_display(self) -> crate::Display {
        let exprs: Vec<_> = self.args.into_iter().collect();
        let mut shorthand_names = BTreeSet::new();
        let mut assigned_names = BTreeSet::new();

        // Do a best-effort parsing here; if we fail, the compiler
        // will likely spit out something more useful when it tries to
        // parse it.
        if let Some((Expr::Lit(l), args)) = exprs.split_first() {
            if let Lit::Str(s) = &l.lit {
                let format_str = s.value();
                let names = extract_field_names(&format_str).map(|n| format_ident!("{}", n));
                shorthand_names.extend(names);
            }

            for arg in args {
                if let Expr::Assign(a) = arg {
                    if let Expr::Path(p) = &*a.left {
                        assigned_names.extend(p.path.get_ident().cloned());
                    }
                }
            }
        }

        crate::Display {
            exprs,
            shorthand_names,
            assigned_names,
        }
    }
}

pub(crate) fn extract_field_names(mut s: &str) -> impl Iterator<Item = &str> {
    std::iter::from_fn(move || loop {
        let open_curly = s.find('{')?;
        s = &s[open_curly + '{'.len_utf8()..];

        if s.starts_with('{') {
            s = &s['{'.len_utf8()..];
            continue;
        }

        let end_curly = s.find('}')?;
        let format_contents = &s[..end_curly];

        let name = match format_contents.find(':') {
            Some(idx) => &format_contents[..idx],
            None => format_contents,
        };

        if name.is_empty() {
            continue;
        }

        return Some(name);
    })
}

impl Parse for Display {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            display_token: input.parse()?,
            paren_token: parenthesized!(content in input),
            args: Punctuated::parse_terminated(&content)?,
        })
    }
}

impl ToTokens for Display {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.display_token.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.args.to_tokens(tokens);
        });
    }
}

struct DocComment {
    _doc_ident: Ident,
    _eq_token: token::Eq,
    str: LitStr,
}

impl Parse for DocComment {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            _doc_ident: input.parse()?,
            _eq_token: input.parse()?,
            str: input.parse()?,
        })
    }
}

struct Implicit {
    implicit_token: kw::implicit,
    arg: MaybeArg<LitBool>,
}

impl Parse for Implicit {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            implicit_token: input.parse()?,
            arg: input.parse()?,
        })
    }
}

impl ToTokens for Implicit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.implicit_token.to_tokens(tokens);
        self.arg.to_tokens(tokens);
    }
}

struct Module {
    module_token: kw::module,
    arg: MaybeArg<Ident>,
}

impl Module {
    fn into_value(self) -> ModuleName {
        match self.arg.into_option() {
            None => ModuleName::Default,
            Some(name) => ModuleName::Custom(name),
        }
    }
}

impl Parse for Module {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            module_token: input.parse()?,
            arg: input.parse()?,
        })
    }
}

impl ToTokens for Module {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.module_token.to_tokens(tokens);
        self.arg.to_tokens(tokens);
    }
}

enum Provide {
    Flag(ProvideFlag),

    Expression(ProvideExpression),
}

impl Parse for Provide {
    fn parse(input: ParseStream) -> Result<Self> {
        let provide_token = input.parse()?;
        let arg = input.parse::<MaybeArg<ProvideArg>>()?;

        Ok(match arg {
            MaybeArg::None => Provide::Flag(ProvideFlag {
                provide_token,
                value: MaybeArg::None,
            }),
            MaybeArg::Some {
                paren_token,
                content,
            } => match content {
                ProvideArg::Flag { value } => Provide::Flag(ProvideFlag {
                    provide_token,
                    value: MaybeArg::Some {
                        paren_token,
                        content: value,
                    },
                }),
                ProvideArg::Expression {
                    flags,
                    ty,
                    arrow,
                    expr,
                } => {
                    let p = ProvideExpression {
                        provide_token,
                        paren_token,
                        flags,
                        ty,
                        arrow,
                        expr,
                    };

                    Provide::Expression(p)
                }
            },
        })
    }
}

struct ProvideFlag {
    provide_token: kw::provide,
    value: MaybeArg<LitBool>,
}

impl ToTokens for ProvideFlag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.provide_token.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

struct ProvideExpression {
    provide_token: kw::provide,
    paren_token: token::Paren,
    flags: ProvideFlags,
    ty: Type,
    arrow: token::FatArrow,
    expr: Expr,
}

impl ProvideExpression {
    fn into_provide(self) -> crate::Provide {
        crate::Provide {
            is_opt: self.flags.is_opt(),
            is_ref: self.flags.is_ref(),
            ty: self.ty,
            expr: self.expr,
        }
    }
}

impl ToTokens for ProvideExpression {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.provide_token.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.flags.to_tokens(tokens);
            self.ty.to_tokens(tokens);
            self.arrow.to_tokens(tokens);
            self.expr.to_tokens(tokens);
        });
    }
}

enum ProvideArg {
    Flag {
        value: LitBool,
    },
    Expression {
        flags: ProvideFlags,
        ty: Type,
        arrow: token::FatArrow,
        expr: Expr,
    },
}

impl Parse for ProvideArg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitBool) {
            Ok(ProvideArg::Flag {
                value: input.parse()?,
            })
        } else {
            Ok(ProvideArg::Expression {
                flags: input.parse()?,
                ty: input.parse()?,
                arrow: input.parse()?,
                expr: input.parse()?,
            })
        }
    }
}

struct ProvideFlags(Punctuated<ProvideFlagInner, token::Comma>);

impl ProvideFlags {
    fn is_opt(&self) -> bool {
        self.0.iter().any(ProvideFlagInner::is_opt)
    }

    fn is_ref(&self) -> bool {
        self.0.iter().any(ProvideFlagInner::is_ref)
    }
}

impl Parse for ProvideFlags {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut flags = Punctuated::new();

        while ProvideFlagInner::peek(input) {
            flags.push_value(input.parse()?);
            flags.push_punct(input.parse()?);
        }

        Ok(Self(flags))
    }
}

impl ToTokens for ProvideFlags {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

enum ProvideFlagInner {
    Opt(kw::opt),
    Ref(token::Ref),
}

impl ProvideFlagInner {
    fn peek(input: ParseStream) -> bool {
        input.peek(kw::opt) || input.peek(token::Ref)
    }

    fn is_opt(&self) -> bool {
        matches!(self, ProvideFlagInner::Opt(_))
    }

    fn is_ref(&self) -> bool {
        matches!(self, ProvideFlagInner::Ref(_))
    }
}

impl Parse for ProvideFlagInner {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::opt) {
            input.parse().map(ProvideFlagInner::Opt)
        } else if lookahead.peek(token::Ref) {
            input.parse().map(ProvideFlagInner::Ref)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for ProvideFlagInner {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ProvideFlagInner::Opt(v) => v.to_tokens(tokens),
            ProvideFlagInner::Ref(v) => v.to_tokens(tokens),
        }
    }
}

enum Source {
    Flag(SourceFlag),

    From(SourceFrom),
}

struct SourceFlag {
    source_token: kw::source,
    value: MaybeArg<LitBool>,
}

impl ToTokens for SourceFlag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.source_token.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

#[derive(Clone)]
struct SourceFrom {
    source_token: kw::source,
    paren_token: token::Paren,
    value: SourceFromArg,
}

fn into_transformation(source_from: Option<SourceFrom>, target_ty: Type) -> Transformation {
    match source_from {
        Some(SourceFrom { value, .. }) => match value.value {
            SourceFromValue::Exact(_) => Transformation::None {
                target_ty,
                from_is_generic: false,
            },

            SourceFromValue::Generic(_) => Transformation::None {
                target_ty,
                from_is_generic: true,
            },

            SourceFromValue::Transform(SourceFromTransform { r#type, expr, .. }) => {
                Transformation::Transform {
                    source_ty: r#type,
                    target_ty,
                    expr,
                }
            }
        },

        None => Transformation::None {
            target_ty,
            from_is_generic: false,
        },
    }
}

impl ToTokens for SourceFrom {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.source_token.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.value.to_tokens(tokens);
        });
    }
}

struct NestedSource {
    source_token: kw::source,
    args: MaybeArg<Punctuated<SourceArg, token::Comma>>,
}

impl NestedSource {
    fn flatten(self, mut f: impl FnMut(Source)) {
        let source_token = self.source_token;

        match self.args {
            MaybeArg::None => f(Source::Flag(SourceFlag {
                source_token,
                value: MaybeArg::None,
            })),

            MaybeArg::Some {
                paren_token,
                content,
            } => {
                for sa in content {
                    let s = match sa {
                        SourceArg::Flag { value } => Source::Flag(SourceFlag {
                            source_token,
                            value: MaybeArg::Some {
                                paren_token,
                                content: value,
                            },
                        }),
                        SourceArg::From(value) => Source::From(SourceFrom {
                            source_token,
                            paren_token,
                            value,
                        }),
                    };
                    f(s);
                }
            }
        }
    }
}

impl Parse for NestedSource {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            source_token: input.parse()?,
            args: MaybeArg::parse_with(input, Punctuated::parse_terminated)?,
        })
    }
}

enum SourceArg {
    Flag { value: LitBool },
    From(SourceFromArg),
}

impl Parse for SourceArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitBool) {
            Ok(SourceArg::Flag {
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::from) {
            input.parse().map(SourceArg::From)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Clone)]
struct SourceFromArg {
    from_token: kw::from,
    paren_token: token::Paren,
    value: SourceFromValue,
}

impl Parse for SourceFromArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(SourceFromArg {
            from_token: input.parse()?,
            paren_token: parenthesized!(content in input),
            value: content.parse()?,
        })
    }
}

impl ToTokens for SourceFromArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.from_token.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.value.to_tokens(tokens);
        });
    }
}

#[derive(Clone)]
enum SourceFromValue {
    Exact(kw::exact),

    Generic(kw::generic),

    Transform(SourceFromTransform),
}

impl Parse for SourceFromValue {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::exact) {
            input.parse().map(Self::Exact)
        } else if input.peek(kw::generic) {
            input.parse().map(Self::Generic)
        } else {
            // We can't peek ahead for a type. If we fail, add our own
            // error that mimics the lookahead error to tell the user
            // that `exact` / `generic` are also possible here.
            //
            // FUTURE: Consider making transforms be keyword-prefixed (with a semver
            // break?) e.g. `transform Type with Expr`
            let span = input.span();
            let txt = "expected one of: `exact`, `generic` or a type followed by a comma and an expression";
            input.parse().map(Self::Transform).map_err(|e| {
                let mut e1 = syn::Error::new(span, txt);
                e1.combine(e);
                e1
            })
        }
    }
}

impl ToTokens for SourceFromValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SourceFromValue::Exact(exact) => exact.to_tokens(tokens),
            SourceFromValue::Generic(generic) => generic.to_tokens(tokens),
            SourceFromValue::Transform(transform) => transform.to_tokens(tokens),
        }
    }
}

#[derive(Clone)]
struct SourceFromTransform {
    r#type: Type,
    comma_token: token::Comma,
    expr: Expr,
}

impl Parse for SourceFromTransform {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            r#type: input.parse()?,
            comma_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for SourceFromTransform {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.r#type.to_tokens(tokens);
        self.comma_token.to_tokens(tokens);
        self.expr.to_tokens(tokens);
    }
}

struct Transparent {
    transparent_token: kw::transparent,
    arg: MaybeArg<LitBool>,
}

impl Parse for Transparent {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            transparent_token: input.parse()?,
            arg: input.parse()?,
        })
    }
}

impl ToTokens for Transparent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.transparent_token.to_tokens(tokens);
        self.arg.to_tokens(tokens);
    }
}

struct Visibility {
    visibility_token: kw::visibility,
    visibility: MaybeArg<syn::Visibility>,
}

impl Visibility {
    // TODO: Remove boxed trait object
    fn into_arbitrary(self) -> Box<dyn ToTokens> {
        // TODO: Move this default value out of parsing
        self.visibility
            .into_option()
            .map_or_else(super::private_visibility, |v| Box::new(v))
    }
}

impl Parse for Visibility {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            visibility_token: input.parse()?,
            visibility: input.parse()?,
        })
    }
}

impl ToTokens for Visibility {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.visibility_token.to_tokens(tokens);
        self.visibility.to_tokens(tokens);
    }
}

struct Whatever {
    whatever_token: kw::whatever,
}

impl Parse for Whatever {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            whatever_token: input.parse()?,
        })
    }
}

impl ToTokens for Whatever {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.whatever_token.to_tokens(tokens);
    }
}

enum MaybeArg<T> {
    None,
    Some {
        paren_token: token::Paren,
        content: T,
    },
}

impl<T> MaybeArg<T> {
    fn to_option(&self) -> Option<&T> {
        match self {
            MaybeArg::None => None,
            MaybeArg::Some { content, .. } => Some(content),
        }
    }

    fn into_option(self) -> Option<T> {
        match self {
            MaybeArg::None => None,
            MaybeArg::Some { content, .. } => Some(content),
        }
    }

    fn parse_with<F>(input: ParseStream<'_>, parser: F) -> Result<Self>
    where
        F: FnOnce(ParseStream<'_>) -> Result<T>,
    {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Paren) {
            let content;
            Ok(MaybeArg::Some {
                paren_token: parenthesized!(content in input),
                content: parser(&content)?,
            })
        } else {
            Ok(MaybeArg::None)
        }
    }
}

impl<T: Parse> Parse for MaybeArg<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_with(input, Parse::parse)
    }
}

impl<T: ToTokens> ToTokens for MaybeArg<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let MaybeArg::Some {
            paren_token,
            content,
        } = self
        {
            paren_token.surround(tokens, |tokens| {
                content.to_tokens(tokens);
            });
        }
    }
}

trait AttributeMeta: quote::ToTokens {
    type Meta: attr::Attribute;
}

trait FlagAttribute: AttributeMeta
where
    Self::Meta: attr::FlagAttribute,
{
    fn arg(&self) -> &MaybeArg<LitBool>;

    fn has_arg(&self) -> bool {
        matches!(self.arg(), MaybeArg::Some { .. })
    }

    fn is_enabled(&self) -> bool {
        self.arg().to_option().map_or(true, |v| v.value)
    }
}

macro_rules! def_attributes {
    ($($name:ident),* $(,)?) => {
        $(
            impl AttributeMeta for $name {
                type Meta = attr::$name;
            }
        )*
    };
}

def_attributes![
    Backtrace,
    ContextFlag,
    ContextName,
    ContextSuffix,
    CrateRoot,
    Display,
    Implicit,
    Module,
    ProvideExpression,
    ProvideFlag,
    SourceFlag,
    SourceFrom,
    Transparent,
    Visibility,
    Whatever,
];

macro_rules! def_flag_attributes {
    ($(($name:ident, $arg:ident)),* $(,)?) => {
        $(
            impl FlagAttribute for $name {
                fn arg(&self) -> &MaybeArg<LitBool> {
                    &self.$arg
                }
            }
        )*
    };
}

def_flag_attributes![
    (Backtrace, arg),
    (ContextFlag, arg),
    (Implicit, arg),
    (ProvideFlag, value),
    (SourceFlag, value),
    (Transparent, arg),
];

#[cfg(test)]
mod test {
    use super::*;

    fn names(s: &str) -> Vec<&str> {
        extract_field_names(s).collect::<Vec<_>>()
    }

    #[test]
    fn ignores_positional_arguments() {
        assert_eq!(names("{}"), [] as [&str; 0]);
    }

    #[test]
    fn finds_named_argument() {
        assert_eq!(names("{a}"), ["a"]);
    }

    #[test]
    fn finds_multiple_named_arguments() {
        assert_eq!(names("{a} {b}"), ["a", "b"]);
    }

    #[test]
    fn ignores_escaped_braces() {
        assert_eq!(names("{{a}}"), [] as [&str; 0]);
    }

    #[test]
    fn finds_named_arguments_around_escaped() {
        assert_eq!(names("{a} {{b}} {c}"), ["a", "c"]);
    }

    #[test]
    fn ignores_format_spec() {
        assert_eq!(names("{a:?}"), ["a"]);
    }
}
