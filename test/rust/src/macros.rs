#[allow(unused_macros)] // rust-analyzer is being weird
macro_rules! check_type {
    ($(+[ $($l:lifetime),* ])? $obj:ty => &self.$method:ident ($($arg:ty),* $(,)?) : $return:ty) => {
        check_type!($(+[ $($l),* ])? $obj => $method(&$obj, $($arg),*) : $return);
        #[allow(unused_unsafe)]
        const _: for<'a> fn(&'a $obj) = |obj| unsafe {
            let _ = obj.$method(
                $(
                    std::mem::zeroed::<$arg>()
                ),*
            );
        };
    };
    // This code is currently not used in any tests, but we consider it highly likely that it will
    // be needed at some point in the future. It has been commented out to silence warnings.
    /*
    ($(+[ $($l:lifetime),* ])? $obj:ty => &mut self.$method:ident ($($arg:ty),* $(,)?) : $return:ty) => {
        check_type!($(+[ $($l),* ])? $obj => $method(&mut $obj, $($arg),*) : $return);
        #[allow(unused_unsafe)]
        const _: for<'a> fn(&'a mut $obj) = |obj| unsafe {
            obj.$method(
                $(
                    std::mem::zeroed::<$arg>()
                ),*
            );
        };
    };
    ($(+[ $($l:lifetime),* ])? $obj:ty => self.$method:ident ($($arg:ty),* $(,)?) : $return:ty) => {
        check_type!($(+[ $($l),* ])? $obj => $method($obj, $($arg),*) : $return);
        #[allow(unused_unsafe)]
        const _: fn($obj) = |obj| unsafe {
            obj.$method(
                $(
                    std::mem::zeroed::<$arg>()
                ),*
            );
        };
    };
     */
    ($(+[ $($l:lifetime),* ])? $obj:ty => $method:ident($($arg:ty),* $(,)?) : $return:ty) => {
        const _: () = {
            trait HasMethod$(< $($l),* > )? {
                const METHOD: fn( $($arg),* ) -> $return;
            }
            impl$(< $($l),* > )? HasMethod$(< $($l),* > )? for $obj {
                const METHOD: fn( $($arg),* ) -> $return = Self::$method;
            }
        };
    };
    ($(+[ $($l:lifetime),* ])? $obj:ty => impl $($trait_to_impl:tt)*) => {
        const _: () = {
            fn assert_impl<$($($l,)*)? T: $($trait_to_impl)*>() {}
            fn helper$(<$($l),*>)?($($(_: &$l()),*)?) {
                assert_impl::<$obj>();
            }
        };
    };
    ($obj:ty => $field:ident : $field_type:ty) => {
        const _: fn($obj) = |obj| {
            let _: $field_type = obj.$field;
        };
    };
    ($obj:ident => { $($field:ident : $field_type:ty),* $(,)?}) => {
        const _: fn($obj) = |obj| {
            let $obj { $($field),* } = obj;
            $(
                let _: $field_type = $field;
            )*
        };
    }
}

#[allow(unused_macros)] // rust-analyzer is being weird
macro_rules! check_enum_variants {
    ($obj:ty : $typ:ty {
        $($name:ident = $value:expr),* $(,)?
    }) => {
        const _: fn($typ) -> Result<$obj, planus::errors::UnknownEnumTagKind> = std::convert::TryFrom::try_from;
        const _: fn($obj) -> $typ = std::convert::From::from;
        #[allow(clippy::match_single_binding)]
        const _: fn($obj) -> ! = |obj| match obj {
            $(
                <$obj>::$name => todo!()
            ),*
        };
        const _: () = {
            $(
                assert!(<$obj>::$name as $typ == $value);
                assert!(unsafe { std::mem::transmute::<$obj, $typ>(<$obj>::$name) } == $value);
            )*
        };
        $(
            assert_eq!(<$typ>::from(<$obj>::$name), $value);
            assert_eq!(<$obj>::try_from($value).unwrap(), <$obj>::$name);
        )*
    }
}

#[allow(unused_macros)] // rust-analyzer is being weird
macro_rules! assert_traits {
    ($obj:ty : $trait:ident $($more:tt)*) => {
        static_assertions::assert_impl_all!($obj: $trait);
        assert_traits!($obj : $($more)*);
    };
    ($obj:ty : !$trait:ident $($more:tt)*) => {
        static_assertions::assert_not_impl_all!($obj: $trait);
        assert_traits!($obj : $($more)*);
    };
    ($obj:ty : + $($more:tt)*) => {
        assert_traits!($obj : $($more)*);
    };
    ($obj:ty : , $($more:tt)*) => {
        assert_traits!($($more)*);
    };
    ($obj:ty :) => {};
    () => {};
}
