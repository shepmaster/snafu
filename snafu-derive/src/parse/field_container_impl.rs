use std::fmt;

use crate::{
    parse::{
        self,
        attr::{self, Attribute as _, ErrorForLocation as _, ErrorLocation},
        field_impl::{parse_field, FieldKind},
        AtMostOne, Attribute, CrateRoot, Display, DocCommentBuilder, FlagAttribute as _, Module,
        ProvideExpression, Sidecar, SynErrors, Visibility,
    },
    ContextSelectorKind, ContextSelectorName, DocComment, Field, FieldContainer,
};

const IMPLICIT_MESSAGE_FIELD_NAME: &str = "message";

fn is_implicit_message(name: &proc_macro2::Ident) -> bool {
    name == IMPLICIT_MESSAGE_FIELD_NAME
}

pub struct Attributes {
    display: Option<Display>,
    doc_comment: Option<DocComment>,
    module: Option<Module>,
    provide_expressions: Vec<ProvideExpression>,
    selector_kind: IntermediateSelectorKind,
    visibility: Option<Visibility>,
}

impl Attributes {
    pub(super) fn from_syn(
        attrs: &[syn::Attribute],
        location: ErrorLocation,
        errors: &mut SynErrors,
        mut f: impl FnMut(&mut SynErrors, CrateRoot),
    ) -> Self {
        let mut context_flags = AtMostOne::attribute(attr::ContextFlag, location);
        let mut context_names = AtMostOne::attribute(attr::ContextName, location);
        let mut context_suffixes = AtMostOne::attribute(attr::ContextSuffix, location);
        let mut displays = AtMostOne::attribute(attr::Display, location);
        let mut doc_comment = DocCommentBuilder::default();
        let mut modules = AtMostOne::attribute(attr::Module, location);
        let mut provide_expressions = Vec::new();
        let mut transparents = AtMostOne::attribute(attr::Transparent, location);
        let mut visibilities = AtMostOne::attribute(attr::Visibility, location);
        let mut whatevers = AtMostOne::attribute(attr::Whatever, location);

        parse::syn_attrs(attrs, errors, |errors, attr| {
            use Attribute::*;

            match attr {
                Backtrace(a) => errors.push_invalid_flag(a, location),
                ContextFlag(a) => context_flags.push(a),
                ContextName(a) => context_names.push(a),
                ContextSuffix(a) => context_suffixes.push(a),
                CrateRoot(a) => f(errors, a),
                Display(a) => displays.push(a),
                DocComment(a) => doc_comment.push(&a.str.value()),
                Implicit(a) => errors.push_invalid_flag(a, location),
                Module(a) => modules.push(a),
                ProvideFlag(a) => errors.push_invalid_flag(a, location),
                ProvideExpression(a) => provide_expressions.push(a),
                SourceFlag(a) => errors.push_invalid_flag(a, location),
                SourceFrom(a) => errors.push_invalid(a, location),
                Transparent(a) => transparents.push(a),
                Visibility(a) => visibilities.push(a),
                Whatever(a) => whatevers.push(a),
            }
        });

        let context_flag = context_flags.finish_default(errors);
        let context_name = context_names.finish_default(errors);
        let context_suffix = context_suffixes.finish_default(errors);
        let display = displays.finish_default(errors);
        let doc_comment = doc_comment.finish();
        let module = modules.finish_default(errors);
        let transparent = transparents.finish_default(errors);
        let visibility = visibilities.finish_default(errors);
        let whatever = whatevers.finish_default(errors);

        let transparent = transparent.filter(|t| {
            let enabled = t.is_enabled();

            if !enabled {
                errors.push_new(t, attr::Transparent::FALSE_DOES_NOTHING);
            }

            enabled
        });

        if let (Some(d), Some(t)) = (&display, &transparent) {
            let txt = attr::IncompatibleAttributes {
                attrs: &[attr::Transparent::NAME, attr::Display::NAME],
                bonus: Some("Transparent errors delegate `Display` to their source"),
            }
            .on(location);
            errors.push_new(d, txt);
            errors.push_new(t, txt);
        }

        let selector_kind = match (
            context_flag,
            context_name,
            context_suffix,
            transparent,
            whatever,
        ) {
            (None, None, None, None, None) => IntermediateSelectorKind::DEFAULT,

            (Some(cf), None, None, None, None) => {
                if cf.is_enabled() {
                    IntermediateSelectorKind::DEFAULT
                } else {
                    IntermediateSelectorKind::WithoutContext {
                        source: WithoutContextSource::ContextFlag,
                    }
                }
            }

            (None, Some(cn), None, None, None) => IntermediateSelectorKind::WithContext {
                selector_name: ContextSelectorName::Provided(cn.name),
            },

            (None, None, Some(cs), None, None) => IntermediateSelectorKind::WithContext {
                selector_name: ContextSelectorName::Suffixed(cs.suffix.into_suffix_kind()),
            },

            (None, None, None, Some(_t), None) => IntermediateSelectorKind::WithoutContext {
                source: WithoutContextSource::Transparent,
            },

            (None, None, None, None, Some(_w)) => IntermediateSelectorKind::Whatever,

            (cf, cn, cs, tt, ww) => {
                let maybe_conflicting: &[Option<(&dyn quote::ToTokens, &str)>] = &[
                    cf.as_ref().map(|cf| (cf as _, attr::ContextFlag::NAME)),
                    cn.as_ref().map(|cn| (cn as _, attr::ContextName::NAME)),
                    cs.as_ref().map(|cs| (cs as _, attr::ContextSuffix::NAME)),
                    tt.as_ref().map(|tt| (tt as _, attr::Transparent::NAME)),
                    ww.as_ref().map(|ww| (ww as _, attr::Whatever::NAME)),
                ];

                let conflicting_names = maybe_conflicting
                    .iter()
                    .copied()
                    .flat_map(|a| Some(a?.1))
                    .collect::<Vec<_>>();

                let txt = attr::IncompatibleAttributes::new(&conflicting_names).on(location);
                for conflict in maybe_conflicting {
                    if let &Some((span, _)) = conflict {
                        errors.push_new(span, txt);
                    }
                }

                IntermediateSelectorKind::DEFAULT
            }
        };

        Self {
            display,
            doc_comment,
            module,
            provide_expressions,
            selector_kind,
            visibility,
        }
    }
}

pub(super) fn parse_field_container(
    name: &syn::Ident,
    variant_span: impl quote::ToTokens,
    attrs: Attributes,
    fields: &[&syn::Field],
    inner_location: ErrorLocation,
) -> syn::Result<FieldContainer> {
    let Attributes {
        display,
        doc_comment,
        module,
        provide_expressions,
        selector_kind,
        visibility,
    } = attrs;

    let mut user_fields = Vec::new();
    let mut source_fields = AtMostOne::new(attr::Source::DUPLICATE_FIELD.on(inner_location));
    let mut backtrace_fields = AtMostOne::new(attr::Backtrace::DUPLICATE_FIELD.on(inner_location));
    let mut implicit_fields = Vec::new();

    let mut errors = SynErrors::default();

    for syn_field in fields {
        let field = match parse_field(syn_field) {
            Ok(v) => v,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };

        match field {
            FieldKind::Backtrace(f) => backtrace_fields.push(f),
            FieldKind::Implicit(f) => implicit_fields.push(f),
            FieldKind::Source(f) => source_fields.push(f),
            FieldKind::User(f) => user_fields.push(f),
        }
    }

    let source = source_fields.finish_default(&mut errors);
    let backtrace = backtrace_fields.finish_default(&mut errors);

    match (&source, &backtrace) {
        (Some(Sidecar(source_span, source)), Some(Sidecar(backtrace_span, _backtrace)))
            if source.backtrace_delegate =>
        {
            let txt = "Cannot have `backtrace` field and `backtrace` attribute on a source field in the same variant";
            errors.push_new(source_span, txt);
            errors.push_new(backtrace_span, txt);
        }
        _ => {} // no conflict
    }

    let source_field = source.map(|Sidecar(_, val)| val);
    let backtrace_field = backtrace.map(|Sidecar(_, val)| val);
    let is_transparent = selector_kind.is_transparent();

    let selector_kind = match selector_kind {
        IntermediateSelectorKind::WithContext { selector_name } => ContextSelectorKind::Context {
            selector_name,
            source_field,
            user_fields,
        },

        IntermediateSelectorKind::WithoutContext { source } => {
            for Field { original, .. } in user_fields {
                errors.push_new(
                    original,
                    format_args!("{} must not have context fields", source),
                );
            }

            let source_field = match source_field {
                Some(v) => v,
                None => {
                    errors.push_new(
                        &variant_span,
                        format_args!("{} must have a source field", source),
                    );
                    return errors.assume_failed();
                }
            };

            ContextSelectorKind::NoContext { source_field }
        }

        IntermediateSelectorKind::Whatever => {
            let txt = "Whatever selectors must have exactly one message field";
            let mut message_fields = AtMostOne::new(txt);

            for f in user_fields {
                if is_implicit_message(&f.name) {
                    message_fields.push(f);
                } else {
                    // FUTURE: phrasing?
                    let txt = "Whatever selectors must not have context fields";
                    errors.push_new(f.original, txt);
                }
            }

            let message_field = match message_fields.finish_exactly_one(&variant_span) {
                Ok(m) => m,
                Err(e) => {
                    errors.push(e);
                    return errors.assume_failed();
                }
            };

            ContextSelectorKind::Whatever {
                source_field,
                message_field,
            }
        }
    };

    let display_format = display.map(|d| d.into_display());
    let module = module.map(|m| m.into_value());
    let name = name.clone();
    let provides = provide_expressions
        .into_iter()
        .map(|p| p.into_provide())
        .collect();
    let visibility = visibility.map(|v| v.into_arbitrary());

    errors.finish(FieldContainer {
        backtrace_field,
        display_format,
        doc_comment,
        implicit_fields,
        is_transparent,
        module,
        name,
        provides,
        selector_kind,
        visibility,
    })
}

enum IntermediateSelectorKind {
    WithContext { selector_name: ContextSelectorName },
    WithoutContext { source: WithoutContextSource },
    Whatever,
}

impl IntermediateSelectorKind {
    const DEFAULT: Self = Self::WithContext {
        selector_name: ContextSelectorName::SUFFIX_DEFAULT,
    };

    fn is_transparent(&self) -> bool {
        matches!(
            self,
            Self::WithoutContext {
                source: WithoutContextSource::Transparent
            }
        )
    }
}

enum WithoutContextSource {
    ContextFlag,
    Transparent,
}

impl fmt::Display for WithoutContextSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            WithoutContextSource::ContextFlag => "Context selectors without context",
            WithoutContextSource::Transparent => "`transparent` errors",
        };
        s.fmt(f)
    }
}
