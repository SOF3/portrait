use either::Either;

#[portrait::make]
trait Foo {
    #[portrait(derive_delegate(enum_either))]
    fn foo(&self, non_copy: String) -> impl Iterator<Item = i32> + DoubleEndedIterator;
}

impl Foo for i32 {
    fn foo(&self, _non_copy: String) -> impl Iterator<Item = i32> + DoubleEndedIterator {
        [1, 2].into_iter()
    }
}

impl Foo for String {
    fn foo(&self, _non_copy: String) -> impl Iterator<Item = i32> + DoubleEndedIterator {
        std::iter::repeat(5).take(5)
    }
}

#[portrait::derive(Foo with portrait::derive_delegate)]
enum Impls {
    A(i32),
    B(i32),
    C(String),
    D(String),
}

fn main() {
    fn assert(_: impl Foo) {}

    assert(Impls::A(1));
}
