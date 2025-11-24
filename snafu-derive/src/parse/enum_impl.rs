use crate::{
    parse::{
        self,
        attr::{self, ErrorLocation},
        into_crate_root, AtMostOne, Attribute, ContextSuffix, CrateRoot, Module, SynErrors,
        Visibility,
    },
    EnumInfo,
};

struct Attributes {
    context_suffix: Option<ContextSuffix>,
    crate_root: Option<CrateRoot>,
    module: Option<Module>,
    visibility: Option<Visibility>,
}

impl Attributes {
    fn from_syn(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let location = ErrorLocation::OnEnum;
        let mut errors = SynErrors::default();

        let mut context_suffixes = AtMostOne::attribute(attr::ContextSuffix, location);
        let mut crate_roots = AtMostOne::attribute(attr::CrateRoot, location);
        let mut modules = AtMostOne::attribute(attr::Module, location);
        let mut visibilities = AtMostOne::attribute(attr::Visibility, location);

        parse::syn_attrs(attrs, &mut errors, |errors, attr| {
            use Attribute::*;

            match attr {
                Backtrace(a) => errors.push_invalid_flag(a, location),
                ContextFlag(a) => errors.push_invalid_flag(a, location),
                ContextName(a) => errors.push_invalid(a, location),
                ContextSuffix(a) => context_suffixes.push(a),
                CrateRoot(a) => crate_roots.push(a),
                Display(a) => errors.push_invalid(a, location),
                DocComment(_a) => { /* no-op */ }
                Implicit(a) => errors.push_invalid_flag(a, location),
                Module(a) => modules.push(a),
                ProvideFlag(a) => errors.push_invalid_flag(a, location),
                ProvideExpression(a) => errors.push_invalid(a, location),
                SourceFlag(a) => errors.push_invalid_flag(a, location),
                SourceFrom(a) => errors.push_invalid(a, location),
                Transparent(a) => errors.push_invalid_flag(a, location),
                Visibility(a) => visibilities.push(a),
                Whatever(a) => errors.push_invalid(a, location),
            }
        });

        let context_suffix = context_suffixes.finish_default(&mut errors);
        let crate_root = crate_roots.finish_default(&mut errors);
        let module = modules.finish_default(&mut errors);
        let visibility = visibilities.finish_default(&mut errors);

        errors.finish(Self {
            context_suffix,
            crate_root,
            module,
            visibility,
        })
    }
}

pub(crate) fn parse_enum(
    enum_: &syn::DataEnum,
    name: &syn::Ident,
    generics: &syn::Generics,
    attrs: &[syn::Attribute],
) -> syn::Result<crate::EnumInfo> {
    let attrs = Attributes::from_syn(attrs);

    let mut errors = SynErrors::default();

    let mut variants = Vec::new();

    for variant in &enum_.variants {
        match crate::parse::variant_impl::parse_variant(variant) {
            Ok(v) => variants.push(v),
            Err(e) => errors.push(e),
        }
    }

    let attrs = match attrs {
        Ok(a) => a,
        Err(e) => {
            errors.push(e);
            return errors.assume_failed();
        }
    };

    let Attributes {
        context_suffix,
        crate_root,
        module,
        visibility,
    } = attrs;

    let crate_root = into_crate_root(crate_root);
    let default_suffix =
        context_suffix.map_or_else(Default::default, |cs| cs.suffix.into_suffix_kind());
    let default_visibility = visibility.map(|v| v.into_arbitrary());
    let generics = generics.clone();
    let module = module.map(|m| m.into_value());
    let name = name.clone();

    errors.finish(EnumInfo {
        crate_root,
        default_suffix,
        default_visibility,
        generics,
        module,
        name,
        variants,
    })
}
