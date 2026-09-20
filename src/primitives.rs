//! Logic related to the conversion of primitives to and from FFI-compatible representation

use core::{cmp::Ordering, ops::Add};

use crate::{
    RustSpec, Stable, Unstable, Zero,
    layout::{NonRobust, Robust},
    mutability::Exclusive,
    niche::{WithNiche, WithoutNiche},
    size::{self, MetaSized, SliceLike},
};

macro_rules! primitive_derive {
    ( $primitive:ty => $alignment:ty ) => {
        unsafe impl RustSpec for $primitive {
            type Layout = Stable;
            type Size = size::Sized<crate::Gt<Zero>>;
            type Alignment = $alignment;
            type Trap = Robust;
            type Niche = WithoutNiche;
            type Mutability = Exclusive;
            type __IndirectTrap = Robust;
        }
    };
}

macro_rules! raw_pointer_derive {
    ( $mutability:tt ) => {
        unsafe impl<R: ?Sized> RustSpec for *$mutability R {
            type Layout = Stable;
            type Size = size::Sized<crate::Gt<Zero>>;
            type Alignment = <usize as RustSpec>::Alignment;
            type Trap = Robust;
            type Niche = WithoutNiche;
            type Mutability = Exclusive;
            type __IndirectTrap = Robust;

        }
    };
}

macro_rules! impl_nonempty_array {
    ($($n:literal),* $(,)?) => { $(
        unsafe impl<R: RustSpec> RustSpec for [R; $n]
        where
            WithoutNiche: Add<R::Niche>,
        {
            type Layout = R::Layout;
            type Size = R::Size;
            type Alignment = R::Alignment;
            type Trap = R::Trap;
            type Niche = <WithoutNiche as Add<R::Niche>>::Output;
            type Mutability = R::Mutability;
            type __IndirectTrap = R::__IndirectTrap;
        })*
    };
}

macro_rules! impl_fn_types {
    ( $( ( $( $arg:ident ),* ) ),* $(,)? ) => {
        $(
            impl_fn_types!(@rust fn($($arg),*) -> R; $($arg),*);
            impl_fn_types!(@rust unsafe fn($($arg),*) -> R; $($arg),*);
            impl_fn_types!(@abi "C"; $($arg),*);
            impl_fn_types!(@abi "C-unwind"; $($arg),*);
            impl_fn_types!(@abi "system"; $($arg),*);
            impl_fn_types!(@abi "system-unwind"; $($arg),*);
            #[cfg(target_arch = "x86")]
            impl_fn_types!(@abi "cdecl"; $($arg),*);
            #[cfg(target_arch = "x86")]
            impl_fn_types!(@abi "cdecl-unwind"; $($arg),*);
            #[cfg(target_arch = "x86")]
            impl_fn_types!(@abi "stdcall"; $($arg),*);
            #[cfg(target_arch = "x86")]
            impl_fn_types!(@abi "stdcall-unwind"; $($arg),*);
            #[cfg(target_arch = "x86")]
            impl_fn_types!(@abi "fastcall"; $($arg),*);
            #[cfg(target_arch = "x86")]
            impl_fn_types!(@abi "fastcall-unwind"; $($arg),*);
            #[cfg(target_arch = "x86")]
            impl_fn_types!(@abi "thiscall"; $($arg),*);
            #[cfg(target_arch = "x86")]
            impl_fn_types!(@abi "thiscall-unwind"; $($arg),*);
            #[cfg(target_arch = "x86_64")]
            impl_fn_types!(@abi "sysv64"; $($arg),*);
            #[cfg(target_arch = "x86_64")]
            impl_fn_types!(@abi "sysv64-unwind"; $($arg),*);
            #[cfg(target_arch = "x86_64")]
            impl_fn_types!(@abi "win64"; $($arg),*);
            #[cfg(target_arch = "x86_64")]
            impl_fn_types!(@abi "win64-unwind"; $($arg),*);
            #[cfg(target_arch = "arm")]
            impl_fn_types!(@abi "aapcs"; $($arg),*);
            #[cfg(target_arch = "arm")]
            impl_fn_types!(@abi "aapcs-unwind"; $($arg),*);
            #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
            impl_fn_types!(@abi "efiapi"; $($arg),*);
        )*
    };
    (@abi $abi:literal; $($arg:ident),*) => {
        impl_fn_types!(@c extern $abi fn($($arg),*) -> R; [] [<R as RustSpec>::Layout]; $($arg),*);
        impl_fn_types!(@c unsafe extern $abi fn($($arg),*) -> R; [] [<R as RustSpec>::Layout]; $($arg),*);
        const _: () = {
            assert!(core::mem::align_of::<extern $abi fn()>() > 1);
            assert!(core::mem::align_of::<unsafe extern $abi fn()>() > 1);
        };
    };
    (@rust $fn_type:ty; $($arg:ident),*) => {
        impl_fn_types!(@impl $fn_type; [$($arg,)* R] [Unstable]);
    };
    (@c $fn_type:ty; [$($params:tt)*] [$layout:ty]; $next:ident $(, $rest:ident)*) => {
        impl_fn_types!(@c $fn_type;
            [$($params)* $next: RustSpec<Layout: Add<$layout>>,]
            [<<$next as RustSpec>::Layout as Add<$layout>>::Output]; $($rest),*);
    };
    (@c $fn_type:ty; [$($params:tt)*] [$layout:ty];) => {
        impl_fn_types!(@impl $fn_type; [$($params)* R: RustSpec] [$layout]);
    };
    (@impl $fn_type:ty; [$($params:tt)*] [$layout:ty]) => {
        unsafe impl<$($params)*> RustSpec for $fn_type {
            type Layout = $layout;
            type Size = size::Sized<crate::Gt<Zero>>;
            type Alignment = <usize as RustSpec>::Alignment;
            type Trap = NonRobust;
            type Niche = WithNiche<Stable>;
            type Mutability = Exclusive;
            type __IndirectTrap = Robust;
        }
    };
}

macro_rules! fieldless_enum_derive {
    ( $src:ty => $alignment:ty ) => {
        unsafe impl RustSpec for $src {
            type Layout = Stable;
            type Size = size::Sized<crate::Gt<Zero>>;
            type Alignment = $alignment;
            type Trap = NonRobust;
            type Niche = WithNiche<Unstable>;
            type Mutability = Exclusive;
            type __IndirectTrap = Robust;
        }
    };
}

primitive_derive! { u8 => crate::One }
primitive_derive! { i8 => crate::One }
primitive_derive! { u16 => crate::Gt<crate::One> }
primitive_derive! { i16 => crate::Gt<crate::One> }
primitive_derive! { u32 => crate::Gt<crate::One> }
primitive_derive! { i32 => crate::Gt<crate::One> }
primitive_derive! { u64 => crate::Gt<crate::One> }
primitive_derive! { i64 => crate::Gt<crate::One> }
primitive_derive! { u128 => crate::Gt<crate::One> }
primitive_derive! { i128 => crate::Gt<crate::One> }
primitive_derive! { f32 => crate::Gt<crate::One> }
primitive_derive! { f64 => crate::Gt<crate::One> }

#[cfg(target_pointer_width = "16")]
primitive_derive! { usize => crate::Gt<crate::One> }
#[cfg(target_pointer_width = "32")]
primitive_derive! { usize => crate::Gt<crate::One> }
#[cfg(target_pointer_width = "64")]
primitive_derive! { usize => crate::Gt<crate::One> }

primitive_derive! { isize => <usize as RustSpec>::Alignment }

raw_pointer_derive! { const }
raw_pointer_derive! { mut }

unsafe impl<R: RustSpec> RustSpec for [R] {
    type Layout = R::Layout;
    type Size = MetaSized<SliceLike>;
    type Alignment = R::Alignment;
    type Trap = R::Trap;
    // TODO: This should not be set at all
    // however we set it to help some impls
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
    type __IndirectTrap = R::__IndirectTrap;
}

unsafe impl<R: RustSpec> RustSpec for [R; 0] {
    type Layout = R::Layout;
    type Size = size::Sized<Zero>;
    type Alignment = R::Alignment;
    type Trap = Robust;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}

impl_nonempty_array!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32
);

impl_fn_types! {
    (),
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
    (A, B, C, D, E, F, G, H, I, J, K, L),
}

// The alignment marker is Gt<One> on the supported targets. Check only that
// category; function and data pointers need not have identical alignment.
const _: () = {
    assert!(core::mem::align_of::<fn()>() > 1);
    assert!(core::mem::align_of::<unsafe fn()>() > 1);
};

fieldless_enum_derive! { char => crate::Gt<crate::One> }
fieldless_enum_derive! { bool => crate::One }
fieldless_enum_derive! { Ordering => crate::One }

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::{boxed::Box, vec::Vec};
    use static_assertions::assert_impl_all;

    use super::*;
    use crate::{
        Unstable,
        layout::{NonRobust, Robust},
        niche::WithNiche,
    };

    #[test]
    fn function_pointer_classification() {
        type RustFn = fn();
        type CFn12 = extern "C" fn(u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8);

        assert_impl_all!(RustFn: RustSpec<Layout = Unstable, Niche = WithNiche<Stable>>);
        assert_impl_all!(unsafe fn(u8) -> u16: RustSpec<Layout = Unstable>);
        assert_impl_all!(extern "Rust" fn(u8) -> u16: RustSpec<Layout = Unstable>);
        assert_impl_all!(extern "C" fn(u8) -> u16: RustSpec<Layout = Stable>);
        assert_impl_all!(unsafe extern "C" fn(u8) -> u16: RustSpec<Layout = Stable>);
        assert_impl_all!(extern "C-unwind" fn(u8) -> u16: RustSpec<Layout = Stable>);
        assert_impl_all!(extern "system" fn(u8) -> u16: RustSpec<Layout = Stable>);
        assert_impl_all!(extern "system-unwind" fn(u8) -> u16: RustSpec<Layout = Stable>);
        #[cfg(target_arch = "x86_64")]
        assert_impl_all!(extern "sysv64" fn(u8) -> u16: RustSpec<Layout = Stable>);
        #[cfg(target_arch = "x86_64")]
        assert_impl_all!(extern "win64" fn(u8) -> u16: RustSpec<Layout = Stable>);
        assert_impl_all!(extern "C" fn((u8,)) -> u16: RustSpec<Layout = Unstable>);
        assert_impl_all!(extern "C" fn(u8) -> (u16,): RustSpec<Layout = Unstable>);
        assert_impl_all!(CFn12: RustSpec<Layout = Stable>);
    }

    #[test]
    fn function_pointer_option_has_pointer_size() {
        macro_rules! assert_option_pointer_size {
            ($pointer:ty) => {
                assert_eq!(
                    core::mem::size_of::<Option<$pointer>>(),
                    core::mem::size_of::<$pointer>()
                );
            };
        }

        fn named() {}
        let _: fn() = named;

        assert_option_pointer_size!(fn());
        assert_option_pointer_size!(unsafe fn(u8) -> u16);
        assert_option_pointer_size!(extern "C" fn(u8) -> u16);
        assert_option_pointer_size!(unsafe extern "C-unwind" fn(u8) -> u16);
        assert_option_pointer_size!(extern "system" fn());
    }

    #[test]
    fn robust_u8() {
        assert_impl_all!(u8:
            RustSpec<
                Layout = Stable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&u8:
            RustSpec<
                Layout = Stable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&mut u8:
            RustSpec<
                Layout = Stable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<u8>:
            RustSpec<
                Layout = Stable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&[u8]:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&mut [u8]:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[u8]>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<u8>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!([u8; 2]:
            RustSpec<
                Layout = Stable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<u8>:
            RustSpec<
                Layout = Unstable,
                Size = size::Sized<crate::Gt<Zero>>,
                Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithNiche<crate::Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }
}
