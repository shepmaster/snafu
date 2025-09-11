/// Defines information for reporting errors about each attribute.
use std::fmt;

pub(super) trait Attribute {
    const NAME: &'static str;

    const VALID_ON: &'static str;

    const DUPLICATE: DuplicateAttribute = DuplicateAttribute {
        attribute: Self::NAME,
    };

    const INVALID: OnlyValidOn = OnlyValidOn {
        attribute: Self::NAME,
        valid_on: Self::VALID_ON,
    };
}

pub(super) trait FlagAttribute: Attribute {
    const BASE_NAME: &'static str;

    const BASE_INVALID: OnlyValidOn = OnlyValidOn {
        attribute: Self::BASE_NAME,
        valid_on: Self::VALID_ON,
    };
}

// Any meaningful name here would be as long as the string, so just
// use some unique ID.
const VALID_A: &str = "an enum or a struct";
const VALID_B: &str = "an enum or structs with named fields";
const VALID_C: &str = "an enum, enum variants, or a struct with named fields";
const VALID_D: &str = "enum variant or struct fields with a name";
const VALID_E: &str = "enum variants or structs with named fields";
const VALID_F: &str = "enum variants, structs with named fields, or tuple structs";

macro_rules! def_attributes {
    ($(($name:ident, $attr:expr, $valid:expr)),*$(,)?) => {
        $(
            pub(super) struct $name;

            impl Attribute for $name {
                const NAME: &'static str = $attr;
                const VALID_ON: &'static str = $valid;
            }
        )*
    };
}

def_attributes![
    (Backtrace, "backtrace", VALID_D),
    (ContextFlag, "context(bool)", VALID_E),
    (ContextName, "context(name)", VALID_E),
    (ContextSuffix, "context(suffix)", VALID_E),
    (CrateRoot, "crate_root", VALID_A),
    (Display, "display", VALID_E),
    (Implicit, "implicit", VALID_D),
    (Module, "module", VALID_B),
    (ProvideExpression, "provide(type => expression)", VALID_F),
    (ProvideFlag, "provide(bool)", VALID_D),
    (SourceFlag, "source(bool)", VALID_D),
    (SourceFrom, "source(from)", VALID_D),
    (Transparent, "transparent", VALID_E),
    (Visibility, "visibility", VALID_C),
    (Whatever, "whatever", VALID_E),
];

macro_rules! def_flag_attributes {
    ($(($name:ident, $attr:expr)),*$(,)?) => {
        $(
            impl FlagAttribute for $name {
                const BASE_NAME: &'static str = $attr;
            }
        )*
    };
}

def_flag_attributes![
    (Backtrace, "backtrace"),
    (ContextFlag, "context"),
    (Implicit, "implicit"),
    (ProvideFlag, "provide"),
    (SourceFlag, "source"),
    (Transparent, "transparent"),
];

impl Backtrace {
    pub(super) const FALSE_ON_WRONG_FIELD: WrongField = WrongField {
        attribute: "backtrace(false)",
        valid_field: "backtrace",
    };

    pub(super) const DUPLICATE_FIELD: DuplicateField = DuplicateField { field: Self::NAME };
}

impl Implicit {
    pub(super) const FALSE_DOES_NOTHING: DoesNothing = DoesNothing {
        attribute: "implicit(false)",
    };
}

pub(super) struct Provide;

impl Provide {
    pub(super) const FALSE_ON_WRONG_FIELD: WrongField = WrongField {
        attribute: "provide(false)",
        valid_field: r#"source" or "backtrace"#,
    };
}

pub(super) struct Source;

impl Source {
    const NAME: &'static str = "source";
    const FALSE: &'static str = "source(false)";

    pub(super) const FALSE_ON_WRONG_FIELD: WrongField = WrongField {
        attribute: Self::FALSE,
        valid_field: Self::NAME,
    };

    pub(super) const FALSE_AND_FROM_INCOMPATIBLE: IncompatibleAttributes<'static> =
        IncompatibleAttributes::new(&[Self::FALSE, SourceFrom::NAME]);

    pub(super) const DUPLICATE_FIELD: DuplicateField = DuplicateField { field: Self::NAME };
}

impl Transparent {
    pub(super) const FALSE_DOES_NOTHING: DoesNothing = DoesNothing {
        attribute: "transparent(false)",
    };
}

#[derive(Copy, Clone)]
pub(super) enum ErrorLocation {
    OnEnum,
    OnVariant,
    InVariant,
    OnField,
    OnNamedStruct,
    InNamedStruct,
    OnTupleStruct,
    OnTupleStructField,
}

impl fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ErrorLocation::*;

        match self {
            OnEnum => "on an enum".fmt(f),
            OnVariant => "on an enum variant".fmt(f),
            InVariant => "within an enum variant".fmt(f),
            OnField => "on a field".fmt(f),
            OnNamedStruct => "on a named struct".fmt(f),
            InNamedStruct => "within a named struct".fmt(f),
            OnTupleStruct => "on a tuple struct".fmt(f),
            OnTupleStructField => "on a tuple struct field".fmt(f),
        }
    }
}

pub(super) trait ErrorForLocation {
    fn for_location(&self, location: ErrorLocation, f: &mut fmt::Formatter) -> fmt::Result;

    fn on(self, location: ErrorLocation) -> ErrorWithLocation<Self>
    where
        Self: Sized,
    {
        ErrorWithLocation(self, location)
    }
}

#[derive(Copy, Clone)]
pub(super) struct ErrorWithLocation<T>(T, ErrorLocation);

impl<T> fmt::Display for ErrorWithLocation<T>
where
    T: ErrorForLocation,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.for_location(self.1, f)
    }
}

/// Helper structure to handle cases where an attribute is
/// syntactically valid but semantically invalid.
#[derive(Copy, Clone)]
pub(super) struct DoesNothing {
    /// The name of the attribute that was misused.
    attribute: &'static str,
}

impl fmt::Display for DoesNothing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}` attribute has no effect", self.attribute)
    }
}

/// Helper structure to handle cases where an attribute was used on an
/// element where it's not valid.
#[derive(Copy, Clone)]
pub(super) struct OnlyValidOn {
    /// The name of the attribute that was misused.
    attribute: &'static str,
    /// A description of where that attribute is valid.
    valid_on: &'static str,
}

impl ErrorForLocation for OnlyValidOn {
    fn for_location(&self, location: ErrorLocation, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` attribute is only valid on {}, not {}",
            self.attribute, self.valid_on, location,
        )
    }
}

/// Helper structure to handle cases where a specific attribute value
/// was used on an field where it's not valid.
#[derive(Copy, Clone)]
pub(super) struct WrongField {
    /// The name of the attribute that was misused.
    attribute: &'static str,
    /// The name of the field where that attribute is valid.
    valid_field: &'static str,
}

impl fmt::Display for WrongField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#"`{}` attribute is only valid on a field named "{}", not on other fields"#,
            self.attribute, self.valid_field,
        )
    }
}

/// Helper structure to handle cases where two incompatible attributes
/// were specified on the same element.
#[derive(Copy, Clone)]
pub(super) struct IncompatibleAttributes<'a> {
    pub(super) attrs: &'a [&'static str],
    pub(super) bonus: Option<&'static str>,
}

impl<'a> IncompatibleAttributes<'a> {
    pub(super) const fn new(attrs: &'a [&'static str]) -> Self {
        Self { attrs, bonus: None }
    }
}

impl<'a> ErrorForLocation for IncompatibleAttributes<'a> {
    fn for_location(&self, location: ErrorLocation, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some((attr0, attrs)) = self.attrs.split_first() {
            write!(f, "`{}`", attr0)?;
            let mut attrs = attrs.iter().peekable();
            while let Some(attr) = attrs.next() {
                if attrs.peek().is_some() {
                    write!(f, ", `{}`", attr)?;
                } else {
                    write!(f, " and `{}`", attr)?;
                }
            }
        }

        write!(f, " may not be provided together {}", location)?;

        if let Some(bonus) = self.bonus {
            write!(f, ". {}.", bonus)?;
        }

        Ok(())
    }
}

/// Helper structure to handle cases where an attribute was
/// incorrectly used multiple times on the same element.
#[derive(Copy, Clone)]
pub(super) struct DuplicateAttribute {
    attribute: &'static str,
}

impl ErrorForLocation for DuplicateAttribute {
    fn for_location(&self, location: ErrorLocation, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Multiple `{}` attributes are not supported {}",
            self.attribute, location,
        )
    }
}

/// Helper structure to handle cases where a field was
/// incorrectly used multiple times.
#[derive(Copy, Clone)]
pub(super) struct DuplicateField {
    field: &'static str,
}

impl ErrorForLocation for DuplicateField {
    fn for_location(&self, location: ErrorLocation, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Multiple {} fields are not supported {}",
            self.field, location,
        )
    }
}
