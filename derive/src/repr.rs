use proc_macro2::Delimiter;
use syn::{
    Attribute, Meta, Token,
    parse::{Parse, ParseStream, Parser as _},
    punctuated::Punctuated,
};

#[derive(Clone)]
pub(crate) enum ReprKind {
    Transparent,
    C(Option<Box<syn::Type>>),
    Primitive(Box<syn::Type>),
}

enum ReprToken {
    Kind(ReprKind),
    Align(usize),
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
                "packed" => Err(cursor.error(
                    "`repr(packed)` is not supported yet; remove `packed` from the repr attribute",
                )),
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

    if repr_attrs.len() > 1 {
        let err = "Multiple repr attributes";
        return Err(syn::Error::new_spanned(repr_attrs[1], err));
    }

    let Some(&attr) = repr_attrs.first() else {
        return Ok(Repr {
            kind: None,
            align: None,
        });
    };

    let Meta::List(list) = &attr.meta else {
        return Ok(Repr {
            kind: None,
            align: None,
        });
    };

    let tokens =
        Punctuated::<ReprToken, Token![,]>::parse_terminated.parse2(list.tokens.clone())?;
    let mut kind = None;
    let mut align = None;

    for token in tokens {
        match token {
            ReprToken::Kind(new_kind) => match (&mut kind, new_kind) {
                (Some(ReprKind::C(None)), ReprKind::Primitive(prim)) => {
                    kind = Some(ReprKind::C(Some(prim)));
                }
                (Some(ReprKind::Primitive(prim)), ReprKind::C(None)) => {
                    kind = Some(ReprKind::C(Some(prim.clone())));
                }
                (Some(_), _) => {
                    let err = "Duplicate repr kind within attribute";
                    return Err(syn::Error::new_spanned(attr, err));
                }
                (None, new_kind) => kind = Some(new_kind),
            },
            ReprToken::Align(value) => align = Some(value),
        }
    }

    Ok(Repr { kind, align })
}

pub(crate) fn infer_repr(num_variants: usize) -> syn::Type {
    const U8_CAPACITY: usize = u8::MAX as usize + 1;
    const U16_CAPACITY: usize = u16::MAX as usize + 1;
    const U32_CAPACITY: usize = u32::MAX as usize + 1;

    #[expect(clippy::match_overlapping_arm)]
    match num_variants {
        0..=U8_CAPACITY => syn::parse_quote!(u8),
        0..=U16_CAPACITY => syn::parse_quote!(u16),
        0..=U32_CAPACITY => syn::parse_quote!(u32),
        _ => syn::parse_quote!(u64),
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
