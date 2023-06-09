#[portrait::make]
trait MyTrait {
    fn with_mut_arg(&self, mut _x: u32) {
        println!("with_mut_arg");
    }
}
struct A {}

impl MyTrait for A {}

struct B {
    inner: A,
}

#[portrait::fill(portrait::delegate(A; self.inner))] // Without this, the panic will not be triggered.
impl MyTrait for B {}
