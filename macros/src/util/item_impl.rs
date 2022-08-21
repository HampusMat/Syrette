use syn::{ImplItem, ImplItemMethod, ItemImpl};

pub fn find_impl_method_by_name<'item_impl>(
    item_impl: &'item_impl ItemImpl,
    method_name: &'static str,
) -> Option<&'item_impl ImplItemMethod>
{
    let impl_items = &item_impl.items;

    impl_items.iter().find_map(|impl_item| match impl_item {
        ImplItem::Method(method_item) => {
            if method_item.sig.ident == method_name {
                Some(method_item)
            } else {
                None
            }
        }
        &_ => None,
    })
}
