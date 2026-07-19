#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use core::ptr::NonNull;
use core::{convert::Infallible, ops::Add};

/// Marker for types with a size that can be determined from pointer metadata.
///
/// Type parameter can be set to either [`SliceLike`] or [`DynTraitLike`] only.
///
/// See [`core::marker::MetaSized`]
pub struct MetaSized<K>(core::marker::PhantomData<K>, Infallible);

/// Marker for types with a constant size known at compile time.
///
/// See [`core::marker::Sized`].
pub struct Sized<K>(core::marker::PhantomData<K>, Infallible);

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

/// Pointers to types implementing this trait alias are “thin”.
///
/// See [`core::ptr::Thin`]
pub(crate) trait Thin {}
impl<K> Thin for Sized<K> {}
impl Thin for ExternTypeLike {}

pub(crate) trait Dst {}
impl Dst for ExternTypeLike {}
impl<K> Dst for MetaSized<K> {}

/// Pointer that consists of data and metadata (also called a `fat` pointer).
///
/// This includes slices, trait objects, and DSTs whose last field is one of aformentioned.
pub trait Wide {
    /// Data component of a pointer.
    type Data;

    /// Metadata component of a pointer.
    type Metadata;

    /// Extracts the metadata component of a pointer.
    fn metadata(&self) -> Self::Metadata;

    /// Returns a raw pointer to the underlying data.
    fn as_ptr(&self) -> *const Self::Data;

    /// Returns an unsafe mutable pointer to the underlying data.
    fn as_mut_ptr(&mut self) -> *mut Self::Data;

    /// Consumes the `Box`, returning a wrapped `NonNull` pointer.
    ///
    /// See [`Box::into_non_null`]
    #[cfg(feature = "alloc")]
    fn into_non_null(self: Box<Self>) -> NonNull<Self::Data>;

    /// Forms a wide reference from a data pointer and metadata.
    ///
    /// # Safety
    ///
    /// See [`core::ptr::from_raw_parts`]
    unsafe fn from_raw_parts<'a>(data: *const Self::Data, metadata: Self::Metadata) -> &'a Self;

    /// Performs the same functionality as [`Self::from_raw_parts`], except that a mutable reference is returned.
    ///
    /// # Safety
    ///
    /// See [`core::ptr::from_raw_parts_mut`]
    unsafe fn from_raw_parts_mut<'a>(
        data: *mut Self::Data,
        metadata: Self::Metadata,
    ) -> &'a mut Self;

    /// Constructs a box from a `NonNull` pointer.
    ///
    /// # Safety
    ///
    /// See [`Box::from_non_null`]
    #[cfg(feature = "alloc")]
    unsafe fn from_non_null(data: NonNull<Self::Data>, metadata: Self::Metadata) -> Box<Self>;
}

impl<R> Wide for [R] {
    type Data = R;
    type Metadata = usize;

    fn metadata(&self) -> Self::Metadata {
        Self::len(self)
    }

    fn as_ptr(&self) -> *const Self::Data {
        Self::as_ptr(self)
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Data {
        Self::as_mut_ptr(self)
    }

    #[cfg(feature = "alloc")]
    fn into_non_null(self: Box<Self>) -> NonNull<Self::Data> {
        let ptr = Box::into_raw(self).cast::<Self::Data>();
        unsafe { NonNull::new_unchecked(ptr) }
    }

    unsafe fn from_raw_parts<'a>(data: *const Self::Data, len: Self::Metadata) -> &'a Self {
        unsafe { core::slice::from_raw_parts(data, len) }
    }

    unsafe fn from_raw_parts_mut<'a>(data: *mut Self::Data, len: Self::Metadata) -> &'a mut Self {
        unsafe { core::slice::from_raw_parts_mut(data, len) }
    }

    #[cfg(feature = "alloc")]
    unsafe fn from_non_null(data: NonNull<Self::Data>, len: Self::Metadata) -> Box<Self> {
        unsafe { Box::from_raw(core::ptr::slice_from_raw_parts_mut(data.as_ptr(), len)) }
    }
}

impl Wide for str {
    type Data = u8;
    type Metadata = usize;

    fn metadata(&self) -> Self::Metadata {
        Self::len(self)
    }

    fn as_ptr(&self) -> *const Self::Data {
        self.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Data {
        self.as_mut_ptr()
    }

    #[cfg(feature = "alloc")]
    fn into_non_null(self: Box<Self>) -> NonNull<Self::Data> {
        // TODO: Use Box::into_non_null when available in stable
        self.into_boxed_bytes().into_non_null()
    }

    unsafe fn from_raw_parts<'a>(data: *const Self::Data, len: Self::Metadata) -> &'a Self {
        let slice = unsafe { <[u8]>::from_raw_parts(data, len) };
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

    unsafe fn from_raw_parts_mut<'a>(data: *mut Self::Data, len: Self::Metadata) -> &'a mut Self {
        let slice = unsafe { <[u8]>::from_raw_parts_mut(data, len) };
        unsafe { core::str::from_utf8_unchecked_mut(slice) }
    }

    #[cfg(feature = "alloc")]
    unsafe fn from_non_null(data: NonNull<Self::Data>, len: Self::Metadata) -> Box<Self> {
        // TODO: Use Box::from_non_null once available on stable
        let slice = unsafe { <[u8]>::from_non_null(data, len) };
        let slice = Box::into_raw(slice);
        let str = unsafe { core::str::from_utf8_unchecked_mut(&mut *slice) };

        unsafe { Box::from_raw(str) }
    }
}

impl Add<Sized<Zst>> for Sized<Zst> {
    type Output = Self;

    fn add(self, _: Sized<Zst>) -> Self::Output {
        unreachable!()
    }
}

impl Add<Sized<Zst>> for Sized<NonZst> {
    type Output = Sized<NonZst>;

    fn add(self, _: Sized<Zst>) -> Self::Output {
        unreachable!()
    }
}

impl<U> Add<Sized<NonZst>> for Sized<U> {
    type Output = Sized<NonZst>;

    fn add(self, _: Sized<NonZst>) -> Self::Output {
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
