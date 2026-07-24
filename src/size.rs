use core::{convert::Infallible, ops::Add};

/// Closed family of size and pointer-metadata classifications.
#[sealed::sealed]
pub trait SizeSpec {}

#[sealed::sealed]
trait SizedKindSpec {}

#[sealed::sealed]
trait MetadataKindSpec {}

/// Marker for types with a constant size known at compile time.
///
/// See [`core::marker::Sized`].
pub struct Sized<K>(core::marker::PhantomData<K>, Infallible);

/// Marker for types with a size that can be determined from pointer metadata.
///
/// Type parameter can be set to either [`SliceLike`] or [`DynTraitLike`] only.
///
/// See [`core::marker::MetaSized`]
pub struct MetaSized<K>(core::marker::PhantomData<K>, Infallible);

/// Marker for types that do not contribute storage to an ABI layout.
pub enum Zst {}

/// Marker for types that contribute storage to an ABI layout.
pub enum NonZst {}

/// Marker for [Slices](https://doc.rust-lang.org/core/primitive.slice.html) or DSTs whose last field is a slice.
///
/// See [`core::slice`].
pub enum SliceLike {}

/// Marker for [Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html) or DSTs whose last field is a trait object.
pub enum DynTraitLike {}

/// Marker for [Extern types](https://rust-lang.github.io/rfcs/1861-extern-types.html) and DSTs whose last field is an extern type.
///
/// Pointers to extern types are thin.
pub enum ExternTypeLike {}

#[sealed::sealed]
impl SizedKindSpec for Zst {}

#[sealed::sealed]
impl SizedKindSpec for NonZst {}

#[sealed::sealed]
impl MetadataKindSpec for SliceLike {}

#[sealed::sealed]
impl MetadataKindSpec for DynTraitLike {}

#[sealed::sealed]
impl<K: SizedKindSpec> SizeSpec for Sized<K> {}

#[sealed::sealed]
impl<K: MetadataKindSpec> SizeSpec for MetaSized<K> {}

#[sealed::sealed]
impl SizeSpec for ExternTypeLike {}

/// Pointers to types implementing this trait alias are “thin”.
///
/// See [`core::ptr::Thin`]
pub(crate) trait Thin {}
impl<K> Thin for Sized<K> {}
impl Thin for ExternTypeLike {}

pub(crate) trait Dst {}
impl Dst for ExternTypeLike {}
impl<K> Dst for MetaSized<K> {}

impl<K> Add<Sized<K>> for Sized<Zst> {
    type Output = Sized<K>;

    fn add(self, _: Sized<K>) -> Self::Output {
        unreachable!()
    }
}

impl<K> Add<Sized<K>> for Sized<NonZst> {
    type Output = Self;

    fn add(self, _: Sized<K>) -> Self::Output {
        unreachable!()
    }
}

impl<K: Dst, U> Add<K> for Sized<U> {
    type Output = K;

    fn add(self, _: K) -> Self::Output {
        unreachable!()
    }
}

impl<K, U> Add<Sized<U>> for MetaSized<K> {
    type Output = Self;

    fn add(self, _: Sized<U>) -> Self::Output {
        unreachable!()
    }
}

impl<U> Add<Sized<U>> for ExternTypeLike {
    type Output = Self;

    fn add(self, _: Sized<U>) -> Self::Output {
        unreachable!()
    }
}
