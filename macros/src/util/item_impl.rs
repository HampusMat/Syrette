use syn::{ImplItem, ImplItemMethod, ItemImpl};

pub fn find_impl_method_by_name_mut<'item_impl>(
    item_impl: &'item_impl mut ItemImpl,
    method_name: &'static str,
) -> Option<&'item_impl mut ImplItemMethod>
{
    let impl_items = &mut item_impl.items;

    impl_items.iter_mut().find_map(|impl_item| match impl_item {
        ImplItem::Method(method_item) => {
            if method_item.sig.ident == method_name {
                Some(method_item)
            } else {
                None
            }
        }
        &mut _ => None,
    })
}
