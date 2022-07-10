check_type!(Example => { x: i32, y: u32 });
check_type!(Example => create(&mut planus::Builder, i32, u32) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.x() : planus::Result<i32>);
check_type!(+['a] ExampleRef<'a> => &self.y() : planus::Result<u32>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    ExampleRef: Copy + Clone + Debug + !Eq + !Ord + !Hash + !Default + {TryInto<Example>} + !{Into<Example>},
);

check_type!(Example2 => { x: i32, y: Option<U> });
check_type!(Example2 => create(&mut planus::Builder, i32, U) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, i32, ()) : planus::Offset<Example2>);

check_type!(+['a] Example2Ref<'a> => &self.x() : planus::Result<i32>);
check_type!(+['a] Example2Ref<'a> => &self.y() : planus::Result<Option<URef>>);
check_type!(+['a] Example2Ref<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example2: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + Default,
    Example2Ref<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Example2>} + !{Into<Example2>},
);
