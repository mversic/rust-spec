# rust-spec Architecture

`RustSpec` is the canonical classification trait for properties of Rust types. Its associated types describe independent axes:

| Axis | Meaning | Markers |
| --- | --- | --- |
| `Layout` | Whether Rust guarantees the representation shape | `Stable`, `Unstable` |
| `Size` | Statically sized or dynamically sized shape | `size::Sized<Zero>`, `size::Sized<Gt<Zero>>`, `size::MetaSized<SliceLike>`, `size::MetaSized<DynTraitLike>`, `size::ExternTypeLike`, `size::NulTerminated` |
| `Alignment` | Whether ABI alignment is one or greater than one | `One`, `Gt<One>` |
| `Trap` | Whether every bit pattern is a valid value | `layout::Robust`, `layout::NonRobust` |
| `Niche` | Availability and stability of a niche value | `niche::WithoutNiche`, `niche::WithNiche<Stable>`, `niche::WithNiche<Unstable>` |
| `Drop` | Whether dropping the type runs custom drop glue, inner drop glue, both, or neither | `drop::NoDrop`, `drop::CustomDrop<drop::NoDrop>`, `drop::AutoDrop`, `drop::CustomDrop<drop::AutoDrop>` |
| `Mutability` | Whether the whole value may be mutated through shared access | `mutability::Exclusive`, `mutability::Interior` |

`Mutability` and `__IndirectTrap` are internal classification axes. `__IndirectTrap` tracks trap values reachable through supported pointer indirections. Implementors of the unsafe `RustSpec` trait must classify their types truthfully.

## Composite types

Implementations for built-in wrappers, pointers, tuples, `Option`, and `Result` live in `src/`. The derive macro covers structs, enums, and unions. It combines field classifications with type-level operations, taking account of the representation and enum tag where applicable. For generic fields, it adds the bounds required by those combinations.

The `Drop` axis distinguishes an outer `Drop` implementation from drop glue for owned contents. The derive starts at `NoDrop` unless the type has `#[rust_spec(custom_drop)]`, in which case it starts at `CustomDrop<NoDrop>`. It then combines the `Drop` classifications of all fields:

| Outer type implements `Drop` | Field or owned content needs dropping | Result |
| --- | --- | --- |
| No | No | `NoDrop` |
| Yes | No | `CustomDrop<NoDrop>` |
| No | Yes | `AutoDrop` |
| Yes | Yes | `CustomDrop<AutoDrop>` |

A field classified as anything other than `NoDrop` counts as inner drop behavior in its container, even when that field's own classification includes custom drop glue. Owned contents of types such as `Box<T>` and `Vec<T>` follow the same rule. Borrowed references do not own their referents; `ManuallyDrop<T>` and `MaybeUninit<T>` suppress dropping their contents.

`custom_drop` is required exactly when the derived type itself implements `core::ops::Drop`. The derive emits compile-time checks for both a missing annotation and an annotation without a `Drop` implementation. These checks apply to generic types as well. For structs, the separate `#[rust_spec(custom_niche)]` attribute selects `WithNiche<Unstable>`. The derive cannot verify the niche. Enums classify niches using unused discriminant values, and enums and unions reject the annotation.

## Safety and guarantees

`Layout` and `Trap` are separate axes: a type can have a stable representation while some bit patterns remain invalid. `Alignment` records only whether alignment is one or greater than one; it does not encode an exact byte count. `Niche` records whether a niche is available and whether Rust guarantees it, rather than a particular niche value. The `Size` markers describe shape, not an exact byte count. `repr(packed)` is not supported by the derive.
