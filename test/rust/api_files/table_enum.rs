check_type!(Example => value : Inner);
check_type!(Example => value_null : Option<Inner>);
check_type!(Example => value_default_x : Inner);
check_type!(Example => value_default_y : Inner);
check_type!(Example => create(&mut planus::Builder, Inner, Inner, Inner, Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Inner, (), Inner, Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Inner, Option<Inner>, Inner, Inner) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Inner>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<Inner>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_x() : planus::Result<Inner>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_y() : planus::Result<Inner>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    ExampleRef: Copy + Clone + Debug + !Eq + !Ord + !Hash + !Default + {TryInto<Example>} + !{Into<Example>},
);

check_type!(Example2 => value_null : Option<Inner2>);
check_type!(Example2 => value_default_x : Inner2);
check_type!(Example2 => value_default_y : Inner2);
check_type!(Example2 => create(&mut planus::Builder, Inner2, Inner2, Inner2) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, (), Inner2, Inner2) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, Option<Inner2>, Inner2, Inner2) : planus::Offset<Example2>);

check_type!(+['a] Example2Ref<'a> => &self.value_null() : planus::Result<Option<Inner2>>);
check_type!(+['a] Example2Ref<'a> => &self.value_default_x() : planus::Result<Inner2>);
check_type!(+['a] Example2Ref<'a> => &self.value_default_y() : planus::Result<Inner2>);
check_type!(+['a] Example2Ref<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example2: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + Default,
    Example2Ref<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Example2>} + !{Into<Example2>},
);
