# rust-spec Architecture

## 1. Rust Spec

Rust-specified type properties are joined under one canonical trait:

```rs
unsafe trait TypeSpec {
    type Repr;
    type Size;
    type Niche;
    type Mutability;
}
```

`TypeSpec` combines representation stability/robustness, size shape, niche availability, and shared-access mutability into one Rust-spec classification. The older `ReprFamily`, `SizeFamily`, `NicheFamily`, and `MutabilityFamily` names are compatibility projections for code that still needs to name one axis independently.

`TypeSpec::Size` carries the same safety contract as `SizeFamily::Kind`: implementors must truthfully classify whether the type is statically sized, metadata-sized, or extern-type-like.

## 2. Representation Axis

Representation categorization is exposed through `TypeSpec::Repr`:

```rs
unsafe trait TypeSpec {
    type Repr;
    // ...
}
```

where `TypeSpec::Repr` is assigned one of the categories below through a marker of the same name:

1. **`Stable<Robust>`** (marker type)
- Types with stable C layout and no trap representations (e.g. `u32`).
- Usually map directly to themselves in ABI (no conversion necessary).

2. **`Stable<NonRobust>`** (marker type)
- Types that can be safely transmuted into a single chosen target type.
- IR/ABI mapping and value conversion continue through the target type.

3. **`Unstable<Robust>`** (marker type)
- Types without stable C layout whose explicitly converted value space has no trap representations.
- Unstable layout still requires explicit conversion, but the converted value does not need robustness validation.

4. **`Unstable<NonRobust>`** (marker type)
- Types without stable C layout whose converted value space can contain trap representations.
- Conversion must keep the same validation obligations as `Stable<NonRobust>`.

### 2.1 Composite Types

Composite types derive `TypeSpec::Repr` by combining layout stability and robustness separately:

- `Stable<_> + Stable<_>` remains `Stable<_>`.
- Any combination containing `Unstable<_>` becomes `Unstable<_>`.
- `Robust + Robust` remains `Robust`.
- Any combination containing `NonRobust` becomes `NonRobust`.

Rust-repr derived structs and enums start from `Unstable<Robust>` and then combine all field families. `#[repr(C)]`, primitive, and transparent derived types start from `Stable<Robust>` unless their tag or fields introduce `NonRobust`.

References and `Box<R>` preserve `Unstable<K>` when the referent is unstable. Stable thin referents are represented as `Stable<NonRobust>`, while stable metadata-sized referents are represented as `Unstable<NonRobust>`.

`Option<R>` and niche-shaped `Result<R, E>` preserve the payload family when the ABI is the payload representation. Explicit wrapper forms are `Unstable<NonRobust>`.

## 3. Size Axis

Size categorization is exposed through `TypeSpec::Size`.

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

## 4. Niche Axis

Niche categorization is exposed through `TypeSpec::Niche`:

```rs
unsafe trait TypeSpec {
    type Niche;
    // ...
}
```

where `TypeSpec::Niche` is assigned one of the categories below through a marker of the same name:

1. **`WithNiche<Stable>`** (marker type)
- Type has a compiler-guaranteed niche value (refer to [doc](https://doc.rust-lang.org/std/option/#representation)).

2. **`WithNiche<Custom>`** (marker type)
- Type has a `crate`-defined sentinel niche value and `Option<T>` is encoded as `T::CType`.

3. **`WithoutNiche`** (marker type)
- Type has no niche value and `Option<T>` must be encoded as a 2-tuple with a discriminant.

### 4.1 Composite Types

The tables below specifies how composite types derive `TypeSpec::Niche`:

| Self | `Self::Kind` |
| --- | --- |
| `&R` | `WithNiche<Stable>` |
| `&mut R` | `WithNiche<Stable>` |
| `Box<R>` | `WithNiche<Stable>` |
| `&[R]` | `WithNiche<Custom>` |
| `&mut [R]` | `WithNiche<Custom>` |
| `Box<[R]>` | `WithNiche<Custom>` |
| `Vec<R>` | `WithNiche<Custom>` |

#### `[R; N]`

| `<R as TypeSpec>::Niche` | `Self::Niche` |
| --- | --- |
| `WithNiche<Stable>` | `WithNiche<Custom>` |
| `WithNiche<Custom>` | `WithNiche<Custom>` |
| `WithoutNiche` | `WithoutNiche` |

#### `Option<R>`

| `<R as TypeSpec>::Niche` | `Self::Niche` |
| --- | --- |
| `WithoutNiche` | `WithNiche<Custom>` |
| `WithNiche<Stable>` | `WithoutNiche` |
| `WithNiche<Custom>` | `WithNiche<Custom>` |

## 5. Mutability Axis

Mutability categorization is exposed through `TypeSpec::Mutability`:

```rs
unsafe trait TypeSpec {
    type Mutability;
    // ...
}
```

where `TypeSpec::Mutability` is assigned one of the categories below through a marker of the same name:

1. **`Interior`** (marker type)
- Types whose whole ABI-exposed value may be mutated through shared access.
- `UnsafeCell<T>` and wrappers whose entire representation is interior-mutable belong to this family.

2. **`Exclusive`** (marker type)
- Types whose ABI-exposed value requires exclusive access to mutate.

### 5.1 Composite Types

Composite types derive `TypeSpec::Mutability` structurally:

| Field kinds | Self::Kind |
| --- | --- |
| All fields `Interior` | `Interior` |
| Any `Exclusive` field | `Exclusive` |

Empty composites are `Exclusive`.
References propagate the referent's mutability family.
`Box<T>` propagates the boxed type's mutability family when `T: Sized`; `Box<T>` for unsized `T` is classified as `Exclusive`.
Raw pointers, `NonNull<T>`, and `Vec<T>` are classified as `Exclusive` regardless of `T`, because their ABI-exposed wrapper state is not wholly mutable through shared access.
`Option<T>` and niche-shaped `Result<T, E>` preserve the payload family when their ABI is the payload representation; explicit wrapper forms are `Exclusive`.
