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
    let fn_ident = create_caster_fn_ident(Uuid::new_v4());

    let new_caster = if sync {
        quote! {
            syrette::private::cast::Caster::<#dst_trait>::new_sync(
                |from| {
                    let concrete = from
                        .downcast::<#ty>()
                        .map_err(|_| syrette::private::cast::CasterError::CastBoxFailed)?;

                    Ok(concrete as Box<#dst_trait>)
                },
                |from| {
                    let concrete = from
                        .downcast::<#ty>()
                        .map_err(|_| syrette::private::cast::CasterError::CastRcFailed)?;

                    Ok(concrete as std::rc::Rc<#dst_trait>)
                },
                |from| {
                    let concrete = from
                        .downcast::<#ty>()
                        .map_err(|_| syrette::private::cast::CasterError::CastArcFailed)?;

                    Ok(concrete as std::sync::Arc<#dst_trait>)
                },
            )
        }
    } else {
        quote! {
            syrette::private::cast::Caster::<#dst_trait>::new(
                |from| {
                    let concrete = from
                        .downcast::<#ty>()
                        .map_err(|_| syrette::private::cast::CasterError::CastBoxFailed)?;

                    Ok(concrete as Box<#dst_trait>)
                },
                |from| {
                    let concrete = from
                        .downcast::<#ty>()
                        .map_err(|_| syrette::private::cast::CasterError::CastRcFailed)?;

                    Ok(concrete as std::rc::Rc<#dst_trait>)
                },
            )
        }
    };

    quote! {
        #[syrette::private::linkme::distributed_slice(syrette::private::cast::CASTERS)]
        #[linkme(crate = syrette::private::linkme)]
        fn #fn_ident() -> (::std::any::TypeId, syrette::private::cast::BoxedCaster) {
            (::std::any::TypeId::of::<#ty>(), Box::new(#new_caster))
        }
    }
}

fn create_caster_fn_ident(uuid: impl IUuid) -> Ident
{
    let buf = &mut [0u8; FN_BUF_LEN];

    buf[..CASTER_FN_NAME_PREFIX.len()].copy_from_slice(CASTER_FN_NAME_PREFIX);

    uuid.encode_simple_lower_into(&mut buf[CASTER_FN_NAME_PREFIX.len()..]);

    let fn_name =
        from_utf8(&buf[..FN_BUF_LEN]).expect("Created caster function name is not UTF-8");

    format_ident!("{}", fn_name)
}

/// Simple interface for `Uuid`.
///
/// Created for ease of testing the [`create_caster_fn_ident`] function.
///
/// [`Uuid`]: uuid::Uuid
#[cfg_attr(test, mockall::automock)]
trait IUuid
{
    /// Writes the Uuid as a simple lower-case string to `buf`.
    fn encode_simple_lower_into(self, buf: &mut [u8]);
}

impl IUuid for Uuid
{
    fn encode_simple_lower_into(self, buf: &mut [u8])
    {
        self.to_simple().encode_lower(buf);
    }
}

#[cfg(test)]
mod tests
{
    use pretty_assertions::assert_eq;
    use utility_macros::repeat_char;

    use super::*;

    #[test]
    fn can_create_caster_fn_ident()
    {
        let mut uuid_mock = MockIUuid::new();

        uuid_mock
            .expect_encode_simple_lower_into()
            .return_once(|buf| {
                for index in 0..(FN_BUF_LEN - 2) {
                    buf[index] = b'f';
                }
            })
            .once();

        assert_eq!(
            create_caster_fn_ident(uuid_mock),
            format_ident!(concat!("__", repeat_char!('f', 32)))
        );
    }
}
