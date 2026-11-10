//! One value per analyzed axis, in the order used for resolution and emission.
//! Opacity is fixed per derive and does not participate in field analysis.
use std::ops::{Index, IndexMut};

// Keep the named fields, traversal order, and associated-type names together.
macro_rules! define_axes {
    ($($field:ident: $axis:ident => $name:ident),+ $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub(crate) enum Axis {
            $($axis),+
        }

        impl Axis {
            pub(crate) fn name(self) -> syn::Ident {
                quote::format_ident!("{}", match self {
                    $(Self::$axis => stringify!($name)),+
                })
            }
        }

        pub(crate) struct Families<T> {
            $(pub $field: T),+
        }

        impl<T> Families<T> {
            pub(crate) fn from_fn(mut f: impl FnMut(Axis) -> T) -> Self {
                Self { $($field: f(Axis::$axis)),+ }
            }

            pub(crate) fn map<U>(self, mut f: impl FnMut(T) -> U) -> Families<U> {
                Families { $($field: f(self.$field)),+ }
            }
        }

        impl<T> IntoIterator for Families<T> {
            type Item = (Axis, T);
            type IntoIter = std::array::IntoIter<Self::Item, { [$(stringify!($axis)),+].len() }>;

            fn into_iter(self) -> Self::IntoIter {
                [$( (Axis::$axis, self.$field) ),+].into_iter()
            }
        }

        impl<T> Index<Axis> for Families<T> {
            type Output = T;

            fn index(&self, axis: Axis) -> &Self::Output {
                match axis {
                    $(Axis::$axis => &self.$field),+
                }
            }
        }

        impl<T> IndexMut<Axis> for Families<T> {
            fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
                match axis {
                    $(Axis::$axis => &mut self.$field),+
                }
            }
        }
    };
}

// Preserve this order when extending the registry: recursive resolution uses it.
define_axes! {
    layout: Layout => Layout,
    size: Size => Size,
    alignment: Alignment => __Alignment,
    trap: Trap => Trap,
    niche: Niche => Niche,
    drop: Drop => Drop,
    mutability: Mutability => Mutability,
    indirect_trap: IndirectTrap => __IndirectTrap,
}
