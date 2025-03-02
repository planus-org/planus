check_type!(Example => value : Option<Vec<bool>>);
check_type!(Example => value_null : Option<Vec<bool>>);
check_type!(Example => value_default_empty : Vec<bool>);
check_type!(Example => value_required : Vec<bool>);
check_type!(Example => create(&mut planus::Builder, Vec<bool>, Vec<bool>, Vec<bool>, Vec<bool>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, (), (), Vec<bool>, Vec<bool>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<Vec<bool>>, Option<Vec<bool>>, Vec<bool>, Vec<bool>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, planus::Offset<[bool]>, planus::Offset<[bool]>, planus::Offset<[bool]>, planus::Offset<[bool]>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<planus::Offset<[bool]>>, Option<planus::Offset<[bool]>>, planus::Offset<[bool]>, planus::Offset<[bool]>) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Builder, &'a [bool], &'b [bool], &'c [bool], &'d [bool]) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Builder, Option<&'a [bool]>, Option<&'b [bool]>, &'c [bool], &'d [bool]) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<planus::Vector<'a, bool>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<planus::Vector<'a, bool>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_empty() : planus::Result<planus::Vector<'a, bool>>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<planus::Vector<'a, bool>>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + Default,
    ExampleRef<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Example>} + !{Into<Example>},
);
