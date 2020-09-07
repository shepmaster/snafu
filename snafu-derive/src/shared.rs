pub(crate) use self::context_selector::ContextSelector;

pub mod context_selector {
    use crate::{ContextSelectorKind, Field};
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote, ToTokens};

    #[derive(Copy, Clone)]
    pub(crate) struct ContextSelector<'a> {
        pub backtrace_field: Option<&'a Field>,
        pub crate_root: &'a dyn ToTokens,
        pub error_constructor_name: &'a dyn ToTokens,
        pub original_generics_without_defaults: &'a [TokenStream],
        pub parameterized_error_name: &'a dyn ToTokens,
        pub selector_doc_string: &'a str,
        pub selector_kind: &'a ContextSelectorKind,
        pub selector_name: &'a proc_macro2::Ident,
        pub user_fields: &'a [Field],
        pub visibility: Option<&'a dyn ToTokens>,
        pub where_clauses: &'a [TokenStream],
    }

    impl ToTokens for ContextSelector<'_> {
        fn to_tokens(&self, stream: &mut TokenStream) {
            use self::ContextSelectorKind::*;

            let context_selector = match self.selector_kind {
                Context { source_field, .. } => {
                    let context_selector_type = self.generate_type();
                    let context_selector_impl = match source_field {
                        Some(_) => None,
                        None => Some(self.generate_leaf()),
                    };
                    let context_selector_into_error_impl =
                        self.generate_into_error(source_field.as_ref());

                    quote! {
                        #context_selector_type
                        #context_selector_impl
                        #context_selector_into_error_impl
                    }
                }
                NoContext { source_field } => self.generate_from_source(source_field),
            };

            stream.extend(context_selector)
        }
    }

    impl ContextSelector<'_> {
        fn user_field_generics(&self) -> Vec<proc_macro2::Ident> {
            (0..self.user_fields.len())
                .map(|i| format_ident!("__T{}", i))
                .collect()
        }

        fn user_field_names(&self) -> Vec<&syn::Ident> {
            self.user_fields
                .iter()
                .map(|Field { name, .. }| name)
                .collect()
        }

        fn parameterized_selector_name(&self) -> TokenStream {
            let selector_name = self.selector_name;
            let user_generics = self.user_field_generics();
            quote! { #selector_name<#(#user_generics,)*> }
        }

        fn extended_where_clauses(&self) -> Vec<TokenStream> {
            let user_fields = self.user_fields;
            let user_field_generics = self.user_field_generics();
            let where_clauses = self.where_clauses;

            let target_types = user_fields
                .iter()
                .map(|Field { ty, .. }| quote! { ::core::convert::Into<#ty>});

            user_field_generics
                .into_iter()
                .zip(target_types)
                .map(|(gen, bound)| quote! { #gen: #bound })
                .chain(where_clauses.iter().cloned())
                .collect()
        }

        fn transfer_user_fields(&self) -> Vec<TokenStream> {
            self.user_field_names()
                .into_iter()
                .map(|name| {
                    quote! { #name: ::core::convert::Into::into(self.#name) }
                })
                .collect()
        }

        fn construct_backtrace_field(&self) -> Option<TokenStream> {
            self.backtrace_field.map(|field| {
                let crate_root = self.crate_root;
                let name = &field.name;
                quote! { #name: #crate_root::GenerateBacktrace::generate(), }
            })
        }

        fn generate_type(self) -> TokenStream {
            let visibility = self.visibility;
            let parameterized_selector_name = self.parameterized_selector_name();
            let user_field_generics = self.user_field_generics();
            let user_field_names = self.user_field_names();
            let selector_doc_string = self.selector_doc_string;

            let body = if user_field_names.is_empty() {
                quote! { ; }
            } else {
                quote! {
                    {
                        #(
                            #[allow(missing_docs)]
                            #visibility #user_field_names: #user_field_generics
                        ),*
                    }
                }
            };

            quote! {
                #[derive(Debug, Copy, Clone)]
                #[doc = #selector_doc_string]
                #visibility struct #parameterized_selector_name #body
            }
        }

        fn generate_leaf(self) -> TokenStream {
            let error_constructor_name = self.error_constructor_name;
            let original_generics_without_defaults = self.original_generics_without_defaults;
            let parameterized_error_name = self.parameterized_error_name;
            let parameterized_selector_name = self.parameterized_selector_name();
            let user_field_generics = self.user_field_generics();
            let visibility = self.visibility;
            let extended_where_clauses = self.extended_where_clauses();
            let transfer_user_fields = self.transfer_user_fields();
            let construct_backtrace_field = self.construct_backtrace_field();

            quote! {
                impl<#(#user_field_generics,)*> #parameterized_selector_name {
                    #[doc = "Consume the selector and return the associated error"]
                    #[must_use]
                    #visibility fn build<#(#original_generics_without_defaults,)*>(self) -> #parameterized_error_name
                    where
                        #(#extended_where_clauses),*
                    {
                        #error_constructor_name {
                            #construct_backtrace_field
                            #(#transfer_user_fields,)*
                        }
                    }

                    #[doc = "Consume the selector and return a `Result` with the associated error"]
                    #visibility fn fail<#(#original_generics_without_defaults,)* __T>(self) -> ::core::result::Result<__T, #parameterized_error_name>
                    where
                        #(#extended_where_clauses),*
                    {
                        ::core::result::Result::Err(self.build())
                    }
                }
            }
        }

        fn generate_into_error(self, source_field: Option<&crate::SourceField>) -> TokenStream {
            let crate_root = self.crate_root;
            let error_constructor_name = self.error_constructor_name;
            let original_generics_without_defaults = self.original_generics_without_defaults;
            let parameterized_error_name = self.parameterized_error_name;
            let parameterized_selector_name = self.parameterized_selector_name();
            let user_field_generics = self.user_field_generics();
            let extended_where_clauses = self.extended_where_clauses();
            let transfer_user_fields = self.transfer_user_fields();
            let construct_backtrace_field = self.construct_backtrace_field();

            let (source_ty, transfer_source_field) = match source_field {
                Some(source_field) => {
                    let (ty, transfer) = build_source_info(source_field);
                    (quote! { #ty }, transfer)
                }
                None => (quote! { #crate_root::NoneError }, quote! {}),
            };

            quote! {
                impl<#(#original_generics_without_defaults,)* #(#user_field_generics,)*> #crate_root::IntoError<#parameterized_error_name> for #parameterized_selector_name
                where
                    #parameterized_error_name: #crate_root::Error + #crate_root::ErrorCompat,
                    #(#extended_where_clauses),*
                {
                    type Source = #source_ty;

                    fn into_error(self, error: Self::Source) -> #parameterized_error_name {
                        #error_constructor_name {
                            #transfer_source_field
                            #construct_backtrace_field
                            #(#transfer_user_fields),*
                        }
                    }
                }
            }
        }

        fn generate_from_source(self, source_field: &crate::SourceField) -> TokenStream {
            let parameterized_error_name = self.parameterized_error_name;
            let error_constructor_name = self.error_constructor_name;
            let construct_backtrace_field = self.construct_backtrace_field();
            let original_generics_without_defaults = self.original_generics_without_defaults;
            let user_field_generics = self.user_field_generics();
            let where_clauses = self.where_clauses;

            let (source_field_type, transfer_source_field) = build_source_info(source_field);

            quote! {
                impl<#(#original_generics_without_defaults,)* #(#user_field_generics,)*> ::core::convert::From<#source_field_type> for #parameterized_error_name
                where
                    #(#where_clauses),*
                {
                    fn from(error: #source_field_type) -> Self {
                        #error_constructor_name {
                            #transfer_source_field
                            #construct_backtrace_field
                        }
                    }
                }
            }
        }
    }

    // Assumes that the error is in a variable called "error"
    fn build_source_info(source_field: &crate::SourceField) -> (&syn::Type, TokenStream) {
        let source_field_name = source_field.name();
        let source_field_type = source_field.transformation.ty();
        let source_transformation = source_field.transformation.transformation();

        (
            source_field_type,
            quote! { #source_field_name: (#source_transformation)(error), },
        )
    }
}
