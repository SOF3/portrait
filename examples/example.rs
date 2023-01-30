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
    }
}

mod user {
    use crate::def::{definition_portrait, Definition};

    struct User;

    #[portrait::fill(portrait::default)]
    impl Definition for User {}
}

fn main() {}
