/**
 * Originally from Intertrait by CodeChain
 *
 * <https://github.com/CodeChain-io/intertrait>
 * <https://crates.io/crates/intertrait/0.2.2>
 *
 * Licensed under either of
 *
 * Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)

 * at your option.
*/
use std::str::from_utf8;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use uuid::adapter::Simple;
use uuid::Uuid;

const CASTER_FN_NAME_PREFIX: &[u8] = b"__";

const FN_BUF_LEN: usize = CASTER_FN_NAME_PREFIX.len() + Simple::LENGTH;

pub fn generate_caster(
    ty: &impl ToTokens,
    dst_trait: &impl ToTokens,
    sync: bool,
) -> TokenStream
{
    let fn_ident = create_caster_fn_ident();

    let new_caster = if sync {
        quote! {
            syrette::libs::intertrait::Caster::<#dst_trait>::new_sync(
                |from| {
                    let concrete = from
                        .downcast::<#ty>()
                        .map_err(|_| syrette::libs::intertrait::CasterError::CastBoxFailed)?;

                    Ok(concrete as Box<#dst_trait>)
                },
                |from| {
                    let concrete = from
                        .downcast::<#ty>()
                        .map_err(|_| syrette::libs::intertrait::CasterError::CastRcFailed)?;

                    Ok(concrete as std::rc::Rc<#dst_trait>)
                },
                |from| {
                    let concrete = from
                        .downcast::<#ty>()
                        .map_err(|_| syrette::libs::intertrait::CasterError::CastArcFailed)?;

                    Ok(concrete as std::sync::Arc<#dst_trait>)
                },
            )
        }
    } else {
        quote! {
            syrette::libs::intertrait::Caster::<#dst_trait>::new(
                |from| {
                    let concrete = from
                        .downcast::<#ty>()
                        .map_err(|_| syrette::libs::intertrait::CasterError::CastBoxFailed)?;

                    Ok(concrete as Box<#dst_trait>)
                },
                |from| {
                    let concrete = from
                        .downcast::<#ty>()
                        .map_err(|_| syrette::libs::intertrait::CasterError::CastRcFailed)?;

                    Ok(concrete as std::rc::Rc<#dst_trait>)
                },
            )
        }
    };

    quote! {
        #[syrette::libs::linkme::distributed_slice(syrette::libs::intertrait::CASTERS)]
        #[linkme(crate = syrette::libs::linkme)]
        fn #fn_ident() -> (::std::any::TypeId, syrette::libs::intertrait::BoxedCaster) {
            (::std::any::TypeId::of::<#ty>(), Box::new(#new_caster))
        }
    }
}

fn create_caster_fn_ident() -> Ident
{
    let buf = &mut [0u8; FN_BUF_LEN];

    buf[..CASTER_FN_NAME_PREFIX.len()].copy_from_slice(CASTER_FN_NAME_PREFIX);

    Uuid::new_v4()
        .to_simple()
        .encode_lower(&mut buf[CASTER_FN_NAME_PREFIX.len()..]);

    let fn_name =
        from_utf8(&buf[..FN_BUF_LEN]).expect("Created caster function name is not UTF-8");

    format_ident!("{}", fn_name)
}
