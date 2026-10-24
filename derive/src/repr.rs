use proc_macro2::Delimiter;
use syn::{
    Attribute, Meta, Token,
    parse::{Parse, ParseStream, Parser as _},
    punctuated::Punctuated,
};

#[derive(Clone, PartialEq)]
pub(crate) enum ReprKind {
    Transparent,
    C(Option<Box<syn::Type>>),
    Primitive(Box<syn::Type>),
}

enum ReprToken {
    Kind(ReprKind),
    Align(usize),
    Packed,
}

impl Parse for ReprToken {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.step(|cursor| {
            let Some((ident, after_token)) = cursor.ident() else {
                return Err(cursor.error("Expected repr kind"));
            };

            match ident.to_string().as_str() {
                "transparent" => Ok((ReprToken::Kind(ReprKind::Transparent), after_token)),
                "C" => Ok((ReprToken::Kind(ReprKind::C(None)), after_token)),
                "u8" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(u8))),
                    after_token,
                )),
                "i8" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(i8))),
                    after_token,
                )),
                "u16" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(u16))),
                    after_token,
                )),
                "i16" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(i16))),
                    after_token,
                )),
                "u32" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(u32))),
                    after_token,
                )),
                "i32" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(i32))),
                    after_token,
                )),
                "u64" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(u64))),
                    after_token,
                )),
                "i64" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(i64))),
                    after_token,
                )),
                "usize" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(usize))),
                    after_token,
                )),
                "isize" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(isize))),
                    after_token,
                )),
                "packed"
                    if let Some((_inside, _span, after_group)) =
                        after_token.group(Delimiter::Parenthesis) =>
                {
                    Ok((ReprToken::Packed, after_group))
                }
                "packed" => Ok((ReprToken::Packed, after_token)),
                "align"
                    if let Some((inside, _span, after_group)) =
                        after_token.group(Delimiter::Parenthesis) =>
                {
                    let literal = syn::parse2::<syn::LitInt>(inside.token_stream())?;
                    Ok((ReprToken::Align(literal.base10_parse()?), after_group))
                }
                "align" => Err(cursor.error("Expected alignment")),
                _ => Err(cursor.error("Unrecognized repr kind")),
            }
        })
    }
}

pub(crate) struct Repr {
    pub kind: Option<ReprKind>,
    pub align: Option<usize>,
}

pub(crate) fn parse_repr(attrs: &[Attribute]) -> syn::Result<Repr> {
    let repr_attrs = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("repr"))
        .collect::<Vec<_>>();

    if repr_attrs.is_empty() {
        return Ok(Repr {
            kind: None,
            align: None,
        });
    }

    let mut kind = None;
    let mut align = None;

    for attr in repr_attrs {
        let Meta::List(list) = &attr.meta else {
            continue;
        };

        let tokens =
            Punctuated::<ReprToken, Token![,]>::parse_terminated.parse2(list.tokens.clone())?;
        for token in tokens {
            match token {
                ReprToken::Kind(new_kind) => match (&mut kind, new_kind) {
                    (Some(ReprKind::C(None)), ReprKind::Primitive(prim)) => {
                        kind = Some(ReprKind::C(Some(prim)));
                    }
                    (Some(ReprKind::Primitive(prim)), ReprKind::C(None)) => {
                        kind = Some(ReprKind::C(Some(prim.clone())));
                    }
                    (Some(existing), new_kind) if *existing == new_kind => {}
                    (Some(_), _) => {
                        return Err(syn::Error::new_spanned(attr, "Duplicate repr kind"));
                    }
                    (None, new_kind) => kind = Some(new_kind),
                },
                ReprToken::Align(value) => {
                    align = Some(align.map_or(value, |existing: usize| existing.max(value)));
                }
                ReprToken::Packed => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "`repr(packed)` is not supported yet; remove `packed` from the repr attribute",
                    ));
                }
            }
        }
    }

    Ok(Repr { kind, align })
}

/// Infer a tag from known discriminants and the number of variants.
/// Unknown values and following implicit increments are checked by rustc against
/// the inferred range. Target-dependent operations are left to that const check.
pub(crate) fn infer_repr(variants: &Punctuated<syn::Variant, Token![,]>) -> syn::Result<syn::Type> {
    let mut previous = Some(-1i128);
    let mut minimum = 0i128;
    let mut maximum = 0i128;
    for variant in variants {
        let value = if let Some((_, expr)) = &variant.discriminant {
            eval_discriminant(expr)
        } else {
            previous.and_then(|value| value.checked_add(1))
        };
        if let Some(value) = value {
            minimum = minimum.min(value);
            maximum = maximum.max(value);
        }
        previous = value;
    }
    let count = variants.len() as u128;
    let tag = if minimum >= 0 {
        match maximum {
            0..=255 if count <= 256 => syn::parse_quote!(u8),
            0..=65535 if count <= 65536 => syn::parse_quote!(u16),
            0..=4294967295 if count <= 4294967296 => syn::parse_quote!(u32),
            _ if maximum <= u64::MAX as i128 => syn::parse_quote!(u64),
            _ => {
                return Err(syn::Error::new_spanned(
                    variants,
                    "enum discriminant exceeds the supported tag range",
                ));
            }
        }
    } else if minimum >= i8::MIN as i128 && maximum <= i8::MAX as i128 && count <= 256 {
        syn::parse_quote!(i8)
    } else if minimum >= i16::MIN as i128 && maximum <= i16::MAX as i128 && count <= 65536 {
        syn::parse_quote!(i16)
    } else if minimum >= i32::MIN as i128 && maximum <= i32::MAX as i128 && count <= 4294967296 {
        syn::parse_quote!(i32)
    } else if minimum >= i64::MIN as i128 && maximum <= i64::MAX as i128 {
        syn::parse_quote!(i64)
    } else {
        return Err(syn::Error::new_spanned(
            variants,
            "enum discriminant exceeds the supported tag range",
        ));
    };
    Ok(tag)
}

pub(crate) fn has_unresolved_discriminants(variants: &Punctuated<syn::Variant, Token![,]>) -> bool {
    variants.iter().any(|variant| {
        variant
            .discriminant
            .as_ref()
            .is_some_and(|(_, expr)| eval_discriminant(expr).is_none())
    })
}

fn eval_discriminant(expr: &syn::Expr) -> Option<i128> {
    match expr {
        syn::Expr::Lit(expr) => match &expr.lit {
            syn::Lit::Int(value) => value.base10_parse().ok(),
            _ => None,
        },
        syn::Expr::Paren(expr) => eval_discriminant(&expr.expr),
        syn::Expr::Group(expr) => eval_discriminant(&expr.expr),
        syn::Expr::Unary(expr) if matches!(expr.op, syn::UnOp::Neg(_)) => {
            eval_discriminant(&expr.expr)?.checked_neg()
        }
        syn::Expr::Binary(expr) => {
            let left = eval_discriminant(&expr.left)?;
            let right = eval_discriminant(&expr.right)?;
            match expr.op {
                syn::BinOp::Add(_) => left.checked_add(right),
                syn::BinOp::Sub(_) => left.checked_sub(right),
                syn::BinOp::Mul(_) => left.checked_mul(right),
                syn::BinOp::Div(_) => left.checked_div(right),
                syn::BinOp::Rem(_) => left.checked_rem(right),
                // Shifts depend on the target isize width and can discard bits.
                // Leave their evaluation to the emitted const check.
                syn::BinOp::BitAnd(_) => Some(left & right),
                syn::BinOp::BitOr(_) => Some(left | right),
                syn::BinOp::BitXor(_) => Some(left ^ right),
                _ => None,
            }
        }
        _ => None,
    }
}

pub(crate) fn is_exhaustive_enum(num_variants: usize, repr: &syn::Type) -> bool {
    fn repr_type_bit_width(repr: &syn::Type) -> Option<u32> {
        let syn::Type::Path(type_path) = repr else {
            return None;
        };
        let ident = type_path.path.get_ident()?.to_string();

        match ident.as_str() {
            "u8" | "i8" => Some(8),
            "u16" | "i16" => Some(16),
            "u32" | "i32" => Some(32),
            "u64" | "i64" => Some(64),
            _ => None,
        }
    }

    let max_values = match repr_type_bit_width(repr) {
        Some(8) => 1u64 << 8,
        Some(16) => 1u64 << 16,
        Some(32) => 1u64 << 32,
        Some(64) | None | Some(_) => return false,
    };

    num_variants as u64 == max_values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_repr_parts_in_separate_attributes() {
        let attrs = [
            syn::parse_quote!(#[repr(C)]),
            syn::parse_quote!(#[repr(align(16))]),
        ];

        let repr = parse_repr(&attrs).unwrap();
        assert!(matches!(repr.kind, Some(ReprKind::C(None))));
        assert_eq!(repr.align, Some(16));
    }

    #[test]
    fn rejects_duplicate_repr_kinds() {
        let attrs = [
            syn::parse_quote!(#[repr(C)]),
            syn::parse_quote!(#[repr(transparent)]),
        ];

        assert!(parse_repr(&attrs).is_err());
    }

    #[test]
    fn accepts_matching_duplicate_repr_parts() {
        let attrs = [
            syn::parse_quote!(#[repr(C, align(16))]),
            syn::parse_quote!(#[repr(C, align(16))]),
        ];

        assert!(parse_repr(&attrs).is_ok());
    }

    #[test]
    fn keeps_the_largest_alignment() {
        let attrs = [
            syn::parse_quote!(#[repr(align(8))]),
            syn::parse_quote!(#[repr(align(16))]),
            syn::parse_quote!(#[repr(align(4))]),
        ];

        assert_eq!(parse_repr(&attrs).unwrap().align, Some(16));
    }
}
