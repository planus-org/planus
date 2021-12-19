macro_rules! check_type {
    ($obj:ty => &self.$method:ident ($($arg:ty),* $(,)?) : $return:ty) => {
        check_type!($obj => $method(&$obj, $($arg),*) : $return);
        #[allow(unused_unsafe)]
        const _: for<'a> fn(&'a $obj) = |obj| unsafe {
            obj.$method(
                $(
                    std::mem::zeroed::<$arg>()
                ),*
            );
        };
    };
    ($obj:ty => &mut self.$method:ident ($($arg:ty),* $(,)?) : $return:ty) => {
        check_type!($obj => $method(&mut $obj, $($arg),*) : $return);
        #[allow(unused_unsafe)]
        const _: for<'a> fn(&'a mut $obj) = |obj| unsafe {
            obj.$method(
                $(
                    std::mem::zeroed::<$arg>()
                ),*
            );
        };
    };
    ($obj:ty => self.$method:ident ($($arg:ty),* $(,)?) : $return:ty) => {
        check_type!($obj => $method($obj, $($arg),*) : $return);
        #[allow(unused_unsafe)]
        const _: fn($obj) = |obj| unsafe {
            obj.$method(
                $(
                    std::mem::zeroed::<$arg>()
                ),*
            );
        };
    };
    ($obj:ty => $method:ident($($arg:ty),* $(,)?) : $return:ty) => {
        const _: fn(
                $($arg),*
            ) -> $return = <$obj>::$method;
    };
    ($obj:ty => $field:ident : $field_type:ty) => {
        const _: fn($obj) = |obj| {
            // This uses autoref-based stable specialization based on
            // https://github.com/dtolnay/case-studies/tree/master/autoref-specialization
            struct CorrectTag;
            trait Correct {
                fn tag(&self) -> CorrectTag {
                    CorrectTag
                }
            }

            impl Correct for $field_type {}

            struct WrongTag;
            trait Wrong {
                fn tag(&self) -> WrongTag;
            }

            impl<T> Wrong for &&&&T {
                fn tag(&self) -> WrongTag{
                    WrongTag
                }
            }

            let tag = (&obj.$field).tag();
            let _: CorrectTag = tag;
        };
    }
}
