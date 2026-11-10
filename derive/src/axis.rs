//! Recursive, axis-specific field analysis. Bounds are emitted only for the
//! expression that survives simplification.
use crate::{
    EmittedFamily, classify, crate_path,
    families::{Axis, Families},
    field_has_generic_params,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, Type};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Known {
    Stable,
    Unstable,
    Zero,
    Nonzero,
    Slice,
    DynTrait,
    Robust,
    NonRobust,
    NoNiche,
    NoDrop,
    AutoDrop,
    Exclusive,
    One,
}
impl Known {
    fn tokens(self) -> TokenStream {
        let c = crate_path();
        match self {
            Self::Stable => quote!(#c::Stable),
            Self::Unstable => quote!(#c::Unstable),
            Self::Zero => quote!(#c::size::Sized<#c::Zero>),
            Self::Nonzero => quote!(#c::size::Sized<#c::Gt<#c::Zero>>),
            Self::Slice => quote!(#c::size::MetaSized<#c::size::SliceLike>),
            Self::DynTrait => quote!(#c::size::MetaSized<#c::size::DynTraitLike>),
            Self::Robust => quote!(#c::layout::Robust),
            Self::NonRobust => quote!(#c::layout::NonRobust),
            Self::NoNiche => quote!(#c::niche::WithoutNiche),
            Self::NoDrop => quote!(#c::drop::NoDrop),
            Self::AutoDrop => quote!(#c::drop::AutoDrop),
            Self::Exclusive => quote!(#c::mutability::Exclusive<#c::mutability::WithoutInterior>),
            Self::One => quote!(#c::One),
        }
    }
}
#[derive(Clone)]
pub(crate) enum Expr {
    Fixed(TokenStream),
    CustomDrop(Box<Expr>),
    Nullable(Box<Expr>, Axis),
    Known(Known),
    Projection(Type, Axis),
    Binary(Axis, Box<Expr>, Box<Expr>),
    SizedField(Box<Expr>, Type),
    PointerLayout(Box<Expr>),
    PointerMutability(Box<Expr>),
    Niche(Box<Expr>),
}
impl Expr {
    pub(crate) fn aggregate(axis: Axis, generics: &Generics, fields: &[&Type]) -> Option<Self> {
        aggregate(axis, fields, generics)
    }

    pub(crate) fn aggregate_with_seed(
        axis: Axis,
        seed: Known,
        generics: &Generics,
        fields: &[&Type],
    ) -> Self {
        let seed = Self::Known(seed);
        aggregate(axis, fields, generics).map_or(seed.clone(), |expression| {
            Self::combine(axis, seed, expression)
        })
    }

    fn known(k: Known) -> Self {
        Self::Known(k)
    }
    fn is(&self, k: Known) -> bool {
        matches!(self, Self::Known(v) if *v == k)
    }
    pub(crate) fn combine(axis: Axis, left: Self, right: Self) -> Self {
        use Known::*;
        if matches!(axis, Axis::Drop | Axis::Niche) {
            // These seeds normalize fields; applying the same seed twice is redundant.
            let seed = if axis == Axis::Drop { NoDrop } else { NoNiche };
            if left.is(seed)
                && let Self::Binary(inner_axis, inner_left, _) = &right
                && *inner_axis == axis
                && inner_left.is(seed)
            {
                return right;
            }
        }
        match axis {
            Axis::Layout => {
                if left.is(Unstable) || right.is(Unstable) {
                    return Self::known(Unstable);
                }
                if left.is(Stable) {
                    return right;
                }
                if right.is(Stable) {
                    return left;
                }
            }
            Axis::Trap | Axis::IndirectTrap => {
                if left.is(NonRobust) || right.is(NonRobust) {
                    return Self::known(NonRobust);
                }
                if left.is(Robust) {
                    return right;
                }
                if right.is(Robust) {
                    return left;
                }
            }
            Axis::Size => {
                if left.is(Slice) || right.is(Slice) {
                    return Self::known(Slice);
                }
                if left.is(Zero) {
                    return right;
                }
                if right.is(Zero) {
                    return left;
                }
                if left.is(Nonzero) && right.is(Nonzero) {
                    return left;
                }
            }
            Axis::Alignment => {
                if left.is(One) {
                    return right;
                }
                if right.is(One) {
                    return left;
                }
            }
            Axis::Drop
                if left.is(AutoDrop)
                    || right.is(AutoDrop)
                    || matches!(&left, Self::CustomDrop(_))
                    || matches!(&right, Self::CustomDrop(_)) =>
            {
                return Self::known(AutoDrop);
            }
            Axis::Drop if left.is(NoDrop) && right.is(NoDrop) => return left,
            Axis::Niche if matches!(&left, Self::Niche(_)) || matches!(&right, Self::Niche(_)) => {
                return Self::Niche(Box::new(Self::known(Unstable)));
            }
            Axis::Niche if left.is(NoNiche) && right.is(NoNiche) => return left,
            Axis::Mutability => {
                if left.is(Exclusive) {
                    return right;
                }
                if right.is(Exclusive) {
                    return left;
                }
            }
            _ => {}
        }
        Self::Binary(axis, Box::new(left), Box::new(right))
    }
    pub(crate) fn nullable(inner: Self, axis: Axis) -> Self {
        let stable = matches!(&inner, Self::Niche(layout) if layout.is(Known::Stable));
        let unstable = inner.is(Known::NoNiche)
            || matches!(&inner, Self::Niche(layout) if layout.is(Known::Unstable));
        if stable || unstable {
            match axis {
                Axis::Layout => Self::known(if stable {
                    Known::Stable
                } else {
                    Known::Unstable
                }),
                Axis::Trap => Self::known(if stable {
                    Known::Robust
                } else {
                    Known::NonRobust
                }),
                Axis::Niche if stable => Self::known(Known::NoNiche),
                Axis::Niche => Self::Niche(Box::new(Self::known(Known::Unstable))),
                _ => unreachable!("nullable representations expose layout, trap, and niche"),
            }
        } else {
            Self::Nullable(Box::new(inner), axis)
        }
    }

    pub(crate) fn pointer_layout(inner: Self) -> Self {
        match inner {
            Self::Known(Known::Zero | Known::Nonzero) => Self::known(Known::Stable),
            Self::Known(Known::Slice | Known::DynTrait) => Self::known(Known::Unstable),
            inner => Self::PointerLayout(Box::new(inner)),
        }
    }

    pub(crate) fn pointer_mutability(inner: Self) -> Self {
        // Pointer mutability always produces Exclusive<...>, on which another
        // pointer transformation is the identity.
        if inner.is(Known::Exclusive) || matches!(&inner, Self::PointerMutability(_)) {
            inner
        } else {
            Self::PointerMutability(Box::new(inner))
        }
    }

    fn absorbs_sized_field(&self) -> bool {
        matches!(
            self,
            Self::Known(Known::Nonzero | Known::Slice | Known::DynTrait)
        )
    }

    fn sized_field(inner: Self, ty: Type) -> Self {
        if inner.absorbs_sized_field() {
            inner
        } else {
            Self::SizedField(Box::new(inner), ty)
        }
    }

    fn has_params(&self, generics: &Generics) -> bool {
        match self {
            Self::Fixed(_) | Self::Known(_) => false,
            Self::CustomDrop(x) | Self::Nullable(x, _) => x.has_params(generics),
            Self::Projection(ty, _) => field_has_generic_params(ty, generics),
            Self::Binary(_, l, r) => l.has_params(generics) || r.has_params(generics),
            Self::SizedField(inner, ty) => {
                inner.has_params(generics) || field_has_generic_params(ty, generics)
            }
            Self::PointerLayout(x) | Self::PointerMutability(x) | Self::Niche(x) => {
                x.has_params(generics)
            }
        }
    }
    pub(crate) fn emit(self, generics: &Generics) -> EmittedFamily {
        let c = crate_path();
        match self {
            Self::Fixed(tokens) => EmittedFamily::fixed(tokens),
            Self::CustomDrop(inner) => {
                let mut family = inner.emit(generics);
                let kind = family.kind;
                if family.depends_on_generics {
                    family
                        .aggregate_bounds
                        .push(quote!(#kind: #c::drop::DropKind));
                }
                family.kind = quote!(#c::drop::CustomDrop<#kind>);
                family
            }
            Self::Nullable(inner, axis) => {
                let mut family = inner.emit(generics);
                let kind = family.kind;
                let name = axis.name();
                if family.depends_on_generics {
                    family
                        .aggregate_bounds
                        .push(quote!(#kind: #c::niche::NullableRepresentation));
                }
                family.kind = quote!(<#kind as #c::niche::NullableRepresentation>::#name);
                family
            }
            Self::Known(k) => EmittedFamily::fixed(k.tokens()),
            Self::Projection(ty, axis) => {
                let name = axis.name();
                let mut family = EmittedFamily::fixed(quote!(<#ty as #c::RustSpec>::#name));
                family.depends_on_generics = field_has_generic_params(&ty, generics);
                if family.depends_on_generics {
                    family.aggregate_bounds.push(quote!(#ty: #c::RustSpec));
                    if axis == Axis::Drop {
                        family
                            .aggregate_bounds
                            .push(quote!(<#ty as #c::RustSpec>::Drop: #c::drop::DropKind));
                    }
                    if axis == Axis::Niche {
                        family.aggregate_bounds.push(quote!(#c::niche::WithoutNiche: ::core::ops::Add<<#ty as #c::RustSpec>::Niche>));
                    }
                }
                family
            }
            Self::Binary(axis, left, right) => {
                let mut left = left.emit(generics);
                let right = right.emit(generics);
                let operator = match axis {
                    Axis::Alignment => quote!(#c::Max),
                    Axis::Drop => quote!(CombineDrop),
                    _ => quote!(::core::ops::Add),
                };
                let l = left.kind;
                let r = right.kind;
                left.depends_on_generics |= right.depends_on_generics;
                left.aggregate_bounds.extend(right.aggregate_bounds);
                // Concrete operations need no where predicate. Generic operations
                // do, including operations whose operands are nested expressions.
                if left.depends_on_generics {
                    left.aggregate_bounds.push(quote!(#l: #operator<#r>));
                }
                left.kind = quote!(<#l as #operator<#r>>::Output);
                left
            }
            Self::SizedField(inner, ty) => {
                let mut family = inner.emit(generics);
                let kind = family.kind;
                family.depends_on_generics |= field_has_generic_params(&ty, generics);
                if family.depends_on_generics {
                    family
                        .aggregate_bounds
                        .push(quote!(#kind: AddSizedField<#ty>));
                }
                family.kind = quote!(<#kind as AddSizedField<#ty>>::Output);
                family
            }
            Self::PointerLayout(inner) => {
                emit_unary(*inner, generics, Some(quote!(#c::size::PointerLayout)))
            }
            Self::PointerMutability(inner) => emit_unary(
                *inner,
                generics,
                Some(quote!(#c::mutability::PointerMutability)),
            ),
            Self::Niche(inner) => emit_unary(*inner, generics, None),
        }
    }
}

fn emit_unary(inner: Expr, generics: &Generics, operator: Option<TokenStream>) -> EmittedFamily {
    let c = crate_path();
    let mut family = inner.emit(generics);
    let kind = family.kind;
    if let Some(operator) = operator {
        if family.depends_on_generics {
            family.aggregate_bounds.push(quote!(#kind: #operator));
        }
        family.kind = quote!(<#kind as #operator>::Output);
    } else {
        family.kind = quote!(#c::niche::WithNiche<#kind>);
    }
    family
}

pub(crate) fn aggregate(axis: Axis, fields: &[&Type], generics: &Generics) -> Option<Expr> {
    // Earlier fields are sized by Rust's well-formedness rules. Only the tail
    // can prevent a nonzero witness from settling the size of the aggregate.
    if axis == Axis::Size
        && let Some(tail) = fields.last()
        && classify::classify_type(tail, generics) == classify::TypeClass::Sized
        && fields
            .iter()
            .any(|t| classify::classify_nonzero(t) == classify::NonZeroClass::Yes)
    {
        return Some(Expr::known(Known::Nonzero));
    }
    if axis == Axis::Size {
        let (tail, prefix) = fields.split_last()?;
        // Every prefix field is Sized. A known nonzero prefix therefore
        // absorbs the other prefix sizes, even when the tail is an unknown
        // projection. Keep the tail on the left so emission retains its bounds.
        if prefix
            .iter()
            .any(|ty| classify::classify_nonzero(ty) == classify::NonZeroClass::Yes)
        {
            return Some(Expr::combine(
                Axis::Size,
                field(tail, axis, generics),
                Expr::known(Known::Nonzero),
            ));
        }
        // Once the tail is known to be sized, any field may be the seed.
        // Prefer a concrete one so rustc can discard generic size dependencies.
        let seed = if classify::classify_type(tail, generics) == classify::TypeClass::Sized {
            fields
                .iter()
                .rposition(|ty| !field_has_generic_params(ty, generics))
                .unwrap_or(fields.len() - 1)
        } else {
            fields.len() - 1
        };
        let mut size = field(fields[seed], axis, generics);
        for ty in fields
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(i, ty)| (i != seed).then_some(ty))
        {
            if size.absorbs_sized_field() {
                break;
            }
            if classify::classify_nonzero(ty) == classify::NonZeroClass::No {
                continue;
            }
            size = if size.is(Known::Zero) {
                field(ty, axis, generics)
            } else {
                Expr::sized_field(size, (*ty).clone())
            };
        }
        return Some(size);
    }
    let (concrete, generic): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|ty| field(ty, axis, generics))
        .partition(|expr| !expr.has_params(generics));
    let fold = |items: Vec<Expr>| items.into_iter().reduce(|l, r| Expr::combine(axis, l, r));
    match (fold(concrete), fold(generic)) {
        (Some(l), Some(r)) => Some(Expr::combine(axis, l, r)),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

pub(crate) fn field(ty: &Type, axis: Axis, generics: &Generics) -> Expr {
    use Axis::*;
    use Known::*;
    let known = Expr::known;
    let projection = || Expr::Projection(ty.clone(), axis);
    match ty {
        Type::Paren(t) => field(&t.elem, axis, generics),
        Type::Group(t) => field(&t.elem, axis, generics),
        Type::Tuple(t) => {
            if t.elems.is_empty() {
                return known(match axis {
                    Layout => Stable,
                    Size => Zero,
                    Alignment => One,
                    Trap | IndirectTrap => Robust,
                    Niche => NoNiche,
                    Drop => NoDrop,
                    Mutability => Exclusive,
                });
            }
            if axis == Layout {
                return known(Unstable);
            }
            let fields: Vec<_> = t.elems.iter().collect();
            let inner = aggregate(axis, &fields, generics).unwrap();
            if axis == Drop {
                Expr::combine(axis, known(NoDrop), inner)
            } else {
                inner
            }
        }
        Type::Array(t) => {
            let length = match &t.len {
                syn::Expr::Lit(l) => match &l.lit {
                    syn::Lit::Int(n) => n.base10_parse::<usize>().ok(),
                    _ => None,
                },
                _ => None,
            };
            if axis == Layout || axis == Alignment {
                return field(&t.elem, axis, generics);
            }
            match length {
                Some(0) => known(match axis {
                    Size => Zero,
                    Trap | IndirectTrap => Robust,
                    Niche => NoNiche,
                    Drop => NoDrop,
                    Mutability => Exclusive,
                    _ => unreachable!(),
                }),
                Some(_) => {
                    let inner = field(&t.elem, axis, generics);
                    match axis {
                        Drop => Expr::combine(axis, known(NoDrop), inner),
                        Niche => Expr::combine(axis, known(NoNiche), inner),
                        _ => inner,
                    }
                }
                None => projection(),
            }
        }
        Type::Slice(t) => match axis {
            Size => known(Slice),
            Niche => known(NoNiche),
            Drop => Expr::combine(Drop, known(NoDrop), field(&t.elem, Drop, generics)),
            _ => field(&t.elem, axis, generics),
        },
        Type::FnPtr(function) => match axis {
            Size => known(Nonzero),
            Alignment => Expr::Projection(syn::parse_quote!(::core::primitive::usize), Alignment),
            Trap => known(NonRobust),
            Niche => Expr::Niche(Box::new(known(Stable))),
            Drop => known(NoDrop),
            Mutability => known(Exclusive),
            IndirectTrap => known(Robust),
            Layout
                if function.abi.as_ref().is_none_or(|abi| {
                    abi.name.as_ref().is_some_and(|name| name.value() == "Rust")
                }) =>
            {
                known(Unstable)
            }
            // Foreign ABI layout depends on the signature. Keep the whole
            // projection so bound lifetimes stay inside the function type.
            Layout => projection(),
        },
        Type::TraitObject(_) if axis == Size => known(DynTrait),
        Type::Reference(_) | Type::Ptr(_) => {
            let (pointee, reference) = match ty {
                Type::Reference(t) => (&*t.elem, true),
                Type::Ptr(t) => (&*t.elem, false),
                _ => unreachable!(),
            };
            match axis {
                Size => known(Nonzero),
                Alignment => {
                    Expr::Projection(syn::parse_quote!(::core::primitive::usize), Alignment)
                }
                Drop => known(NoDrop),
                Trap if reference => known(NonRobust),
                Trap | IndirectTrap => field(pointee, Trap, generics),
                Niche if !reference => known(NoNiche),
                Niche => Expr::Niche(Box::new(pointer_layout(pointee, generics))),
                Layout => Expr::combine(
                    Layout,
                    pointer_layout(pointee, generics),
                    field(pointee, Layout, generics),
                ),
                Mutability => {
                    let inner = field(pointee, axis, generics);
                    Expr::pointer_mutability(inner)
                }
            }
        }
        Type::Path(_) | Type::Macro(_) | Type::Verbatim(_) => projection(),
        // These are not ordinary supported field types; leave diagnostics and
        // any future RustSpec implementations to the compiler.
        Type::ImplTrait(_) | Type::Infer(_) | Type::Never(_) => projection(),
        _ => projection(),
    }
}
fn pointer_layout(ty: &Type, generics: &Generics) -> Expr {
    match classify::classify_type(ty, generics) {
        classify::TypeClass::Sized => Expr::known(Known::Stable),
        classify::TypeClass::Unsized(
            classify::UnsizedTail::SliceLike | classify::UnsizedTail::DynTraitLike,
        ) => Expr::known(Known::Unstable),
        _ => Expr::pointer_layout(field(ty, Axis::Size, generics)),
    }
}

/// Block-scoped public traits can appear in the public implementation's
/// associated types, but their names cannot be reached outside the anonymous
/// constant. Each expansion owns its helpers and introduces no module names.
pub(crate) fn helpers() -> TokenStream {
    let c = crate_path();
    quote! {
        pub trait CombineDrop<Rhs> {
            type Output: #c::drop::DropKind;
        }
        impl<L, R> CombineDrop<R> for L
        where
            L: ::core::ops::Add<R>,
            <L as ::core::ops::Add<R>>::Output: #c::drop::DropKind,
        {
            type Output = <L as ::core::ops::Add<R>>::Output;
        }

        pub trait AddSizedField<T> {
            type Output;
        }
        impl<T> AddSizedField<T> for #c::size::Sized<#c::Gt<#c::Zero>> {
            type Output = Self;
        }
        impl<T: #c::RustSpec> AddSizedField<T> for #c::size::Sized<#c::Zero> {
            type Output = T::Size;
        }
        impl<T, K: #c::size::MetadataKind> AddSizedField<T> for #c::size::MetaSized<K> {
            type Output = Self;
        }
        impl<T> AddSizedField<T> for #c::size::ExternTypeLike {
            type Output = Self;
        }
        impl<T> AddSizedField<T> for #c::size::NulTerminated {
            type Output = Self;
        }
    }
}

/// Resolve the enclosing type's axis equations before generating projections.
/// A direct self edge contributes no new layout instability, trap values or
/// interior mutability. Start those cycles at their identity classifications;
/// the other fields and the enclosing representation supply the actual result.
pub(crate) fn resolve_self(
    families: Families<Expr>,
    name: &syn::Ident,
    generics: &Generics,
) -> (Families<Expr>, crate::wrappers::Checks) {
    enum Resolution {
        Unvisited(Expr),
        Active,
        Resolved(Expr),
    }

    struct Resolver<'a> {
        own_type: Type,
        generics: &'a Generics,
        states: Families<Resolution>,
        checks: crate::wrappers::Checks,
    }
    impl Resolver<'_> {
        fn is_self(&self, ty: &Type) -> bool {
            match ty {
                Type::Paren(t) => self.is_self(&t.elem),
                Type::Group(t) => self.is_self(&t.elem),
                Type::Path(p) if p.qself.is_none() && p.path.is_ident("Self") => true,
                _ => ty == &self.own_type,
            }
        }
        fn contains_self(&self, ty: &Type) -> bool {
            struct Visitor<'a, 'b> {
                resolver: &'a Resolver<'b>,
                found: bool,
            }
            impl syn::visit::Visit<'_> for Visitor<'_, '_> {
                fn visit_type(&mut self, ty: &Type) {
                    if self.resolver.is_self(ty) {
                        self.found = true;
                    } else {
                        syn::visit::visit_type(self, ty);
                    }
                }
            }
            let mut visitor = Visitor {
                resolver: self,
                found: false,
            };
            syn::visit::Visit::visit_type(&mut visitor, ty);
            visitor.found
        }
        fn axis(&mut self, axis: Axis) -> Expr {
            let root = match &self.states[axis] {
                Resolution::Resolved(expr) => return expr.clone(),
                Resolution::Active => {
                    return match axis {
                        Axis::Layout => Expr::known(Known::Stable),
                        Axis::Trap | Axis::IndirectTrap => Expr::known(Known::Robust),
                        Axis::Drop => Expr::known(Known::NoDrop),
                        Axis::Mutability => Expr::known(Known::Exclusive),
                        // Other cycles are not justified by the pointer rules.
                        // Preserve them for rustc instead of inventing a result.
                        _ => Expr::Projection(self.own_type.clone(), axis),
                    };
                }
                Resolution::Unvisited(expr) => expr.clone(),
            };
            self.states[axis] = Resolution::Active;
            let expression = self.expression(root);
            self.states[axis] = Resolution::Resolved(expression.clone());
            expression
        }
        fn expression(&mut self, expr: Expr) -> Expr {
            let expr = match expr {
                Expr::CustomDrop(inner) => Expr::CustomDrop(Box::new(self.expression(*inner))),
                Expr::Nullable(inner, axis) => Expr::nullable(self.expression(*inner), axis),
                Expr::Binary(axis, left, right) => {
                    Expr::combine(axis, self.expression(*left), self.expression(*right))
                }
                Expr::SizedField(inner, ty) => Expr::sized_field(self.expression(*inner), ty),
                Expr::PointerLayout(inner) => Expr::pointer_layout(self.expression(*inner)),
                Expr::PointerMutability(inner) => Expr::pointer_mutability(self.expression(*inner)),
                Expr::Niche(inner) => Expr::Niche(Box::new(self.expression(*inner))),
                expr => expr,
            };
            match expr {
                Expr::Projection(ty, axis) if self.is_self(&ty) => self.axis(axis),
                Expr::Projection(ty, axis) if self.contains_self(&ty) => {
                    if let Some((wrapper, args)) =
                        crate::wrappers::Wrapper::recursive_use(&ty, |ty| self.is_self(ty))
                    {
                        self.checks.add(ty, wrapper, args.clone());
                        let expr = wrapper.expression(&args, axis, self.generics);
                        self.expression(expr)
                    } else {
                        Expr::Projection(ty, axis)
                    }
                }
                Expr::SizedField(inner, ty) if self.contains_self(&ty) => {
                    let field_size = self.expression(field(&ty, Axis::Size, self.generics));
                    Expr::combine(Axis::Size, *inner, field_size)
                }
                expr => expr,
            }
        }
    }
    let (_, args, _) = generics.split_for_impl();
    let mut resolver = Resolver {
        own_type: syn::parse_quote!(#name #args),
        generics,
        states: families.map(Resolution::Unvisited),
        checks: crate::wrappers::Checks::default(),
    };
    let families = Families::from_fn(|axis| resolver.axis(axis));
    (families, resolver.checks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn normalization_of_nullable_pointers_keeps_only_pointee_dependencies() {
        let generics = parse_quote!(<T>);
        let pointee = parse_quote!(T);
        for (ty, axis, expected) in [
            (
                parse_quote!(&T),
                Axis::Layout,
                field(&pointee, Axis::Layout, &generics),
            ),
            (
                parse_quote!(&T),
                Axis::Trap,
                field(&pointee, Axis::Trap, &generics),
            ),
            (parse_quote!(&T), Axis::Niche, Expr::known(Known::NoNiche)),
            (
                parse_quote!(*mut T),
                Axis::Layout,
                Expr::known(Known::Unstable),
            ),
            (
                parse_quote!(*mut T),
                Axis::Trap,
                Expr::known(Known::NonRobust),
            ),
        ] {
            // These rules do not require recursive resolution. A reference's
            // null niche preserves pointee dependencies; a raw pointer needs a
            // tag whose unstable/non-robust contributions absorb them.
            let actual = crate::wrappers::Wrapper::Option
                .expression(&[ty], axis, &generics)
                .emit(&generics);
            let expected = expected.emit(&generics);
            assert_eq!(actual.kind.to_string(), expected.kind.to_string());
            assert_eq!(
                actual
                    .aggregate_bounds
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                expected
                    .aggregate_bounds
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            );
        }
    }

    #[test]
    fn normalization_does_not_add_bounds_for_concrete_custom_drop() {
        let ty = parse_quote!(u8);
        for generics in [Generics::default(), parse_quote!(<T>)] {
            let family = crate::gen_drop_family(&generics, &[&ty], true).emit(&generics);
            assert!(family.aggregate_bounds.is_empty());
        }
    }

    #[test]
    fn zero_sized_prefixes_do_not_add_size_dependencies() {
        let generics = parse_quote!(<T: ?Sized, U>);
        let expected = field(&parse_quote!(T), Axis::Size, &generics).emit(&generics);
        for ty in [
            parse_quote!(((), T)),
            parse_quote!(([U; 0], T)),
            parse_quote!((((), [U; 0]), (), T)),
        ] {
            let actual = field(&ty, Axis::Size, &generics).emit(&generics);
            assert_eq!(actual.kind.to_string(), expected.kind.to_string());
            assert_eq!(
                actual
                    .aggregate_bounds
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                expected
                    .aggregate_bounds
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            );
        }
        // A potentially nonzero field must still contribute.
        let actual = field(&parse_quote!((U, T)), Axis::Size, &generics).emit(&generics);
        assert!(actual.kind.to_string().contains("AddSizedField"));
    }

    #[test]
    fn nested_pointer_mutability_needs_only_one_transformation() {
        let generics = parse_quote!(<'a, T: ?Sized>);
        let expected = field(&parse_quote!(*const T), Axis::Mutability, &generics).emit(&generics);
        for ty in [
            parse_quote!(*const *mut T),
            parse_quote!(&'a &'a T),
            parse_quote!(&'a mut *const &'a T),
        ] {
            let actual = field(&ty, Axis::Mutability, &generics).emit(&generics);
            assert_eq!(actual.kind.to_string(), expected.kind.to_string());
            assert_eq!(
                actual
                    .aggregate_bounds
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                expected
                    .aggregate_bounds
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            );
        }
    }

    #[test]
    fn nested_syntax_discards_unneeded_axis_dependencies() {
        let generics = parse_quote!(<'a, T, U>);
        for (ty, axis, expected) in [
            (parse_quote!([(&'a T, U); 2]), Axis::Layout, Known::Unstable),
            (parse_quote!([(&'a T, U); 2]), Axis::Trap, Known::NonRobust),
            (parse_quote!(([T; 0], &'a U)), Axis::Drop, Known::NoDrop),
            (parse_quote!([(&'a T, U); 0]), Axis::Size, Known::Zero),
            (parse_quote!([(&'a T, U); 0]), Axis::Trap, Known::Robust),
            (
                parse_quote!([(&'a T, U); 0]),
                Axis::Mutability,
                Known::Exclusive,
            ),
            (parse_quote!((T, (&'a U, ()))), Axis::Size, Known::Nonzero),
            (parse_quote!(((&'a T,), [U])), Axis::Size, Known::Slice),
        ] {
            let family = field(&ty, axis, &generics).emit(&generics);
            assert_eq!(family.kind.to_string(), expected.tokens().to_string());
            assert!(family.aggregate_bounds.is_empty());
        }
    }

    #[test]
    fn aggregate_absorbers_discard_other_fields_and_their_bounds() {
        let generics = parse_quote!(<T, U>);
        let unknown = parse_quote!(T);
        let tuple = parse_quote!([(&U,); 2]);
        for (axis, expected) in [
            (Axis::Layout, Known::Unstable),
            (Axis::Trap, Known::NonRobust),
        ] {
            let family = aggregate(axis, &[&unknown, &tuple], &generics)
                .unwrap()
                .emit(&generics);
            assert_eq!(family.kind.to_string(), expected.tokens().to_string());
            assert!(family.aggregate_bounds.is_empty());
        }
    }

    #[test]
    fn function_pointer_axes_do_not_depend_on_signature_types() {
        let generics = parse_quote!(<T>);
        for ty in [
            parse_quote!(fn(T) -> T),
            parse_quote!(unsafe extern "C" fn(T) -> T),
            parse_quote!(for<'a> fn(&'a T) -> &'a T),
        ] {
            for (axis, expected) in [
                (Axis::Size, Known::Nonzero),
                (Axis::Drop, Known::NoDrop),
                (Axis::Trap, Known::NonRobust),
                (Axis::Mutability, Known::Exclusive),
                (Axis::IndirectTrap, Known::Robust),
            ] {
                let family = field(&ty, axis, &generics).emit(&generics);
                assert_eq!(family.kind.to_string(), expected.tokens().to_string());
                assert!(family.aggregate_bounds.is_empty());
            }
        }
        let rust =
            field(&parse_quote!(extern "Rust" fn(T)), Axis::Layout, &generics).emit(&generics);
        assert_eq!(rust.kind.to_string(), Known::Unstable.tokens().to_string());
        assert!(rust.aggregate_bounds.is_empty());
        let foreign =
            field(&parse_quote!(extern "C" fn(T)), Axis::Layout, &generics).emit(&generics);
        assert!(!foreign.aggregate_bounds.is_empty());
    }

    #[test]
    fn trait_object_tail_determines_size() {
        let generics = parse_quote!(<T>);
        let family = field(
            &parse_quote!((T, dyn ::core::fmt::Debug)),
            Axis::Size,
            &generics,
        )
        .emit(&generics);
        assert_eq!(
            family.kind.to_string(),
            Known::DynTrait.tokens().to_string()
        );
        assert!(family.aggregate_bounds.is_empty());
    }

    #[test]
    fn zero_arrays_keep_alignment_and_layout_but_unknown_lengths_keep_projections() {
        let generics = parse_quote!(<T, const N: usize>);
        for axis in [Axis::Alignment, Axis::Layout] {
            let family = field(&parse_quote!([T; 0]), axis, &generics).emit(&generics);
            assert!(!family.aggregate_bounds.is_empty());
            assert!(family.kind.to_string().contains("T as"));
        }
        for axis in [
            Axis::Size,
            Axis::Trap,
            Axis::Niche,
            Axis::Drop,
            Axis::Mutability,
        ] {
            let family = field(&parse_quote!([T; N]), axis, &generics).emit(&generics);
            assert!(family.kind.to_string().contains("[T ; N] as"));
        }
    }
}
