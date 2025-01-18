#[portrait::make]
trait Foo {
    fn new(arg1: i32, arg2: &str, arg3: &mut i64) -> Self;
    fn print(&self);
    fn clone(&self) -> Self;
    #[portrait(derive_delegate(reduce = |a, b| a && b))]
    fn eq(&self, other: &Self) -> bool;
}

#[portrait::fill(portrait::default)]
impl Foo for i32 { }

#[portrait::fill(portrait::default)]
impl Foo for String {}

#[portrait::derive(Foo with portrait::derive_delegate)]
struct Fields {
    a: i32,
    b: String,
}

fn main() {
    fn assert(_: impl Foo) {}

    assert(Fields { a: 1, b: String::new() });
}
