struct MyStruct {}

#[portrait::make()]
pub trait PassSelf {
    fn pass_self_by_mut_ref(&mut self);
    fn pass_self_by_ref(&self);
}

impl PassSelf for MyStruct {
    fn pass_self_by_mut_ref(&mut self) { println!("ok") }
    fn pass_self_by_ref(&self) { println!("ok") }
}

#[portrait::fill(portrait::delegate(T; *self))]
impl<T> PassSelf for Box<T> where T: PassSelf {}

#[portrait::make()]
pub trait PassSelfByValue: Sized {
    fn by_value(self);
    fn by_value_default(self) {}
}

impl PassSelfByValue for MyStruct {
    fn by_value(self) { println!("ok") }
}

#[portrait::fill(portrait::delegate(T; *self))]
impl<T> PassSelfByValue for Box<T> where T: PassSelfByValue {}
