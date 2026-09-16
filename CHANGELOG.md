# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## 0.1.0 - 16.09.2026

### Added

- Compile-time type classification by layout, size, alignment, value validity, and niche.
- `#[derive(RustSpec)]` for structs, enums, and unions, including supported `repr` attributes.
- `no_std` support, with optional `alloc` and `derive` features.
