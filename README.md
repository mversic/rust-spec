# Rust Specification

[<img alt="crates.io" src="https://img.shields.io/crates/v/rust-spec.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/rust-spec)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-rust--spec-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/rust-spec)
[<img alt="CI" src="https://img.shields.io/github/actions/workflow/status/mversic/rust-spec/main.yaml?style=for-the-badge&label=CI" height="20">](https://github.com/mversic/rust-spec/actions/workflows/main.yaml)

**Compile-time classification of types** according to the Rust specification.

## Classification Axes

`RustSpec` describes a type across the following axes:
- `Layout`: representation stability.
- `Size`: statically sized, metadata-sized, or extern-type-like shape.
- `Alignment`: whether ABI alignment is one or greater than one.
- `Trap`: whether the value representation has trap values.
- `Niche`: whether a type has a stable, unstable, or no niche value.
- `Drop`: whether droping runs custom or compiler-generated code.

### Layout

Describes total reachable (through pointer indirection) representation stability:
- `Stable`: compiler-guaranteed representation.
- `Unstable`: representation is not guaranteed.

### Size

Describes compile-time size and pointer metadata shape:
- `Sized<Zero>`: compile-time known zero-sized type.
- `Sized<Gt<Zero>>`: compile-time known non-zero-sized type.
- `MetaSized<SliceLike>`: dynamically sized slice-like type.
- `MetaSized<DynTraitLike>`: dynamically sized trait-object-like type.
- `ExternTypeLike`: dynamically sized extern-type-like type.
- `NulTerminated`: unsized data whose length is found from a nul terminator.

### Alignment

- `One` means ABI alignment is exactly one;
- `Gt<One>` means it is greater than one.

### Trap

Describes total reachable (through pointer indirection) value-representation validity:
- `Robust`: every bit pattern is valid.
- `NonRobust`: some bit patterns are trap/invalid values.

### Niche

Describes whether and what kind of niche is available for the type:
- `WithoutNiche`: no niche is available.
- `WithNiche<Stable>`: compiler-guaranteed niche.
- `WithNiche<Unstable>`: niche exists but is not guaranteed.

**Use `#[rust_spec(custom_niche)]` when deriving `RustSpec` to select `WithNiche<Unstable>`**.

### Drop

Describes the type's behavior when dropped:
- `NoDrop`: dropping the type runs no drop glue.
- `AutoDrop`: a field or other owned content has drop behavior.
- `CustomDrop<NoDrop>`: the type implements `Drop` without any compiler-generated "drop glue".
- `CustomDrop<AutoDrop>`: the type implements `Drop` and owns content that needs drop glue.

**Use `#[rust_spec(custom_drop)]` when deriving `RustSpec` for a type that implements `Drop`**.


## How to Use

Derive `RustSpec` for your types, then use its associated marker families as bounds when implementing other traits:

```rust
use disjoint_impls::disjoint_impls;
use rust_spec::{RustSpec, Stable, Unstable, niche::{WithNiche, WithoutNiche}};

#[derive(RustSpec)]
struct Header {
    id: u32,
    flags: u16,
}

disjoint_impls! {
    trait NullableEncoding {
        const NEEDS_TAG: bool;
    }

    impl<T: RustSpec<Niche = WithoutNiche>> NullableEncoding for T {
        const NEEDS_TAG: bool = true;
    }

    impl<T: RustSpec<Niche = WithNiche<Stable>>> NullableEncoding for T {
        const NEEDS_TAG: bool = false;
    }

    impl<T: RustSpec<Niche = WithNiche<Unstable>>> NullableEncoding for T {
        const NEEDS_TAG: bool = false;
    }
}

const HEADER_OPTION_NEEDS_TAG: bool = Header::NEEDS_TAG;
```

Check out [CO3](https://github.com/mversic/co3) to see how this crate allows for building higher zero-cost abstractions.
