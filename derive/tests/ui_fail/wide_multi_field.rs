use rust_spec::size::Wide;
use rust_spec_derive::RustSpec;
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
struct Pair {
    left: u8,
    right: u8,
}

fn main() {
    assert_impl_all!(Pair: Wide<Data = u8, Metadata = usize>);
}
