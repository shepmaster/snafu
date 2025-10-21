use crate::{
    parse::{
        self,
        attr::{self, ErrorForLocation as _, ErrorLocation},
        into_transformation, AtMostOne, Attribute, Backtrace, FlagAttribute as _, ProvideFlag,
        Sidecar, SourceFlag, SourceFrom, SynErrors,
    },
    Field, SourceField,
};

const IMPLICIT_SOURCE_FIELD_NAME: &str = "source";
const IMPLICIT_BACKTRACE_FIELD_NAME: &str = "backtrace";

fn is_implicit_source(name: &proc_macro2::Ident) -> bool {
    name == IMPLICIT_SOURCE_FIELD_NAME
}

fn is_implicit_backtrace(name: &proc_macro2::Ident) -> bool {
    name == IMPLICIT_BACKTRACE_FIELD_NAME
}

fn is_implicit_provide(name: &proc_macro2::Ident) -> bool {
    is_implicit_source(name) || is_implicit_backtrace(name)
}

struct Attributes {
    backtrace: Option<Backtrace>,
    implicit: bool,
    provide_flag: Option<ProvideFlag>,
    source_attr_enabled: Option<(bool, SourceOrigin)>,
    source_from: Option<SourceFrom>,
}

impl Attributes {
    fn from_syn(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let location = ErrorLocation::OnField;
        let mut errors = SynErrors::default();

        let mut backtraces = AtMostOne::attribute(attr::Backtrace, location);
        let mut implicits = AtMostOne::attribute(attr::Implicit, location);
        let mut provide_flags = AtMostOne::attribute(attr::ProvideFlag, location);
        let mut source_flags = AtMostOne::attribute(attr::SourceFlag, location);
        let mut source_froms = AtMostOne::attribute(attr::SourceFrom, location);

        parse::syn_attrs(attrs, &mut errors, |errors, attr| {
            use Attribute::*;

            match attr {
                Backtrace(a) => backtraces.push(a),
                ContextFlag(a) => errors.push_invalid_flag(a, location),
                ContextName(a) => errors.push_invalid(a, location),
                ContextSuffix(a) => errors.push_invalid(a, location),
                CrateRoot(a) => errors.push_invalid(a, location),
                Display(a) => errors.push_invalid(a, location),
                DocComment(_a) => { /* no-op */ }
                Implicit(a) => implicits.push(a),
                Module(a) => errors.push_invalid(a, location),
                ProvideFlag(a) => provide_flags.push(a),
                ProvideExpression(a) => errors.push_invalid(a, location),
                SourceFlag(a) => source_flags.push(a),
                SourceFrom(a) => source_froms.push(a),
                Transparent(a) => errors.push_invalid_flag(a, location),
                Visibility(a) => errors.push_invalid(a, location),
                Whatever(a) => errors.push_invalid(a, location),
            }
        });

        let backtrace = backtraces.finish_default(&mut errors);
        let implicit = implicits.finish_default(&mut errors);
        let provide_flag = provide_flags.finish_default(&mut errors);
        let source_flag = source_flags.finish_default(&mut errors);
        let source_from = source_froms.finish_default(&mut errors);

        let implicit = implicit.map(|i| (i.is_enabled(), i));
        let implicit = match implicit {
            // The user didn't specify anything
            None => false,

            // The user is overriding the default
            Some((true, _)) => true,

            // This is redundant but misleading
            Some((false, i)) => {
                errors.push_new(i, attr::Implicit::FALSE_DOES_NOTHING);
                false
            }
        };

        let source_attr_enabled = match (source_flag, &source_from) {
            (None, None) => None,

            (None, Some(source_from)) => Some((true, SourceOrigin::From(source_from.clone()))),

            (Some(source_flag), None) => {
                Some((source_flag.is_enabled(), SourceOrigin::Flag(source_flag)))
            }

            (Some(source_flag), Some(source_from)) => {
                if !source_flag.is_enabled() {
                    let txt = attr::Source::FALSE_AND_FROM_INCOMPATIBLE.on(location);
                    errors.push_new(source_flag, txt);
                    errors.push_new(source_from, txt);
                    None
                } else {
                    Some((true, SourceOrigin::Flag(source_flag)))
                }
            }
        };

        errors.finish(Attributes {
            backtrace,
            implicit,
            provide_flag,
            source_attr_enabled,
            source_from,
        })
    }
}

pub(super) fn parse_field(syn_field: &syn::Field) -> syn::Result<FieldKind> {
    let name = syn_field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(syn_field, "Must have a named field"));

    let attrs = Attributes::from_syn(&syn_field.attrs);

    let (name, attrs) = join_syn_error!(name, attrs)?;

    let Attributes {
        backtrace,
        implicit,
        provide_flag,
        source_attr_enabled,
        source_from,
    } = attrs;

    let mut errors = SynErrors::default();

    let backtrace = backtrace.map(|b| (b.is_enabled(), BacktraceOrigin::Attribute(b)));
    let backtrace = match (backtrace, is_implicit_backtrace(name)) {
        // The user didn't specify anything
        (None, v) => v.then(|| BacktraceOrigin::Field(syn_field.clone())),

        // This is redundant but harmless
        (Some((true, span)), true) => Some(span),

        // The user is overriding the default
        (Some((false, _)), true) => None,
        (Some((true, span)), false) => Some(span),

        // This is redundant but misleading
        (Some((false, span)), false) => {
            errors.push_new(span, attr::Backtrace::FALSE_ON_WRONG_FIELD);
            None
        }
    };

    let source = match (source_attr_enabled, is_implicit_source(name)) {
        // The user didn't specify anything
        (None, v) => v.then(|| SourceOrigin::Field(syn_field.clone())),

        // This is redundant but harmless
        (Some((true, span)), true) => Some(span),

        // The user is overriding the default
        (Some((false, _)), true) => None,
        (Some((true, span)), false) => Some(span),

        // This is redundant but misleading
        (Some((false, span)), false) => {
            errors.push_new(span, attr::Source::FALSE_ON_WRONG_FIELD);
            None
        }
    };

    let provide = provide_flag.map(|p| (p.is_enabled(), p));
    let provide = match (provide, is_implicit_provide(name)) {
        // The user didn't specify anything
        (None, v) => v,

        // This is redundant but harmless
        (Some((true, _)), true) => true,

        // The user is overriding the default
        (Some((false, _)), true) => false,
        (Some((true, _)), false) => true,

        // This is redundant but misleading
        (Some((false, p)), false) => {
            errors.push_new(p, attr::Provide::FALSE_ON_WRONG_FIELD);
            false
        }
    };

    let field = Field {
        name: name.clone(),
        ty: syn_field.ty.clone(),
        provide,
        original: syn_field.clone(),
    };

    let field = if let Some(span) = source {
        let Field {
            name, ty, provide, ..
        } = field;

        let transformation = into_transformation(source_from, ty, false);
        // Specifying `backtrace` on a source field is how you request
        // delegation of the backtrace to the source error type.
        let backtrace_delegate = backtrace.is_some();

        let field = SourceField {
            name,
            transformation,
            backtrace_delegate,
            provide,
        };

        FieldKind::Source(Sidecar(span, field))
    } else if let Some(span) = backtrace {
        FieldKind::Backtrace(Sidecar(span, field))
    } else if implicit {
        FieldKind::Implicit(field)
    } else {
        FieldKind::User(field)
    };

    errors.finish(field)
}

pub(super) enum FieldKind {
    Backtrace(Sidecar<BacktraceOrigin, Field>),
    Implicit(Field),
    Source(Sidecar<SourceOrigin, SourceField>),
    User(Field),
}

pub(super) enum BacktraceOrigin {
    Attribute(Backtrace),

    Field(syn::Field),
}

impl quote::ToTokens for BacktraceOrigin {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            BacktraceOrigin::Attribute(backtrace) => backtrace.to_tokens(tokens),

            BacktraceOrigin::Field(field) => field.to_tokens(tokens),
        }
    }
}

pub(super) enum SourceOrigin {
    Field(syn::Field),

    Flag(SourceFlag),

    From(SourceFrom),
}

impl quote::ToTokens for SourceOrigin {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            SourceOrigin::Field(field) => field.to_tokens(tokens),

            SourceOrigin::Flag(flat_source_flag) => flat_source_flag.to_tokens(tokens),

            SourceOrigin::From(flat_source_from) => flat_source_from.to_tokens(tokens),
        }
    }
}
