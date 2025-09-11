use crate::parse::{self, attr::ErrorLocation, AtMostOne, Attribute, SynErrors};

struct Attributes;

impl Attributes {
    fn from_syn(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let location = ErrorLocation::OnTupleStructField;
        let mut errors = SynErrors::default();

        parse::syn_attrs(attrs, &mut errors, |errors, attr| {
            use Attribute::*;

            match attr {
                Backtrace(a) => errors.push_invalid_flag(a, location),
                ContextFlag(a) => errors.push_invalid_flag(a, location),
                ContextName(a) => errors.push_invalid(a, location),
                ContextSuffix(a) => errors.push_invalid(a, location),
                CrateRoot(a) => errors.push_invalid(a, location),
                Display(a) => errors.push_invalid(a, location),
                DocComment(_a) => { /* no-op */ }
                Implicit(a) => errors.push_invalid_flag(a, location),
                Module(a) => errors.push_invalid(a, location),
                ProvideFlag(a) => errors.push_invalid_flag(a, location),
                ProvideExpression(a) => errors.push_invalid(a, location),
                SourceFlag(a) => errors.push_invalid_flag(a, location),
                SourceFrom(a) => errors.push_invalid(a, location),
                Transparent(a) => errors.push_invalid_flag(a, location),
                Visibility(a) => errors.push_invalid(a, location),
                Whatever(a) => errors.push_invalid(a, location),
            }
        });

        errors.finish(Self)
    }
}

pub fn parse_tuple_struct_field(
    fields: &syn::FieldsUnnamed,
    span: impl quote::ToTokens,
) -> syn::Result<syn::Type> {
    let txt = "Can only derive `Snafu` for tuple structs with exactly one field";

    let mut unnamed_fields = AtMostOne::new(txt);
    unnamed_fields.extend(fields.unnamed.iter());
    let field = unnamed_fields.finish_exactly_one(span)?;

    Attributes::from_syn(&field.attrs)?;
    Ok(field.ty.clone())
}
