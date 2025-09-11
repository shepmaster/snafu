use crate::{
    parse::{attr::ErrorLocation, field_container_impl, SynErrors},
    FieldContainer,
};

struct Attributes {
    field_container_attrs: field_container_impl::Attributes,
}

impl Attributes {
    fn from_syn(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let location = ErrorLocation::OnVariant;
        let mut errors = SynErrors::default();

        let field_container_attrs = field_container_impl::Attributes::from_syn(
            attrs,
            location,
            &mut errors,
            |errors, crate_root| {
                errors.push_invalid(crate_root, location);
            },
        );

        errors.finish(Self {
            field_container_attrs,
        })
    }
}

pub(super) fn parse_variant(variant: &syn::Variant) -> syn::Result<FieldContainer> {
    let attrs = Attributes::from_syn(&variant.attrs);

    let fields = match &variant.fields {
        syn::Fields::Named(f) => Ok(f.named.iter().collect()),
        syn::Fields::Unnamed(_) => {
            let txt = "Can only derive `Snafu` for enums with struct-like and unit enum variants";
            Err(syn::Error::new_spanned(&variant.fields, txt))
        }
        syn::Fields::Unit => Ok(vec![]),
    };

    let (attrs, fields) = join_syn_error!(attrs, fields)?;

    let name = &variant.ident;
    let attrs = attrs.field_container_attrs;

    field_container_impl::parse_field_container(
        name,
        name,
        attrs,
        &fields,
        ErrorLocation::InVariant,
    )
}
