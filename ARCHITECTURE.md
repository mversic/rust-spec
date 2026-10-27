# rust-spec Architecture

`RustSpec` is the canonical classification trait for properties of Rust types. Its associated types describe independent axes:

`Opaque` is a separate unsafe marker trait, not a `RustSpec` axis. It marks types whose value representation must not be inspected by conversions. Deriving `RustSpec` with `#[rust_spec(opaque)]` implements it for source types and selects the opaque classification; generated opaque-export implementations require it.

| Axis | Meaning | Markers |
| --- | --- | --- |
| `Layout` | Whether Rust guarantees the representation shape | `Stable`, `Unstable` |
| `Size` | Statically sized or dynamically sized shape | `size::Sized<Zero>`, `size::Sized<Gt<Zero>>`, `size::MetaSized<SliceLike>`, `size::MetaSized<DynTraitLike>`, `size::ExternTypeLike`, `size::NulTerminated` |
| `Alignment` | Whether alignment is one or greater than one | `One`, `Gt<One>` |
| `Trap` | Whether every bit pattern is a valid value | `layout::Robust`, `layout::NonRobust` |
| `Niche` | Availability and stability of a niche value | `niche::WithoutNiche`, `niche::WithNiche<Stable>`, `niche::WithNiche<Unstable>` |
| `Drop` | Whether dropping the type runs custom drop glue, inner drop glue, both, or neither | `drop::NoDrop`, `drop::CustomDrop<drop::NoDrop>`, `drop::AutoDrop`, `drop::CustomDrop<drop::AutoDrop>` |
| `Mutability` | Whether type can be mutated through shared access. | `mutability::Exclusive`, `mutability::WithInterior` |

## Composite types

Implementations for built-in wrappers, pointers, tuples, `Option`, and `Result` live in `src/`. The derive macro covers structs, enums, and unions. It combines field classifications with type-level operations, taking account of the representation and enum tag where applicable. For generic fields, it adds the bounds required by those combinations. Syntax-known absorbing contributions bypass the corresponding axis fold: nonempty tuple fields and pointers/references to slices fix layout to `Unstable`; references or a tag with invalid values fix `Trap` to `NonRobust`; a slice tail fixes `Size` to `MetaSized<SliceLike>`. Field `RustSpec` requirements remain in place, and named types retain trait-based classification.

The `Drop` axis distinguishes an outer `Drop` implementation from drop glue for owned contents. The derive combines the `Drop` classifications of all fields starting at `NoDrop`, then wraps the result in `CustomDrop` if the type has `#[rust_spec(custom_drop)]`:

| Outer type implements `Drop` | Field or owned content needs dropping | Result |
| --- | --- | --- |
| No | No | `NoDrop` |
| Yes | No | `CustomDrop<NoDrop>` |
| No | Yes | `AutoDrop` |
| Yes | Yes | `CustomDrop<AutoDrop>` |

A field classified as anything other than `NoDrop` counts as inner drop behavior in its container, even when that field's own classification includes custom drop glue. Owned contents of types such as `Box<T>` follow the same rule. `Vec<T>` is always `CustomDrop<AutoDrop>` because its `RawVec` field frees the allocation after `Vec::drop`, independently of `T`'s drop behavior. Syntactically recognized raw pointer and reference fields are omitted from the derive’s drop calculation because they contribute `NoDrop`; an empty calculation remains `NoDrop`, and `custom_drop` is applied afterward. This includes parenthesized and grouped pointer/reference types. Borrowed references do not own their referents; `ManuallyDrop<T>` and `MaybeUninit<T>` suppress dropping their contents.

`custom_drop` is required exactly when the derived type itself implements `core::ops::Drop`. The derive emits compile-time checks for both a missing annotation and an annotation without a `Drop` implementation. These checks apply to generic types as well. For structs, the separate `#[rust_spec(custom_niche)]` attribute selects `WithNiche<Unstable>`. The derive cannot verify the niche. Enums classify niches using unused discriminant values, and enums and unions reject the annotation.

## Safety and guarantees

`Layout` and `Trap` are separate axes: a type can have a stable representation while some bit patterns remain invalid. `Alignment` records only whether alignment is one or greater than one; it does not encode an exact byte count. `Niche` records whether a niche is available and whether Rust guarantees it, rather than a particular niche value. The `Size` markers describe shape, not an exact byte count. `repr(packed)` is not supported by the derive.
