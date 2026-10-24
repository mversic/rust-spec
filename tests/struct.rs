use core::{cell::UnsafeCell, num::NonZero as StdNonZero, num::NonZeroU8, ops::Add};
use std::ffi::{CStr, c_void};

use rust_spec::{
    RustSpec, Stable, Unstable,
    drop::{AutoDrop, CustomDrop, NoDrop},
    layout::{NonRobust, Robust},
    mutability::{Exclusive, WithInterior},
    niche::{WithNiche, WithoutNiche},
    size::{self, Gt, MetaSized, NulTerminated, Sized as SpecSized, SliceLike, Zero},
};
use static_assertions::{assert_impl_all, assert_not_impl_any};

pub trait Projection {
    type Kita<'a>
    where
        Self: 'a;
}

impl Projection for c_void {
    type Kita<'a>
        = &'a u8
    where
        Self: 'a;
}

pub trait CTypeProjection {
    type CType;
}

impl CTypeProjection for u8 {
    type CType = u8;
}

impl<T: CTypeProjection> CTypeProjection for UnsafeCell<T> {
    type CType = T::CType;
}

#[derive(RustSpec)]
pub struct ParamReprCZst<T: ?Sized> {
    pub a: (),
    pub b: T,
}

#[derive(RustSpec)]
pub struct NulTerminatedTail {
    pub prefix: u32,
    pub tail: CStr,
}

#[test]
fn nul_terminated_tail_classification() {
    assert_impl_all!(NulTerminatedTail: RustSpec<Size = NulTerminated>);
}

#[derive(RustSpec)]
pub struct WithGat1<'b>(pub <c_void as Projection>::Kita<'b>)
where
    for<'d> c_void: Projection,
    Self: 'b;

#[derive(RustSpec)]
#[repr(C)]
pub struct WithGat2<'b>(pub <u8 as Projection>::Kita<'b>)
where
    for<'d> u8: Projection,
    Self: 'b;

#[derive(RustSpec)]
#[repr(transparent)]
pub struct Custom2View<'_dšč, 'a>(pub <&'a i8 as Projection>::Kita<'_dšč>)
where
    Self: '_dšč,
    for<'_dummy> &'a i8: Projection;

#[derive(RustSpec)]
#[repr(transparent)]
pub struct CounterData(<usize as CTypeProjection>::CType)
where
    for<'__dummy> usize: CTypeProjection;

#[derive(RustSpec)]
#[repr(C)]
pub struct OpaqueData(<usize as CTypeProjection>::CType)
where
    for<'__dummy> usize: CTypeProjection;

#[derive(RustSpec)]
struct Empty;

#[derive(RustSpec)]
pub struct RustStruct {
    pub value: u8,
}

#[derive(RustSpec)]
pub struct NonRobustRustRepr {
    pub value: bool,
}

#[derive(RustSpec)]
#[repr(C)]
struct ReprCStruct {
    value: u8,
}

#[derive(RustSpec)]
#[repr(C)]
struct ReprCProjectionCTypeFields {
    first: <UnsafeCell<u8> as CTypeProjection>::CType,
    second: <u8 as CTypeProjection>::CType,
}

#[derive(RustSpec)]
pub struct InteriorWrapper {
    _left: UnsafeCell<u8>,
    pub right: UnsafeCell<u16>,
}

#[derive(RustSpec)]
pub struct MixedMutability {
    pub cell: UnsafeCell<u8>,
    pub value: u8,
}

#[derive(RustSpec)]
pub struct Wrapper<T> {
    pub value: T,
}

#[derive(RustSpec)]
#[repr(C)]
struct Pair<T, U> {
    first: T,
    second: U,
}

#[derive(RustSpec)]
#[rust_spec(custom_niche)]
#[allow(dead_code)]
struct AssertedCustomNiche(u8);

#[test]
fn custom_niche_is_a_trap_value() {
    assert_impl_all!(AssertedCustomNiche: RustSpec<
        Trap = NonRobust,
        Niche = WithNiche<Unstable>,
    >);
}

#[derive(RustSpec)]
pub struct SingletonWithNiche(pub NonZeroU8);

#[derive(RustSpec)]
#[repr(transparent)]
struct TransparentWithNiche(NonZeroU8);

#[derive(RustSpec)]
#[repr(transparent)]
struct TransparentWithMultipleFields((), NonZeroU8);

#[derive(RustSpec)]
#[repr(transparent)]
struct TransparentZstStruct;

struct NotRustSpec;

#[derive(RustSpec)]
#[repr(transparent)]
struct Bytes([u8]);

#[derive(RustSpec)]
pub struct Packet {
    pub tag: NonZeroU8,
    pub payload: [u8],
}

#[derive(RustSpec)]
pub struct TuplePacket(pub NonZeroU8, pub [u8]);

#[derive(RustSpec)]
#[repr(transparent)]
pub struct TransparentSlice<T>([T]);

#[derive(RustSpec)]
pub struct NoDropStruct(pub u8);

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
pub struct CustomDropStruct(pub u8);

impl Drop for CustomDropStruct {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
pub struct InnerDropStruct(pub CustomDropStruct);

#[derive(RustSpec)]
pub struct DropFirst(pub CustomDropStruct, pub u8);

#[derive(RustSpec)]
pub struct DropSecond(pub u8, pub CustomDropStruct);

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
pub struct CustomAndInnerDropStruct(pub CustomDropStruct);

impl Drop for CustomAndInnerDropStruct {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
pub struct CustomWithInnerDropStruct(pub InnerDropStruct);

impl Drop for CustomWithInnerDropStruct {
    fn drop(&mut self) {}
}

#[test]
fn struct_drop_classification() {
    assert_impl_all!(NoDropStruct: RustSpec<Drop = NoDrop>);
    assert_impl_all!(CustomDropStruct: RustSpec<Drop = CustomDrop<NoDrop>>);
    assert_impl_all!(InnerDropStruct: RustSpec<Drop = AutoDrop>);
    assert_impl_all!(DropFirst: RustSpec<Drop = AutoDrop>);
    assert_impl_all!(DropSecond: RustSpec<Drop = AutoDrop>);
    assert_impl_all!(CustomAndInnerDropStruct: RustSpec<Drop = CustomDrop<AutoDrop>>);
    assert_impl_all!(CustomWithInnerDropStruct: RustSpec<Drop = CustomDrop<AutoDrop>>);
    assert_impl_all!(Option<CustomAndInnerDropStruct>: RustSpec<Drop = AutoDrop>);
    assert_impl_all!((CustomDropStruct, CustomDropStruct): RustSpec<Drop = AutoDrop>);
    assert_impl_all!(Result<CustomDropStruct, u8>: RustSpec<Drop = AutoDrop>);
    assert_impl_all!(Result<u8, CustomDropStruct>: RustSpec<Drop = AutoDrop>);
    assert_impl_all!(Result<CustomDropStruct, ()>: RustSpec<Drop = AutoDrop>);
    assert_impl_all!(Result<(), CustomDropStruct>: RustSpec<Drop = AutoDrop>);
    assert_impl_all!(Result<CustomDropStruct, CustomDropStruct>: RustSpec<Drop = AutoDrop>);
    assert_impl_all!(Result<NoDropStruct, u8>: RustSpec<Drop = NoDrop>);
    assert_impl_all!(Box<u8>: RustSpec<Drop = CustomDrop<NoDrop>>);
    assert_impl_all!(Box<CustomDropStruct>: RustSpec<Drop = CustomDrop<AutoDrop>>);
    assert_impl_all!(Box<CustomAndInnerDropStruct>: RustSpec<Drop = CustomDrop<AutoDrop>>);
    assert_impl_all!(Vec<u8>: RustSpec<Drop = CustomDrop<NoDrop>>);
    assert_impl_all!(Vec<CustomDropStruct>: RustSpec<Drop = CustomDrop<AutoDrop>>);
}

fn assert_rust_spec<T: RustSpec + ?Sized>() {}

fn a_slice_of_any_rust_spec_type_is_rust_spec<T: RustSpec>()
where
    rust_spec::drop::NoDrop: Add<T::Drop>,
{
    assert_rust_spec::<[T]>();
}

fn a_transparent_slice_wrapper_of_any_rust_spec_type_is_rust_spec<T: RustSpec>()
where
    WithoutNiche: Add<T::Niche>,
    rust_spec::drop::NoDrop: Add<T::Drop>,
    rust_spec::drop::NoDrop: Add<<rust_spec::drop::NoDrop as Add<T::Drop>>::Output>,
{
    assert_rust_spec::<TransparentSlice<T>>();
}

#[test]
fn generic_slice_classification() {
    a_slice_of_any_rust_spec_type_is_rust_spec::<u8>();
    a_transparent_slice_wrapper_of_any_rust_spec_type_is_rust_spec::<u8>();
}

#[test]
fn struct_classification() {
    assert_impl_all!(ParamReprCZst<()>:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Zero>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ParamReprCZst<UnsafeCell<NonZeroU8>>:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithoutNiche,
            Mutability = WithInterior,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ParamReprCZst<u8>:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );

    assert_impl_all!(SingletonWithNiche:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );

    assert_impl_all!(TransparentWithNiche:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Stable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(TransparentWithMultipleFields:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(TransparentZstStruct:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Zero>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );

    assert_impl_all!(Empty:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Zero>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(RustStruct:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(NonRobustRustRepr:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ReprCStruct:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ReprCProjectionCTypeFields:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(InteriorWrapper:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::Gt<rust_spec::One>,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = WithInterior,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(MixedMutability:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = WithInterior,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Pair<UnsafeCell<u8>, UnsafeCell<u16>>:
        RustSpec<Mutability = WithInterior>,
    );
    assert_impl_all!(Pair<UnsafeCell<u8>, u8>:
        RustSpec<Mutability = WithInterior>,
    );
    assert_impl_all!(Wrapper<u8>:
        RustSpec<
            Layout = Unstable,
            Size = size::Sized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Wrapper<UnsafeCell<u8>>:
        RustSpec<
            Layout = Unstable,
            Size = size::Sized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = WithInterior,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Pair<u8, StdNonZero<u8>>:
        RustSpec<
            Layout = Stable,
            Size = size::Sized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Pair<bool, u8>:
        RustSpec<
            Layout = Stable,
            Size = size::Sized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(*mut u8:
        RustSpec<
            Layout = Stable,
            Size = size::Sized<Gt<Zero>>,
            Alignment = <usize as RustSpec>::Alignment,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
}

#[test]
fn struct_wide_classification() {
    assert_impl_all!(Bytes:
        RustSpec<
            Layout = Stable,
            Size = MetaSized<SliceLike>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Packet:
        RustSpec<
            Layout = Unstable,
            Size = MetaSized<SliceLike>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(TuplePacket:
        RustSpec<
            Layout = Unstable,
            Size = MetaSized<SliceLike>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
}

#[test]
fn generic_raw_pointer_bounds_follow_pointee_shape() {
    assert_impl_all!(*const bool: RustSpec<Layout = Stable, Trap = Robust>);
    assert_impl_all!(*const [bool]: RustSpec<Layout = Unstable, Trap = Robust>);
    assert_impl_all!(*mut [bool]: RustSpec<Layout = Unstable, Trap = Robust>);
    assert_impl_all!(*const *mut [bool]: RustSpec<Layout = Stable, Trap = Robust>);
    assert_not_impl_any!(*const NotRustSpec: RustSpec);
}

#[test]
fn nul_terminated_raw_pointers() {
    fn check<
        P: RustSpec<
                Layout = Unstable,
                Size = size::Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Drop = NoDrop,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
    >() {
    }

    check::<*const CStr>();
    check::<*mut CStr>();
    check::<*const NulTerminatedTail>();
    check::<*mut NulTerminatedTail>();
}

mod tail_classification {
    #![allow(dead_code)]

    use rust_spec::{
        Gt, RustSpec, Stable, Zero,
        layout::Robust,
        size::{MetaSized, Sized as SpecSized, SliceLike},
    };
    use static_assertions::assert_impl_all;

    #[derive(RustSpec)]
    #[repr(C)]
    struct SizedTail<T> {
        prefix: (),
        tail: T,
    }

    #[derive(RustSpec)]
    #[repr(C)]
    struct SliceTail<T> {
        prefix: u8,
        tail: [T],
    }

    #[derive(RustSpec)]
    struct TupleTail<T: ?Sized> {
        tail: (u8, T),
    }

    #[derive(RustSpec)]
    struct ZeroTail {
        tail: (),
    }

    #[derive(RustSpec)]
    #[repr(C)]
    struct ZeroFields {
        first: [u8; 0],
        tail: (),
    }

    #[derive(RustSpec)]
    #[repr(C)]
    struct NonzeroPrefix {
        first: u8,
        tail: (),
    }

    #[test]
    fn tail_size_classification() {
        assert_impl_all!(SizedTail<u8>: RustSpec<Size = SpecSized<Gt<Zero>>>);
        assert_impl_all!(SizedTail<()>: RustSpec<Size = SpecSized<Zero>>);
        assert_impl_all!(ZeroTail: RustSpec<Size = SpecSized<Zero>>);
        assert_impl_all!(ZeroFields: RustSpec<Size = SpecSized<Zero>>);
        assert_impl_all!(NonzeroPrefix: RustSpec<Size = SpecSized<Gt<Zero>>>);
        assert_impl_all!(SliceTail<u8>: RustSpec<Size = MetaSized<SliceLike>>);
        assert_impl_all!(TupleTail<[u8]>: RustSpec<Size = MetaSized<SliceLike>>);
    }

    #[test]
    fn primitive_spelling_does_not_override_field_size() {
        #[allow(non_camel_case_types)]
        type u8 = ();

        #[derive(RustSpec)]
        struct Shadowed {
            tail: u8,
        }

        assert_impl_all!(Shadowed: RustSpec<Size = SpecSized<Zero>>);
    }

    // A named wrapper owns the classification of its type arguments.
    struct Unclassified;
    struct Hidden<T>(T);
    unsafe impl RustSpec for Hidden<*mut Unclassified> {
        type Layout = Stable;
        type Size = SpecSized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = Robust;
        type Niche = rust_spec::niche::WithoutNiche;
        type Drop = rust_spec::drop::NoDrop;
        type Mutability = rust_spec::mutability::Exclusive;
        type Opacity = rust_spec::NonOpaque;
        type __IndirectTrap = Robust;
    }

    #[derive(RustSpec)]
    #[repr(C)]
    struct ContainsHidden {
        field: Hidden<*mut Unclassified>,
    }

    #[test]
    fn named_wrapper_arguments_are_not_inspected() {
        assert_impl_all!(ContainsHidden: RustSpec<Layout = Stable, Trap = Robust>);
    }
}

mod pointer_classification {
    #![allow(dead_code)]
    use core::ptr::NonNull;
    use rust_spec::{
        RustSpec, Stable, Unstable,
        drop::NoDrop,
        mutability::Exclusive,
        niche::{WithNiche, WithoutNiche},
    };
    use static_assertions::assert_impl_all;

    #[derive(RustSpec)]
    #[repr(C)]
    struct Tail<T: ?Sized> {
        prefix: u8,
        tail: T,
    }
    #[derive(RustSpec)]
    #[repr(transparent)]
    struct Pointer<T: ?Sized>(NonNull<T>);

    trait Object {}
    impl Object for u32 {}
    unsafe impl RustSpec for dyn Object + '_ {
        type Layout = Unstable;
        type Size = rust_spec::size::MetaSized<rust_spec::size::DynTraitLike>;
        type Alignment = rust_spec::Gt<rust_spec::One>;
        type Trap = rust_spec::layout::NonRobust;
        type Niche = WithoutNiche;
        type Drop = NoDrop;
        type Mutability = Exclusive;
        type Opacity = rust_spec::NonOpaque;
        type __IndirectTrap = rust_spec::layout::Robust;
    }

    #[test]
    fn nonnull_layout_follows_pointer_shape() {
        assert_impl_all!(NonNull<u8>: RustSpec<Layout = Stable, Niche = WithNiche<Stable>>);
        assert_impl_all!(Option<NonNull<u8>>: RustSpec<Layout = Stable>);
        macro_rules! check {
            ($($ty:ty),*) => {$(
                assert_impl_all!(NonNull<$ty>: RustSpec<Layout = Unstable, Niche = WithNiche<Stable>>);
                assert_impl_all!(Option<NonNull<$ty>>: RustSpec<Layout = Unstable>);
                assert_impl_all!(Pointer<$ty>: RustSpec<Layout = Unstable>);
            )*};
        }
        check!([u8], str, dyn Object, Tail<[u8]>, Tail<dyn Object>);
    }
}

mod opacity {
    #![allow(dead_code)]
    use core::{cell::Cell, marker::PhantomData, mem::MaybeUninit};
    use rust_spec::{NonOpaque, Opaque, RustSpec};
    use static_assertions::assert_impl_all;

    #[derive(RustSpec)]
    #[rust_spec(opaque)]
    struct Hidden(*mut u8);

    #[derive(RustSpec)]
    struct Pair<A, B>(A, B);

    #[derive(RustSpec)]
    #[repr(transparent)]
    struct Wrapper<T>(T);

    #[test]
    fn opacity_is_explicit_and_does_not_propagate() {
        assert_impl_all!(Hidden: RustSpec<Opacity = Opaque>);
        assert_impl_all!(Pair<Hidden, u8>: RustSpec<Opacity = NonOpaque>);
        assert_impl_all!(Pair<u8, Hidden>: RustSpec<Opacity = NonOpaque>);
        assert_impl_all!(Wrapper<Hidden>: RustSpec<Opacity = NonOpaque>);
        assert_impl_all!((Hidden, u8): RustSpec<Opacity = NonOpaque>);
        assert_impl_all!([Hidden; 2]: RustSpec<Opacity = NonOpaque>);
        assert_impl_all!(Cell<Hidden>: RustSpec<Opacity = NonOpaque>);
        assert_impl_all!(MaybeUninit<Hidden>: RustSpec<Opacity = NonOpaque>);
        assert_impl_all!(&Hidden: RustSpec<Opacity = NonOpaque>);
        assert_impl_all!(PhantomData<Hidden>: RustSpec<Opacity = NonOpaque>);
    }
}
