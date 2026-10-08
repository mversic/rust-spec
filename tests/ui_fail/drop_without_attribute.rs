#![allow(dead_code)]

use rust_spec::RustSpec;

#[derive(RustSpec)]
struct RustStruct(u8);

impl Drop for RustStruct {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
#[repr(C)]
struct ReprCStruct(u8);

impl Drop for ReprCStruct {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
#[repr(transparent)]
struct TransparentStruct(u8);

impl Drop for TransparentStruct {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
struct Generic<T>(core::marker::PhantomData<T>);

impl<T> Drop for Generic<T> {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
#[repr(transparent)]
enum TransparentEnum {
    Value(u8),
}

impl Drop for TransparentEnum {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
enum RustDataEnum {
    Value(u8),
    Empty,
}

impl Drop for RustDataEnum {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
#[repr(u8)]
enum PrimitiveDataEnum {
    Value(u8),
    Empty,
}

impl Drop for PrimitiveDataEnum {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
#[repr(u8)]
enum FieldlessEnum {
    Value,
    Empty,
}

impl Drop for FieldlessEnum {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
#[repr(C)]
enum DataEnum {
    Value(u8),
    Empty,
}

impl Drop for DataEnum {
    fn drop(&mut self) {}
}

#[derive(RustSpec)]
union ValueUnion {
    value: u8,
}

impl Drop for ValueUnion {
    fn drop(&mut self) {}
}

fn main() {}
