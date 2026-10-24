#![cfg(feature = "derive")]
#![allow(dead_code)]

use rust_spec::{
    Gt, One, RustSpec, Unstable,
    layout::{NonRobust, Robust},
    niche::{WithNiche, WithoutNiche},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
enum Sparse {
    A = 0,
    B = 256,
}

#[derive(RustSpec)]
enum Increment {
    A = 255,
    B,
}

#[derive(RustSpec)]
enum Negative {
    A = -129,
    B = -128,
}

#[derive(RustSpec)]
enum Arithmetic {
    A = (1 << 8) - 1,
    B,
    C = 0x200 | 3,
}

const SMALL: isize = 12;
const LARGE: isize = 256;
#[derive(RustSpec)]
enum Named {
    A = SMALL,
    B,
}
#[derive(RustSpec)]
enum SelfReference {
    A = 7,
    B = Self::A as isize + 1,
}
#[derive(RustSpec)]
enum Mixed {
    A = LARGE,
    B,
    C = 512,
}

// An unresolved reset must not be guessed as the previous value plus one.
const RESET: isize = 0;
#[derive(RustSpec)]
enum Reset {
    A = 255,
    B = RESET,
}

#[derive(RustSpec)]
enum MacroInput {
    A = stringify!(Self).len() as isize,
    B,
}

#[allow(non_upper_case_globals)]
const __rust_spec_discriminant: isize = 20;
#[derive(RustSpec)]
enum Hygienic {
    A = 10,
    B = __rust_spec_discriminant,
}

#[derive(RustSpec)]
#[repr(u16)]
enum Explicit {
    A = LARGE as u16,
    B,
}
#[derive(RustSpec)]
#[repr(u16)]
enum Data {
    A(u8) = LARGE as u16,
    B(u16),
}
#[derive(RustSpec)]
#[repr(C, u16)]
enum CData {
    A(u8) = LARGE as u16,
    B(u16),
}

macro_rules! exhaustive_enums {
    ($($variant:ident,)*) => {
        #[derive(RustSpec)]
        enum Full { First = 0, $($variant,)* }
        #[derive(RustSpec)]
        enum SignedFull { First = -128, $($variant,)* }
        #[derive(RustSpec)]
        enum Shifted { First = 256, $($variant,)* }
        #[derive(RustSpec)]
        #[repr(align(16))]
        enum AlignedFull { First = 0, $($variant,)* }
    };
}

exhaustive_enums! {
        V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12,
        V13, V14, V15, V16, V17, V18, V19, V20, V21, V22, V23, V24,
        V25, V26, V27, V28, V29, V30, V31, V32, V33, V34, V35, V36,
        V37, V38, V39, V40, V41, V42, V43, V44, V45, V46, V47, V48,
        V49, V50, V51, V52, V53, V54, V55, V56, V57, V58, V59, V60,
        V61, V62, V63, V64, V65, V66, V67, V68, V69, V70, V71, V72,
        V73, V74, V75, V76, V77, V78, V79, V80, V81, V82, V83, V84,
        V85, V86, V87, V88, V89, V90, V91, V92, V93, V94, V95, V96,
        V97, V98, V99, V100, V101, V102, V103, V104, V105, V106, V107, V108,
        V109, V110, V111, V112, V113, V114, V115, V116, V117, V118, V119, V120,
        V121, V122, V123, V124, V125, V126, V127, V128, V129, V130, V131, V132,
        V133, V134, V135, V136, V137, V138, V139, V140, V141, V142, V143, V144,
        V145, V146, V147, V148, V149, V150, V151, V152, V153, V154, V155, V156,
        V157, V158, V159, V160, V161, V162, V163, V164, V165, V166, V167, V168,
        V169, V170, V171, V172, V173, V174, V175, V176, V177, V178, V179, V180,
        V181, V182, V183, V184, V185, V186, V187, V188, V189, V190, V191, V192,
        V193, V194, V195, V196, V197, V198, V199, V200, V201, V202, V203, V204,
        V205, V206, V207, V208, V209, V210, V211, V212, V213, V214, V215, V216,
        V217, V218, V219, V220, V221, V222, V223, V224, V225, V226, V227, V228,
        V229, V230, V231, V232, V233, V234, V235, V236, V237, V238, V239, V240,
        V241, V242, V243, V244, V245, V246, V247, V248, V249, V250, V251, V252,
        V253, V254, V255,
}

#[test]
fn inferred_tag_tracks_values_and_implicit_increments() {
    assert_impl_all!(Sparse: RustSpec<Alignment = Gt<One>, Trap = NonRobust>);
    assert_impl_all!(Increment: RustSpec<Alignment = Gt<One>, Trap = NonRobust>);
    assert_impl_all!(Negative: RustSpec<Alignment = Gt<One>, Trap = NonRobust>);
    assert_impl_all!(Arithmetic: RustSpec<Alignment = Gt<One>, Trap = NonRobust>);
    assert_eq!(
        core::mem::align_of::<Sparse>(),
        core::mem::align_of::<u16>()
    );
    assert_eq!(
        core::mem::align_of::<Increment>(),
        core::mem::align_of::<u16>()
    );
    assert_eq!(
        core::mem::align_of::<Negative>(),
        core::mem::align_of::<i16>()
    );
}

#[test]
fn exhaustiveness_uses_inferred_width() {
    assert_impl_all!(Full: RustSpec<Alignment = One, Trap = Robust, Niche = WithoutNiche>);
    assert_impl_all!(SignedFull: RustSpec<Alignment = One, Trap = Robust, Niche = WithoutNiche>);
    assert_impl_all!(Shifted: RustSpec<Alignment = Gt<One>, Trap = NonRobust, Niche = WithNiche<Unstable>>);
    assert_impl_all!(AlignedFull: RustSpec<Alignment = Gt<One>, Trap = Robust, Niche = WithoutNiche>);
    assert_eq!(core::mem::size_of::<Shifted>(), 2);
}

#[test]
fn unresolved_values_are_checked_and_explicit_repr_is_respected() {
    assert_impl_all!(Named: RustSpec<Alignment = One, Trap = NonRobust>);
    assert_impl_all!(SelfReference: RustSpec<Alignment = One, Trap = NonRobust>);
    assert_impl_all!(Mixed: RustSpec<Alignment = Gt<One>, Trap = NonRobust>);
    assert_impl_all!(Reset: RustSpec<Alignment = One, Trap = NonRobust>);
    assert_impl_all!(MacroInput: RustSpec<Alignment = One, Trap = NonRobust>);
    assert_impl_all!(Hygienic: RustSpec<Alignment = One, Trap = NonRobust>);
    assert_impl_all!(Explicit: RustSpec<Alignment = Gt<One>, Trap = NonRobust>);
    assert_impl_all!(Data: RustSpec<Alignment = Gt<One>, Trap = NonRobust>);
    assert_impl_all!(CData: RustSpec<Alignment = Gt<One>, Trap = NonRobust>);
}
