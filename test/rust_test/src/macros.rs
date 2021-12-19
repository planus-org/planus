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
    ($obj:ty => $field:ident : $field_type:ty) => {
        const _: fn($obj) = |obj| {
            let _: $field_type = obj.$field;
        };
    }
}
