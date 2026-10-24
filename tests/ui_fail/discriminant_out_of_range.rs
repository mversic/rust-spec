use rust_spec::RustSpec;

const LARGE: isize = 256;
const EDGE: isize = 255;
const NEGATIVE: isize = -1;

#[derive(RustSpec)]
enum TooWide { A = LARGE, B }

#[derive(RustSpec)]
enum IncrementTooWide { A = EDGE, B }

#[derive(RustSpec)]
enum NegativeUnknown { A = NEGATIVE, B }

fn main() {}
