use rust_spec::{
    RustSpec,
    layout::{Robust, Stable},
    mutability::Exclusive,
    niche::WithoutNiche,
    size::{MetaSized, SliceLike, Wide},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
#[repr(transparent)]
struct Bytes([u8]);

#[test]
fn rust_spec_derives_wide() {
    assert_impl_all!(Bytes:
        RustSpec<
            Layout = Stable<Robust>,
            Size = MetaSized<SliceLike>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
        Wide<Data = u8, Metadata = usize>,
    );
}

#[cfg(feature = "alloc")]
#[test]
fn rust_spec_derives_alloc_wide_methods() {
    extern crate alloc;

    use alloc::boxed::Box;

    let slice: Box<[u8]> = Box::new([1, 2, 3]);
    let bytes = unsafe { Box::from_raw(Box::into_raw(slice) as *mut Bytes) };
    let len = bytes.metadata();
    assert_eq!(len, 3);

    let data = bytes.into_non_null();
    let bytes = unsafe { <Bytes as Wide>::from_non_null(data, len) };
    assert_eq!(bytes.metadata(), 3);
}
