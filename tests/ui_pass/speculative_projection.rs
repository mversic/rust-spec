use rust_spec::RustSpec;

trait Wide { type Header; }
struct Opaque;

#[derive(RustSpec)]
#[repr(C)]
struct Data(u32, <Opaque as Wide>::Header)
where
    for<'unused> Opaque: Wide;

fn main() {}
