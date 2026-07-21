use rust_spec::{
    RustSpec,
    layout::{Robust, Stable},
    mutability::Exclusive,
    niche::WithoutNiche,
    size::{SliceLike, Wide, MetaSized},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
#[repr(transparent)]
struct Bytes([u8]);

fn main() {
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
