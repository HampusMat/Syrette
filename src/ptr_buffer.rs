//! Pointer buffer;

use std::any::TypeId;
use std::mem::{size_of, MaybeUninit};
use std::ptr::addr_of;
use std::rc::Rc;
use std::sync::Arc;

/// Pointer buffer;
pub struct PtrBuffer
{
    buf: Box<[MaybeUninit<u8>]>,
    type_id: TypeId,
    kind: Kind,
}

impl PtrBuffer
{
    /// AA.
    #[must_use]
    pub fn new_from<Value, ValuePtr>(value: ValuePtr) -> Self
    where
        Value: ?Sized + 'static,
        ValuePtr: Into<SmartPtr<Value>>,
    {
        let value = value.into();

        let kind = value.kind();

        let buf = ptr_to_byte_buf(value.into_raw());

        Self {
            buf,
            type_id: TypeId::of::<Value>(),
            kind,
        }
    }

    pub(crate) fn cast_into_boxed<Dest>(self) -> Option<Box<Dest>>
    where
        Dest: ?Sized + 'static,
    {
        if !matches!(self.kind, Kind::Box) {
            return None;
        }

        let dest_ptr = self.cast_into()?;

        // SAFETY: We know the pointer was retrieved using Box::into_raw in the
        // new_from function since the kind is Kind::Box (checked above). We
        // also know it was the exact same pointed to type since this is checked in the
        // cast_into function
        Some(unsafe { Box::from_raw(dest_ptr) })
    }

    pub(crate) fn cast_into_rc<Dest>(self) -> Option<Rc<Dest>>
    where
        Dest: ?Sized + 'static,
    {
        if !matches!(self.kind, Kind::Rc) {
            return None;
        }

        let dest_ptr = self.cast_into()?;

        // SAFETY: We know the pointer was retrieved using Rc::into_raw in the
        // new_from function since the kind is Kind::Rc (checked above). We
        // also know it was the exact same pointed to type since this is checked in the
        // cast_into function
        Some(unsafe { Rc::from_raw(dest_ptr) })
    }

    #[cfg(feature = "async")]
    pub(crate) fn cast_into_arc<Dest>(self) -> Option<Arc<Dest>>
    where
        Dest: ?Sized + 'static,
    {
        if !matches!(self.kind, Kind::Arc) {
            return None;
        }

        let dest_ptr = self.cast_into()?;

        // SAFETY: We know the pointer was retrieved using Arc::into_raw in the
        // new_from function since the kind is Kind::Arc (checked above). We
        // also know it was the exact same pointed to type since this is checked in the
        // cast_into function
        Some(unsafe { Arc::from_raw(dest_ptr) })
    }

    fn cast_into<Dest>(self) -> Option<*mut Dest>
    where
        Dest: ?Sized + 'static,
    {
        if TypeId::of::<Dest>() != self.type_id {
            return None;
        }

        if size_of::<*mut Dest>() != self.buf.len() {
            // Pointer kinds are different so continuing would cause UB. This should
            // not be possible since the type IDs are the same but we check it just to
            // be extra safe
            return None;
        }

        let mut ptr = MaybeUninit::<*mut Dest>::uninit();

        // SAFETY:
        // - We know the source buffer is valid for reads the number of bytes since it is
        //   ensured by the array primitive
        // - We know the destination is valid for writes the number of bytes since we
        //   check above if the buffer length is the same as the size of *mut Dest
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.buf.as_ptr().cast::<u8>(),
                ptr.as_mut_ptr().cast::<u8>(),
                self.buf.len(),
            );
        }

        // SAFETY: We initialize the value above by copying the buffer to it
        Some(unsafe { ptr.assume_init() })
    }
}

/// Smart pointers supported as input to [`PtrBuffer`].
#[derive(Debug)]
#[non_exhaustive]
pub enum SmartPtr<Value: ?Sized + 'static>
{
    /// Box.
    Box(Box<Value>),

    /// Rc.
    Rc(Rc<Value>),

    /// Arc.
    Arc(Arc<Value>),
}

impl<Value: ?Sized + 'static> SmartPtr<Value>
{
    fn into_raw(self) -> *const Value
    {
        match self {
            Self::Box(value) => Box::into_raw(value),
            Self::Rc(value) => Rc::into_raw(value),
            Self::Arc(value) => Arc::into_raw(value),
        }
    }

    fn kind(&self) -> Kind
    {
        match self {
            Self::Box(_) => Kind::Box,
            Self::Rc(_) => Kind::Rc,
            Self::Arc(_) => Kind::Arc,
        }
    }
}

impl<Value> From<Box<Value>> for SmartPtr<Value>
where
    Value: ?Sized + 'static,
{
    fn from(value: Box<Value>) -> Self
    {
        Self::Box(value)
    }
}

impl<Value> From<Rc<Value>> for SmartPtr<Value>
where
    Value: ?Sized + 'static,
{
    fn from(value: Rc<Value>) -> Self
    {
        Self::Rc(value)
    }
}

impl<Value> From<Arc<Value>> for SmartPtr<Value>
where
    Value: ?Sized + 'static,
{
    fn from(value: Arc<Value>) -> Self
    {
        Self::Arc(value)
    }
}

enum Kind
{
    Box,
    Rc,
    Arc,
}

fn ptr_to_byte_buf<Value>(value_ptr: *const Value) -> Box<[MaybeUninit<u8>]>
where
    Value: ?Sized + 'static,
{
    // Transform the full pointer (data pointer + (optional) metadata) into a byte
    // slice
    let value_ptr_bytes = unsafe {
        std::slice::from_raw_parts::<u8>(
            addr_of!(value_ptr).cast::<u8>(),
            size_of::<*const Value>(),
        )
    };

    value_ptr_bytes
        .iter()
        .map(|byte| MaybeUninit::new(*byte))
        .collect::<Vec<_>>()
        .into()
}

#[cfg(test)]
mod tests
{
    use std::mem::{size_of, transmute, MaybeUninit};
    use std::path::PathBuf;
    use std::rc::Rc;

    use crate::ptr_buffer::{ptr_to_byte_buf, PtrBuffer};

    trait Anything
    {
        fn get_value(&self) -> u32;
    }

    struct Something;

    impl Anything for Something
    {
        fn get_value(&self) -> u32
        {
            1234
        }
    }

    #[test]
    fn works_with_thin()
    {
        let text = Box::new("Hello there".to_string());

        let ptr_buf = PtrBuffer::new_from(text);

        assert!(ptr_buf
            .cast_into_boxed::<String>()
            .map_or_else(|| false, |text| *text == "Hello there"));
    }

    #[test]
    fn works_with_dyn()
    {
        let text: Box<dyn Anything> = Box::new(Something);

        let ptr_buf = PtrBuffer::new_from(text);

        assert!(ptr_buf
            .cast_into_boxed::<dyn Anything>()
            .map_or_else(|| false, |anything| anything.get_value() == 1234));
    }

    #[test]
    fn cast_box_when_wrong_kind_fails()
    {
        let text = Rc::new("Hello there".to_string());

        let ptr_buf = PtrBuffer::new_from(text);

        assert!(ptr_buf.cast_into_boxed::<String>().is_none());
    }

    #[test]
    fn cast_rc_when_wrong_kind_fails()
    {
        let text = Box::new("Hello there".to_string());

        let ptr_buf = PtrBuffer::new_from(text);

        assert!(ptr_buf.cast_into_rc::<String>().is_none());
    }

    #[test]
    #[cfg(feature = "async")]
    fn cast_arc_when_wrong_kind_fails()
    {
        let text = Box::new("Hello there".to_string());

        let ptr_buf = PtrBuffer::new_from(text);

        assert!(ptr_buf.cast_into_arc::<String>().is_none());
    }

    #[test]
    fn cast_into_fails_when_wrong_type()
    {
        let text = Box::new(123_456u64);

        let ptr_buf = PtrBuffer::new_from(text);

        assert!(ptr_buf.cast_into::<PathBuf>().is_none());
    }

    #[test]
    fn ptr_to_byte_buf_works()
    {
        let thin_ptr_addr = 123_456_789usize;

        assert_eq!(
            unsafe {
                slice_assume_init_ref(&ptr_to_byte_buf(thin_ptr_addr as *const u32))
            },
            thin_ptr_addr.to_ne_bytes()
        );

        let fat_ptr_addr_buf: [u8; size_of::<*const dyn Send>()] =
            [26, 88, 91, 77, 2, 0, 0, 0, 12, 34, 56, 78, 90, 9, 98, 87];

        let fat_ptr_addr: *const dyn Send = unsafe { transmute(fat_ptr_addr_buf) };

        assert_eq!(
            unsafe { slice_assume_init_ref(&ptr_to_byte_buf(fat_ptr_addr)) },
            &fat_ptr_addr_buf
        );
    }

    /// TODO: Remove when `MaybeUninit::slice_assume_init_ref` is stabilized
    const unsafe fn slice_assume_init_ref<T>(slice: &[MaybeUninit<T>]) -> &[T]
    {
        // SAFETY: casting `slice` to a `*const [T]` is safe since the caller guarantees
        // that `slice` is initialized, and `MaybeUninit` is guaranteed to have
        // the same layout as `T`. The pointer obtained is valid since it refers
        // to memory owned by `slice` which is a reference and thus guaranteed to
        // be valid for reads.
        unsafe { &*(slice as *const [MaybeUninit<T>] as *const [T]) }
    }
}
