#![allow(dead_code)]

use rust_spec::{RustSpec, drop::{CustomDrop, NoDrop, AutoDrop}};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
struct Generic<T: RustSpec>(T);

impl<T: RustSpec> Drop for Generic<T> {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
#[repr(transparent)]
struct Transparent(u8);

impl Drop for Transparent {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
#[repr(C)]
enum DataEnum {
    Value(u8),
}

impl Drop for DataEnum {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
#[repr(u8)]
enum FieldlessEnum {
    Value,
}

impl Drop for FieldlessEnum {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
union ValueUnion {
    value: u8,
}

impl Drop for ValueUnion {
    fn drop(&mut self) {}
}

fn main() {
    assert_impl_all!(Generic<u8>: RustSpec<Drop = CustomDrop<NoDrop>>);
    assert_impl_all!(Generic<Transparent>: RustSpec<Drop = CustomDrop<AutoDrop>>);
    assert_impl_all!(Transparent: RustSpec<Drop = CustomDrop<NoDrop>>);
    assert_impl_all!(DataEnum: RustSpec<Drop = CustomDrop<NoDrop>>);
    assert_impl_all!(FieldlessEnum: RustSpec<Drop = CustomDrop<NoDrop>>);
    assert_impl_all!(ValueUnion: RustSpec<Drop = CustomDrop<NoDrop>>);
}
