#[portrait::make()]
pub trait X {
    fn m(&mut self);
}

struct MyStruct {}

impl X for MyStruct {
    fn m(&mut self) {
        println!("ok")
    }
}

#[portrait::fill(portrait::delegate(T; self))]
impl<T> X for Box<T> where T: X {}
