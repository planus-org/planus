check_type!(Example => value : Option<Vec<u32>>);
check_type!(Example => value_null : Option<Vec<u32>>);
check_type!(Example => value_default_empty : Vec<u32>);
check_type!(Example => value_required : Vec<u32>);
check_type!(Example => create(&mut planus::Builder, Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, (), (), Vec<u32>, Vec<u32>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<Vec<u32>>, Option<Vec<u32>>, Vec<u32>, Vec<u32>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, planus::Offset<[u32]>, planus::Offset<[u32]>, planus::Offset<[u32]>, planus::Offset<[u32]>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<planus::Offset<[u32]>>, Option<planus::Offset<[u32]>>, planus::Offset<[u32]>, planus::Offset<[u32]>) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Builder, &'a [u32], &'b [u32], &'c [u32], &'d [u32]) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Builder, Option<&'a [u32]>, Option<&'b [u32]>, &'c [u32], &'d [u32]) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<planus::Vector<'a, u32>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<planus::Vector<'a, u32>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_empty() : planus::Result<planus::Vector<'a, u32>>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<planus::Vector<'a, u32>>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + Default,
    ExampleRef<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Example>} + !{Into<Example>},
);
