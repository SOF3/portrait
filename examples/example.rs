mod def {
    use core::num;
    use std::net::Ipv4Addr;
    use std::{cell, string};

    #[portrait::make(import(
        core::num,
        std::net::Ipv4Addr,
        std::{cell, string},
    ))]
    pub trait Definition {
        fn foo() -> i32;
        fn bar(&self) -> u32;
        fn qux(
            &mut self,
            ip: Ipv4Addr,
            val: cell::RefMut<'_, (num::NonZeroU8,)>,
        ) -> std::collections::BTreeSet<string::String>;

        type Corge<T>;
    }
}

mod user {
    use crate::def::{definition_portrait, Definition};

    struct DefaultUser<T>(T);

    #[portrait::fill(portrait::default)]
    impl<U> Definition for DefaultUser<U> {
        // portrait::default cannot fill types because there is no such thing as "default type".
        type Corge<T> = Option<T>;
    }

    struct DelegateUser<T> {
        inner: DefaultUser<T>,
    }

    #[portrait::fill(portrait::delegate(DefaultUser<U>; self.inner))]
    impl<U> Definition for DelegateUser<U> {}
}

fn main() {}
