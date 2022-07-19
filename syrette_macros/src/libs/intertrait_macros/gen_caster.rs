/**
 * Originally from Intertrait by CodeChain
 *
 * https://github.com/CodeChain-io/intertrait
 * https://crates.io/crates/intertrait/0.2.2
 *
 * Licensed under either of
 *
 * Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

 * at your option.
*/
use std::str::from_utf8_unchecked;

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use uuid::adapter::Simple;
use uuid::Uuid;

pub fn generate_caster(ty: &impl ToTokens, trait_: &impl ToTokens) -> TokenStream
{
    let mut fn_buf = [0u8; FN_BUF_LEN];

    let fn_ident = format_ident!("{}", new_fn_name(&mut fn_buf));

    let new_caster = quote! {
        syrette::libs::intertrait::Caster::<dyn #trait_>::new(
            |from| from.downcast::<#ty>().unwrap(),
            |from| from.downcast::<#ty>().unwrap(),
        )
    };

    quote! {
        #[syrette::libs::linkme::distributed_slice(syrette::libs::intertrait::CASTERS)]
        #[linkme(crate = syrette::libs::linkme)]
        fn #fn_ident() -> (::std::any::TypeId, syrette::libs::intertrait::BoxedCaster) {
            (::std::any::TypeId::of::<#ty>(), Box::new(#new_caster))
        }
    }
}

const FN_PREFIX: &[u8] = b"__";
const FN_BUF_LEN: usize = FN_PREFIX.len() + Simple::LENGTH;

fn new_fn_name(buf: &mut [u8]) -> &str
{
    buf[..FN_PREFIX.len()].copy_from_slice(FN_PREFIX);
    Uuid::new_v4()
        .to_simple()
        .encode_lower(&mut buf[FN_PREFIX.len()..]);
    unsafe { from_utf8_unchecked(&buf[..FN_BUF_LEN]) }
}
