#[portrait::make]
trait MyTrait {
    fn with_lifetime<'a>(&'a self);
}
struct A {}

impl MyTrait for A {
    fn with_lifetime<'a>(&'a self) {
        println!("do nothing");
    }
}

struct B {
    inner: A,
}

#[portrait::fill(portrait::delegate(A; self.inner))] // Without this, the panic will not be triggered.
impl MyTrait for B {}
