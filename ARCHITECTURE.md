# rust-spec Architecture

## 1. Rust Spec

Rust-specified type properties are joined under one canonical trait:

```rs
unsafe trait RustSpec {
    type Layout;
    type Size;
    type Alignment;
    type Trap;
    type Niche;
    type Mutability;
}
```

`RustSpec` combines layout stability, size shape, alignment, trap robustness,
niche availability, and shared-access mutability into one Rust-spec
classification.

`RustSpec::Size` carries the safety contract for size classification: implementors must truthfully classify whether the type is statically sized, metadata-sized, or extern-type-like.

`RustSpec::Alignment` carries the ABI alignment in bytes as a `typenum`
unsigned integer. Its value must be a non-zero power of two. Composite types
combine field and enum-tag alignments with type-level maximum operations;
`repr(align)` raises that result. `repr(packed)` is currently unsupported.

## 2. Representation Axis

Representation categorization is exposed through `RustSpec::Layout`:

```rs
unsafe trait RustSpec {
    type Layout;
    // ...
}
```

where `RustSpec::Layout` is assigned one of the categories below through a marker of the same name:

1. **`Stable<Robust>`** (marker type)
- Types with stable C layout and no trap representations (e.g. `u32`).
- Usually map directly to themselves in ABI (no conversion necessary).

2. **`Stable<NonRobust>`** (marker type)
- Types that can be safely transmuted into a single chosen target type.
- IR/ABI mapping and value conversion continue through the target type.

3. **`Unstable<()>`** (marker type)
- Types whose representation Rust does not guarantee.
- Their raw-representation robustness is unknown.

### 2.1 Composite Types

Composite types derive `RustSpec::Layout` by combining layout stability and, for stable layouts, robustness:

- `Stable<_> + Stable<_>` remains `Stable<_>`.
- Any combination containing `Unstable<_>` becomes `Unstable<()>`.
- `Robust + Robust` remains `Robust`.
- Any combination containing `NonRobust` becomes `NonRobust`.

Rust-repr derived structs and enums start from `Unstable<()>` and then combine all field families. `#[repr(C)]`, primitive, and transparent derived types start from `Stable<Robust>` unless their tag or fields introduce `NonRobust`.

References and `Box<R>` follow their referent's layout stability, so an
unstable referent makes their `Layout` unstable. Their robustness remains
`NonRobust`, because a reference or owning pointer itself has invalid values.
Stable thin references start from `Stable<NonRobust>`, while stable
metadata-sized references start from `Unstable<()>`.

`Option<R>` and niche-shaped `Result<R, E>` preserve the payload family when the ABI is the payload representation. Explicit wrapper forms are `Unstable<()>`.

### 2.2 Indirect Layout

Supported pointers follow their pointee when classifying layout stability,
because a valid Rust reference or `Box` requires a valid pointee. Raw pointers
and `NonNull<R>` do not follow their pointees because their pointees are not
part of their validity invariant.

`RustSpec::__IndirectLayout` remains an implementation accumulator: it excludes
the immediate representation and lets wrappers calculate their layout. Its
neutral value is `Stable<Robust>`.

## 3. Size Axis

Size categorization is exposed through `RustSpec::Size`.

The categories are:

1. **`Sized<Zst>`**
- Statically sized type with zero size.

2. **`Sized<NonZst>`**
- Statically sized type with nonzero size.

3. **`ExternTypeLike`**
- Unsized extern-type-like layout with thin pointers.

4. **`MetaSized<SliceLike>`**
- DST layout whose last element is a Rust slice.

5. **`MetaSized<DynTraitLike>`**
- DST layout whose last element is a trait object.

## 4. Alignment Axis

Alignment is exposed through `RustSpec::Alignment`. Leaf types declare their
typenum alignment directly; aggregates combine the alignments of their fields,
and transparent wrappers preserve their wrapped alignment.

## 5. Niche Axis

Niche categorization is exposed through `RustSpec::Niche`:

```rs
unsafe trait RustSpec {
    type Niche;
    // ...
}
```

where `RustSpec::Niche` is assigned one of the categories below through a marker of the same name:

1. **`WithNiche<Stable>`** (marker type)
- Type has a compiler-guaranteed niche value (refer to [doc](https://doc.rust-lang.org/std/option/#representation)).

2. **`WithNiche<Unstable>`** (marker type)
- Type has a niche value, but that niche is not compiler-guaranteed.

3. **`WithoutNiche`** (marker type)
- Type has no niche value and `Option<T>` must be encoded as a 2-tuple with a discriminant.

### 4.1 Composite Types

The tables below specifies how composite types derive `RustSpec::Niche`:

| Self | `Self::Kind` |
| --- | --- |
| `&R` | `WithNiche<Stable>` |
| `&mut R` | `WithNiche<Stable>` |
| `Box<R>` | `WithNiche<Stable>` |
| `&[R]` | `WithNiche<Unstable>` |
| `&mut [R]` | `WithNiche<Unstable>` |
| `Box<[R]>` | `WithNiche<Unstable>` |
| `Vec<R>` | `WithNiche<Unstable>` |

#### `[R; N]`

| `<R as RustSpec>::Niche` | `Self::Niche` |
| --- | --- |
| `WithNiche<Stable>` | `WithNiche<Unstable>` |
| `WithNiche<Unstable>` | `WithNiche<Unstable>` |
| `WithoutNiche` | `WithoutNiche` |

#### `Option<R>`

| `<R as RustSpec>::Niche` | `Self::Niche` |
| --- | --- |
| `WithoutNiche` | `WithNiche<Unstable>` |
| `WithNiche<Stable>` | `WithoutNiche` |
| `WithNiche<Unstable>` | `WithNiche<Unstable>` |

## 5. Mutability Axis

Mutability categorization is exposed through `RustSpec::Mutability`:

```rs
unsafe trait RustSpec {
    type Mutability;
    // ...
}
```

where `RustSpec::Mutability` is assigned one of the categories below through a marker of the same name:

1. **`Interior`** (marker type)
- Types whose whole value may be mutated through shared access.
- `UnsafeCell<T>` and wrappers whose entire representation is interior-mutable belong to this family.

2. **`Exclusive`** (marker type)
- Types whose value requires exclusive access to mutate.

### 5.1 Composite Types

Composite types only propagate `RustSpec::Mutability` through a single payload field:

| Field kinds | Self::Kind |
| --- | --- |
| Exactly one field, `Interior` | `Interior` |
| Otherwise | `Exclusive` |

Structs use this rule regardless of representation. Enums use it only with Rust or transparent
representation and exactly one variant. C and integer-representation enums, unions, and empty
composites are `Exclusive`.
References propagate the referent's mutability family.
`Box<T>` propagates the boxed type's mutability family when `T: Sized`; `Box<T>` for unsized `T` is classified as `Exclusive`.
Raw pointers, `NonNull<T>`, and `Vec<T>` are classified as `Exclusive` regardless of `T`, because their wrapper state is not wholly mutable through shared access.
`Option<T>` and niche-shaped `Result<T, E>` preserve the payload family when their ABI is the payload representation; explicit wrapper forms are `Exclusive`.
