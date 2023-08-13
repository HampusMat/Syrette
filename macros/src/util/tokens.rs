use std::fmt::Write;

use proc_macro2::{Delimiter, Spacing, TokenTree};
use quote::ToTokens;

pub trait ToTokensExt
{
    fn to_str_pretty(&self) -> String;
}

impl<T: ToTokens> ToTokensExt for T
{
    fn to_str_pretty(&self) -> String
    {
        let mut spaceable = Spaceable::None;

        self.to_token_stream()
            .into_iter()
            .fold(String::new(), |mut acc, token_tree| {
                let prev_spaceable = spaceable;

                spaceable = get_tt_spaceable(&token_tree);

                if matches!(prev_spaceable, Spaceable::Left | Spaceable::LeftRight)
                    && matches!(spaceable, Spaceable::Right | Spaceable::LeftRight)
                {
                    write!(acc, " ").ok();
                }

                match token_tree {
                    TokenTree::Group(group) => match group.delimiter() {
                        Delimiter::Parenthesis => {
                            write!(acc, "({})", group.stream().to_str_pretty()).ok();
                        }
                        Delimiter::Brace => {
                            write!(acc, "{{{}}}", group.stream().to_str_pretty()).ok();
                        }
                        Delimiter::Bracket => {
                            write!(acc, "[{}]", group.stream().to_str_pretty()).ok();
                        }
                        Delimiter::None => {
                            write!(acc, "{}", group.stream().to_str_pretty()).ok();
                        }
                    },
                    tt => {
                        write!(acc, "{tt}").ok();
                    }
                }

                acc
            })
    }
}

fn get_tt_spaceable(token_tree: &TokenTree) -> Spaceable
{
    match &token_tree {
        TokenTree::Ident(_) => Spaceable::LeftRight,
        TokenTree::Punct(punct)
            if punct.spacing() == Spacing::Alone && (punct.as_char() == '+') =>
        {
            Spaceable::LeftRight
        }
        TokenTree::Punct(punct)
            if punct.spacing() == Spacing::Alone
                && (punct.as_char() == '>' || punct.as_char() == ',') =>
        {
            Spaceable::Left
        }
        TokenTree::Punct(punct)
            if punct.spacing() == Spacing::Joint && punct.as_char() == '-' =>
        {
            // Is part of ->
            Spaceable::Right
        }
        TokenTree::Punct(punct) if punct.as_char() == '&' => Spaceable::Right,
        TokenTree::Group(_) => Spaceable::Left,
        _ => Spaceable::None,
    }
}

#[derive(Debug, Clone, Copy)]
enum Spaceable
{
    Left,
    Right,
    LeftRight,
    None,
}
