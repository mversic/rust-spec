# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
