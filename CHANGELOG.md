# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.1] - 2026-10-31

### Added

- track robustness of types with interior mutability

### Changed

- hide `RustSpec::__Alignment` axis
- remove `disjoint_impls` dependency

## [0.7.0] - 2026-10-28

### Changed

- Replace the `Opaque` marker trait with the `RustSpec::Opacity = Opaque | NonOpaque` axis.
- Reject raw pointer fields in non-opaque derives and conflicting `opaque` attributes.

### Fixed

- Classify raw pointers and `NonNull` according to the pointee's metadata shape.
- Correct custom and automatic drop classification for containers and derived types.
- Preserve payload mutability in derived enums and classify custom niches as `NonRobust`.
- Check unresolved enum discriminants against the inferred tag range.

## [0.6.1] - 2026-10-21

### Fixed

- inherit `Layout` axis for `UnsafeCell` from the wrapped type

## [0.6.0] - 2026-10-16

### Added

- add support for opaque types
- make `Mutability = Exclusive | WithInterior` axis public

### Fixed

- Don't propagate `Mutability` through pointer indirection

## [0.5.2] - 2026-10-11

### Fixed

- Set `<String as RustSpec>::__IndirectTrap` to `NonRobust`

## [0.5.1] - 2026-10-09

### Added

- Categorize types with custom `Drop` based on drop behavior of their contents

### Changed

- Flatten the `Drop` classification markers by removing `WithDrop` wrapper

## [0.5.0] - 2026-10-09

### Added

- Classify custom and inner drop behavior through `RustSpec::Drop`.
- Support `#[rust_spec(custom_drop)]` with compile-time checks.

### Fixed

- Reject `#[rust_spec(with_custom_niche)` on enums and unions.

### Changed

- Rename `#[rust_spec(with_custom_niche)]` to `#[rust_spec(custom_niche)]`.

## [0.4.1] - 2026-10-03

### Fixed

- Slices now inherit `Mutability` of the element type

## [0.4.0] - 2026-10-01

### Added

- introduce new `Size` axis classifier `NulTerminated`

## [0.3.0] - 2026-09-21

### Added

- support function pointers

## [0.2.0] - 2026-09-20

### Fixed

- categorize `CString` as `Sized`

## [0.1.0] - 2026-09-16

### Added

- Compile-time type classification by layout, size, alignment, value validity, and niche.
- `#[derive(RustSpec)]` for structs, enums, and unions, including supported `repr` attributes.
- `no_std` support, with optional `alloc` and `derive` features.
