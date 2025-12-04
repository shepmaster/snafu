use quote::quote;
use syn::{spanned::Spanned, Item, ItemFn, ReturnType, Signature};

pub fn body(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
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

    let output = quote! { -> ::snafu::Report<#error_ty> };

    let captured_original_body = if asyncness.is_some() {
        quote! { async #block.await }
    } else {
        quote! { (|| #block)() }
    };

    let ascribed_original_result = quote! {
        let __snafu_body: #output_ty = #captured_original_body;
    };

    let block = quote! {
        {
            #ascribed_original_result;
            <::snafu::Report<_> as ::core::convert::From<_>>::from(__snafu_body)
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
