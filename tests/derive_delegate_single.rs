#[portrait::make]
trait Foo {
    fn foo(&self, arg1: i32, arg2: &str, arg3: &mut i64) -> bool;
}

#[portrait::fill(portrait::default)]
impl Foo for i32 {}

#[portrait::fill(portrait::default)]
impl Foo for String {}

#[portrait::derive(Foo with portrait::derive_delegate)]
enum Impls {
    A(i32),
    B(String),
}

fn main() {
    fn assert(_: impl Foo) {}

    assert(Impls::A(1));
}
