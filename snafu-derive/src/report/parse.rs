use quote::ToTokens;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, LitBool, LitStr, Path,
};

use super::Args;

mod kw {
    use syn::custom_keyword;

    custom_keyword!(crate_root);
    custom_keyword!(env);
    custom_keyword!(show_note);
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
            .map(|arg| -> Result<(), syn::Error> {
                match &arg {
                    Arg::CrateRoot { value, .. } => {
                        let value = Box::new(value.to_token_stream());
                        set_once(&mut args.crate_root, value, "crate_root", arg)
                    }

                    Arg::EnvName { value, .. } => {
                        let value = value.value();
                        set_once(&mut args.env_name, value, "env", arg)
                    }

                    Arg::ShowNote { value, .. } => {
                        let value = value.value;
                        if value != false {
                            return Err(syn::Error::new_spanned(
                                arg,
                                "`show_note` may only be set to `false`",
                            ));
                        }
                        set_once(&mut args.show_note, value, "show_note", arg)
                    }
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
    CrateRoot {
        crate_root_token: kw::crate_root,
        paren_token: token::Paren,
        value: Path,
    },

    EnvName {
        env_token: kw::env,
        paren_token: token::Paren,
        value: LitStr,
    },

    ShowNote {
        show_note_token: kw::show_note,
        paren_token: token::Paren,
        value: LitBool,
    },
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::crate_root) {
            let content;
            Ok(Arg::CrateRoot {
                crate_root_token: input.parse()?,
                paren_token: parenthesized!(content in input),
                value: content.parse()?,
            })
        } else if lookahead.peek(kw::env) {
            let content;
            Ok(Arg::EnvName {
                env_token: input.parse()?,
                paren_token: parenthesized!(content in input),
                value: content.parse()?,
            })
        } else if lookahead.peek(kw::show_note) {
            let content;
            Ok(Arg::ShowNote {
                show_note_token: input.parse()?,
                paren_token: parenthesized!(content in input),
                value: content.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Arg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Arg::CrateRoot {
                crate_root_token,
                paren_token,
                value,
            } => {
                crate_root_token.to_tokens(tokens);
                paren_token.surround(tokens, |tokens| {
                    value.to_tokens(tokens);
                });
            }

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

            Arg::ShowNote {
                show_note_token,
                paren_token,
                value,
            } => {
                show_note_token.to_tokens(tokens);
                paren_token.surround(tokens, |tokens| {
                    value.to_tokens(tokens);
                });
            }
        }
    }
}
