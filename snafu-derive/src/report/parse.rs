use quote::ToTokens;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, LitStr,
};

use super::Args;

mod kw {
    use syn::custom_keyword;

    custom_keyword!(env);
}

fn set_once<T>(
    option: &mut Option<T>,
    value: T,
    name: &str,
    span: impl ToTokens,
) -> Result<(), syn::Error> {
    match option {
        None => {
            *option = Some(value);
            Ok(())
        }

        Some(_) => {
            let message = format!("`{}` may only be provided once", name);
            Err(syn::Error::new_spanned(span, message))
        }
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Args::default();

        let parser = Punctuated::<Arg, token::Comma>::parse_terminated;
        parser(input)?
            .into_iter()
            .map(|arg| match &arg {
                Arg::EnvName { value, .. } => {
                    set_once(&mut args.env_name, value.value(), "env", arg)
                }
            })
            .fold(Ok(()), |acc, r| match (acc, r) {
                (Ok(()), Ok(())) => Ok(()),
                (Ok(()), Err(e)) | (Err(e), Ok(())) => Err(e),
                (Err(mut e1), Err(e2)) => {
                    e1.combine(e2);
                    Err(e1)
                }
            })?;

        Ok(args)
    }
}

enum Arg {
    EnvName {
        env_token: kw::env,
        paren_token: token::Paren,
        value: LitStr,
    },
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Arg::EnvName {
            env_token: input.parse()?,
            paren_token: parenthesized!(content in input),
            value: content.parse()?,
        })
    }
}

impl ToTokens for Arg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Arg::EnvName {
                env_token,
                paren_token,
                value,
            } => {
                env_token.to_tokens(tokens);
                paren_token.surround(tokens, |tokens| {
                    value.to_tokens(tokens);
                });
            }
        }
    }
}
