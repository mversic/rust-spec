# Rust Specification

[<img alt="crates.io" src="https://img.shields.io/crates/v/rust-spec.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/rust-spec)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-rust-spec-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/rust-spec)
[<img alt="CI" src="https://img.shields.io/github/actions/workflow/status/mversic/rust-spec/main.yaml?style=for-the-badge&label=CI" height="20">](https://github.com/mversic/rust-spec/actions/workflows/main.yaml)

Compile-time classification of types according to Rust specification.

## Classification Axes

`RustSpec` describes a type across the following axes:
- `Layout`: layout stability and robustness.
- `Size`: statically sized, metadata-sized, or extern-type-like shape.
- `Niche`: stable, unstable, or absent niche value.
- `Mutability`: whether the whole value can be mutated through shared access.

### Layout

Describes whether a type has a stable layout or trap/invalid values:
- `Stable<Robust>`: stable layout with no trap values.
- `Stable<NonRobust>`: stable layout, but with trap values.
- `Unstable<Robust>`: unstable layout with no trap values.
- `Unstable<NonRobust>`: unstable layout, but with trap values.

### Size

Describes compile-time size and pointer metadata shape:
- `Sized<Zst>`: compile-time known zero-sized type.
- `Sized<NonZst>`: compile-time known non-zero-sized type.
- `MetaSized<SliceLike>`: dynamically sized slice-like type.
- `MetaSized<DynTraitLike>`: dynamically sized trait-object-like type.
- `ExternTypeLike`: dynamically sized extern-type-like type.

### Niche

Describes whether and what kind of niche is available for the type:
- `WithoutNiche`: no niche is available.
- `WithNiche<Stable>`: compiler-guaranteed niche.
- `WithNiche<Unstable>`: niche exists but is not guaranteed.

### Mutability

Describes whether shared access (`&R`) can mutate the whole value:
- `Interior`: the whole value may be mutated through shared access.
- `Exclusive`: mutation of the value requires exclusive access.

## How to Use

Derive `RustSpec` for your types, then use its associated marker families as bounds when implementing other traits:

```rust
use disjoint_impls::disjoint_impls;
use rust_spec::{
    RustSpec,
    niche::{self, WithNiche, WithoutNiche},
};

trait NullableEncoding {
    const NEEDS_TAG: bool;
}

#[derive(RustSpec)]
struct Header {
    id: u32,
    flags: u16,
}

disjoint_impls! {
    impl<T: RustSpec<Niche = WithoutNiche>> NullableEncoding for T {
        const NEEDS_TAG: bool = true;
    }

    impl<T: RustSpec<Niche = WithNiche<niche::Stable>>> NullableEncoding for T {
        const NEEDS_TAG: bool = false;
    }

    impl<T: RustSpec<Niche = WithNiche<niche::Unstable>>> NullableEncoding for T {
        const NEEDS_TAG: bool = false;
    }
}

const HEADER_OPTION_NEEDS_TAG: bool = Header::NEEDS_TAG;
```
