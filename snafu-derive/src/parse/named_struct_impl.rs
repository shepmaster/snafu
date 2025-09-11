use crate::{
    parse::{
        attr::{self, ErrorLocation},
        field_container_impl::{self, parse_field_container},
        into_crate_root, AtMostOne, CrateRoot, SynErrors,
    },
    NamedStructInfo,
};

struct Attributes {
    crate_root: Option<CrateRoot>,
    field_container_attrs: field_container_impl::Attributes,
}

impl Attributes {
    fn from_syn(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let location = ErrorLocation::OnNamedStruct;
        let mut errors = SynErrors::default();

        let mut crate_roots = AtMostOne::attribute(attr::CrateRoot, location);

        let field_container_attrs = field_container_impl::Attributes::from_syn(
            attrs,
            location,
            &mut errors,
            |_errors, crate_root| {
                crate_roots.push(crate_root);
            },
        );

        let crate_root = crate_roots.finish_default(&mut errors);

        errors.finish(Self {
            crate_root,
            field_container_attrs,
        })
    }
}

pub(crate) fn parse_named_struct(
    fields: &[&syn::Field],
    name: &syn::Ident,
    generics: &syn::Generics,
    attrs: &[syn::Attribute],
    span: impl quote::ToTokens,
) -> syn::Result<NamedStructInfo> {
    let attrs = Attributes::from_syn(attrs)?;
    let Attributes {
        crate_root,
        field_container_attrs,
    } = attrs;

    let field_container = parse_field_container(
        name,
        span,
        field_container_attrs,
        fields,
        ErrorLocation::InNamedStruct,
    )?;

    let crate_root = into_crate_root(crate_root);
    let generics = generics.clone();

    Ok(NamedStructInfo {
        crate_root,
        field_container,
        generics,
    })
}
