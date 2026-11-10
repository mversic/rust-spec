use rust_spec::RustSpec;

#[derive(RustSpec)]
pub struct Example<T>(pub (T, u32));

fn drop_helper<T: CombineDrop<()>>() {}
fn size_helper<T: AddSizedField<()>>() {}

fn main() {}
