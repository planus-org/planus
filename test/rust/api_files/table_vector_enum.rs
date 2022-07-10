check_type!(Example => value : Option<Vec<Inner>>);
check_type!(Example => value_null : Option<Vec<Inner>>);
check_type!(Example => value_default_empty : Vec<Inner>);
check_type!(Example => value_required : Vec<Inner>);
check_type!(Example => create(&mut planus::Builder, Vec<Inner>, Vec<Inner>, Vec<Inner>, Vec<Inner>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, (), (), Vec<Inner>, Vec<Inner>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<Vec<Inner>>, Option<Vec<Inner>>, Vec<Inner>, Vec<Inner>) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Builder, &'a [Inner], &'b [Inner], &'c [Inner], &'d [Inner]) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Builder, Option<&'a [Inner]>, Option<&'b [Inner]>, &'c [Inner], &'d [Inner]) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<planus::Vector<'a, Result<Inner, planus::errors::UnknownEnumTag>>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<planus::Vector<'a, Result<Inner, planus::errors::UnknownEnumTag>>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_empty() : planus::Result<planus::Vector<'a, Result<Inner, planus::errors::UnknownEnumTag>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<planus::Vector<'a, Result<Inner, planus::errors::UnknownEnumTag>>>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    ExampleRef<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Example>} + !{Into<Example>},
);

check_type!(Example2 => value : Option<Vec<Inner2>>);
check_type!(Example2 => value_null : Option<Vec<Inner2>>);
check_type!(Example2 => value_default_empty : Vec<Inner2>);
check_type!(Example2 => value_required : Vec<Inner2>);
check_type!(Example2 => create(&mut planus::Builder, Vec<Inner2>, Vec<Inner2>, Vec<Inner2>, Vec<Inner2>) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, (), (), Vec<Inner2>, Vec<Inner2>) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, Option<Vec<Inner2>>, Option<Vec<Inner2>>, Vec<Inner2>, Vec<Inner2>) : planus::Offset<Example2>);
check_type!(+['a, 'b, 'c, 'd] Example2 => create(&mut planus::Builder, &'a [Inner2], &'b [Inner2], &'c [Inner2], &'d [Inner2]) : planus::Offset<Example2>);
check_type!(+['a, 'b, 'c, 'd] Example2 => create(&mut planus::Builder, Option<&'a [Inner2]>, Option<&'b [Inner2]>, &'c [Inner2], &'d [Inner2]) : planus::Offset<Example2>);

check_type!(+['a] Example2Ref<'a> => &self.value() : planus::Result<Option<planus::Vector<'a, Result<Inner2, planus::errors::UnknownEnumTag>>>>);
check_type!(+['a] Example2Ref<'a> => &self.value_null() : planus::Result<Option<planus::Vector<'a, Result<Inner2, planus::errors::UnknownEnumTag>>>>);
check_type!(+['a] Example2Ref<'a> => &self.value_default_empty() : planus::Result<planus::Vector<'a, Result<Inner2, planus::errors::UnknownEnumTag>>>);
check_type!(+['a] Example2Ref<'a> => &self.value_required() : planus::Result<planus::Vector<'a, Result<Inner2, planus::errors::UnknownEnumTag>>>);
check_type!(+['a] Example2Ref<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example2: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    Example2Ref<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Example2>} + !{Into<Example2>},
);
