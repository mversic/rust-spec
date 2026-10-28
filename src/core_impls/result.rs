//! Result representation selected by the two variants' size categories.

use core::ops::Add;

use crate::{
    Gt, Max, NonOpaque, RustSpec, Stable, Sum, Unstable, Zero,
    layout::{LayoutKind, NonRobust, Robust},
    niche::{NullableRepresentation, WithNiche, WithoutNiche},
    size::{self, SizedKind},
};

/// Representation contributions from the alignment of an empty alternative.
///
/// Combine these with [NullableRepresentation] using [Add]. Alignment one
/// preserves the nullable classification; greater alignment prevents relying
/// on the payload's guaranteed niche representation.
#[sealed::sealed]
pub trait NullableAlignment {
    type Layout: LayoutKind;
    type Trap;
    type Niche;
}

#[sealed::sealed]
impl NullableAlignment for crate::One {
    type Layout = Stable;
    type Trap = Robust;
    type Niche = WithoutNiche;
}

#[sealed::sealed]
impl NullableAlignment for crate::Gt<crate::One> {
    type Layout = Unstable;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
}

// Public because its associated types appear in the public RustSpec impl.
// The containing module keeps this dispatch machinery internal.
pub trait ResultRepresentation<R: RustSpec, E: RustSpec> {
    type Layout;
    type Trap;
    type Niche;
    type IndirectTrap;
}

impl<R, E, K: SizedKind> ResultRepresentation<R, E> for (size::Sized<K>, size::Sized<K>)
where
    R: RustSpec<__IndirectTrap: Add<E::__IndirectTrap>>,
    E: RustSpec,
{
    type Layout = Unstable;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
    type IndirectTrap = Sum<R::__IndirectTrap, E::__IndirectTrap>;
}

impl<R, E> ResultRepresentation<R, E> for (size::Sized<Gt<Zero>>, size::Sized<Zero>)
where
    R: RustSpec<Niche: NullableRepresentation>,
    E: RustSpec<__Alignment: NullableAlignment>,
    <E::__Alignment as NullableAlignment>::Layout:
        Add<<R::Niche as NullableRepresentation>::Layout>,
    Sum<
        <E::__Alignment as NullableAlignment>::Layout,
        <R::Niche as NullableRepresentation>::Layout,
    >: Add<R::Layout>,
    <E::__Alignment as NullableAlignment>::Trap: Add<<R::Niche as NullableRepresentation>::Trap>,
    Sum<<E::__Alignment as NullableAlignment>::Trap, <R::Niche as NullableRepresentation>::Trap>:
        Add<R::__IndirectTrap>,
    <E::__Alignment as NullableAlignment>::Niche: Add<<R::Niche as NullableRepresentation>::Niche>,
{
    type Layout = Sum<
        Sum<
            <E::__Alignment as NullableAlignment>::Layout,
            <R::Niche as NullableRepresentation>::Layout,
        >,
        R::Layout,
    >;
    type Trap = Sum<
        Sum<
            <E::__Alignment as NullableAlignment>::Trap,
            <R::Niche as NullableRepresentation>::Trap,
        >,
        R::__IndirectTrap,
    >;
    type Niche = Sum<
        <E::__Alignment as NullableAlignment>::Niche,
        <R::Niche as NullableRepresentation>::Niche,
    >;
    type IndirectTrap = R::__IndirectTrap;
}

impl<R, E> ResultRepresentation<R, E> for (size::Sized<Zero>, size::Sized<Gt<Zero>>)
where
    E: RustSpec<Niche: NullableRepresentation>,
    R: RustSpec<__Alignment: NullableAlignment>,
    <R::__Alignment as NullableAlignment>::Layout:
        Add<<E::Niche as NullableRepresentation>::Layout>,
    Sum<
        <R::__Alignment as NullableAlignment>::Layout,
        <E::Niche as NullableRepresentation>::Layout,
    >: Add<E::Layout>,
    <R::__Alignment as NullableAlignment>::Trap: Add<<E::Niche as NullableRepresentation>::Trap>,
    Sum<<R::__Alignment as NullableAlignment>::Trap, <E::Niche as NullableRepresentation>::Trap>:
        Add<E::__IndirectTrap>,
    <R::__Alignment as NullableAlignment>::Niche: Add<<E::Niche as NullableRepresentation>::Niche>,
{
    type Layout = Sum<
        Sum<
            <R::__Alignment as NullableAlignment>::Layout,
            <E::Niche as NullableRepresentation>::Layout,
        >,
        E::Layout,
    >;
    type Trap = Sum<
        Sum<
            <R::__Alignment as NullableAlignment>::Trap,
            <E::Niche as NullableRepresentation>::Trap,
        >,
        E::__IndirectTrap,
    >;
    type Niche = Sum<
        <R::__Alignment as NullableAlignment>::Niche,
        <E::Niche as NullableRepresentation>::Niche,
    >;
    type IndirectTrap = E::__IndirectTrap;
}

// SAFETY: The representation helper handles equal-sized categories and either
// nullable orientation. The remaining axes combine both owned alternatives.
unsafe impl<R, E> RustSpec for Result<R, E>
where
    R: RustSpec<
            __Alignment: Max<E::__Alignment>,
            Drop: Add<E::Drop>,
            Mutability: Add<E::Mutability>,
        >,
    E: RustSpec,
    (R::Size, E::Size): ResultRepresentation<R, E>,
{
    type Layout = <(R::Size, E::Size) as ResultRepresentation<R, E>>::Layout;
    type Size = size::Sized<Gt<Zero>>;
    type __Alignment = <R::__Alignment as Max<E::__Alignment>>::Output;
    type Trap = <(R::Size, E::Size) as ResultRepresentation<R, E>>::Trap;
    type Niche = <(R::Size, E::Size) as ResultRepresentation<R, E>>::Niche;
    type Drop = Sum<R::Drop, E::Drop>;
    type Mutability = Sum<R::Mutability, E::Mutability>;
    type Opacity = NonOpaque;
    type __IndirectTrap = <(R::Size, E::Size) as ResultRepresentation<R, E>>::IndirectTrap;
}
