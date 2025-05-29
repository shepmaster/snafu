use quote::quote;
use syn::{spanned::Spanned, Item, ItemFn, ReturnType, Signature};

mod parse;

#[derive(Debug, Default)]
struct Args {
    env_name: Option<String>,
}

pub fn body(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let args = syn::parse::<Args>(attr)?;
    let item = syn::parse::<Item>(item)?;

    let f = match item {
        Item::Fn(f) => f,
        _ => {
            return Err(syn::Error::new(
                item.span(),
                "`#[snafu::report]` may only be used on functions",
            ))
        }
    };

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = f;

    let Signature {
        constness,
        asyncness,
        unsafety,
        abi,
        fn_token,
        ident,
        generics,
        paren_token: _,
        inputs,
        variadic,
        output,
    } = sig;

    let output_ty = match output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    let error_ty = quote! { <#output_ty as ::snafu::__InternalExtractErrorType>::Err };

    let output = if cfg!(feature = "rust_1_61") {
        quote! { -> ::snafu::Report<#error_ty> }
    } else {
        quote! { -> ::core::result::Result<(), ::snafu::Report<#error_ty>> }
    };

    let captured_original_body = if asyncness.is_some() {
        quote! { async #block.await }
    } else {
        quote! { (|| #block)() }
    };

    let ascribed_original_result = quote! {
        let __snafu_body: #output_ty = #captured_original_body;
    };

    let Args { env_name } = args;

    let set_env_name = env_name.map(|env_name| {
        quote! {
            __snafu_report.environment_variable_name(#env_name);
        }
    });

    let set_report_options = quote! {
       #set_env_name;
    };

    let block = if cfg!(feature = "rust_1_61") {
        quote! {
            {
                #ascribed_original_result;
                let mut __snafu_report = <::snafu::Report<_> as ::core::convert::From<_>>::from(__snafu_body);
                #set_report_options;
                __snafu_report
            }
        }
    } else {
        quote! {
            {
                #ascribed_original_result;
                ::core::result::Result::map_err(__snafu_body, |e| {
                    let mut __snafu_report = ::snafu::Report::from_error(e);
                    #set_report_options;
                    __snafu_report
                })
            }
        }
    };

    Ok(quote! {
        #(#attrs)*
        #vis
        #constness
        #asyncness
        #unsafety
        #abi
        #fn_token
        #ident
        #generics
        (#inputs #variadic)
        #output
        #block
    })
}
