check_type!(Example => value : Option<Inner>);
check_type!(Example => value_null : Option<Inner>);
check_type!(Example => value_required : Inner);
check_type!(Example => create(&mut planus::Builder, Inner, Inner, Inner) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c] Example => create(&mut planus::Builder, &'a Inner, &'b Inner, &'c Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Box<Inner>, Box<Inner>, Box<Inner>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<Box<Inner>>, Option<Box<Inner>>, Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<Inner>, Option<Inner>, Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, (), (), Inner) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<InnerRef<'a>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<InnerRef<'a>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<InnerRef<'a>>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    ExampleRef<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Example>} + !{Into<Example>},
);

check_type!(Example2 => value : Option<Inner2>);
check_type!(Example2 => value_null : Option<Inner2>);
check_type!(Example2 => value_required : Inner2);
check_type!(Example2 => create(&mut planus::Builder, Inner2, Inner2, Inner2) : planus::Offset<Example2>);
check_type!(+['a, 'b, 'c] Example2 => create(&mut planus::Builder, &'a Inner2, &'b Inner2, &'c Inner2) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, Box<Inner2>, Box<Inner2>, Box<Inner2>) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, Option<Box<Inner2>>, Option<Box<Inner2>>, Inner2) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, Option<Inner2>, Option<Inner2>, Inner2) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, (), (), Inner2) : planus::Offset<Example2>);

check_type!(+['a] Example2Ref<'a> => &self.value() : planus::Result<Option<Inner2Ref>>);
check_type!(+['a] Example2Ref<'a> => &self.value_null() : planus::Result<Option<Inner2Ref>>);
check_type!(+['a] Example2Ref<'a> => &self.value_required() : planus::Result<Inner2Ref>);
check_type!(+['a] Example2Ref<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example2: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    Example2Ref<'_>: Copy + Clone + Debug + !PartialEq + !PartialEq + !Eq + !Ord + !Hash + !Default + {TryInto<Example2>} + !{Into<Example2>},
);
