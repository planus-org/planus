check_type!(Example => value : Option<Vec<i8>>);
check_type!(Example => value_null : Option<Vec<i8>>);
check_type!(Example => value_default_empty : Vec<i8>);
check_type!(Example => value_required : Vec<i8>);
check_type!(Example => create(&mut planus::Builder, Vec<i8>, Vec<i8>, Vec<i8>, Vec<i8>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, (), (), Vec<i8>, Vec<i8>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<Vec<i8>>, Option<Vec<i8>>, Vec<i8>, Vec<i8>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, planus::Offset<[i8]>, planus::Offset<[i8]>, planus::Offset<[i8]>, planus::Offset<[i8]>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<planus::Offset<[i8]>>, Option<planus::Offset<[i8]>>, planus::Offset<[i8]>, planus::Offset<[i8]>) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Builder, &'a [i8], &'b [i8], &'c [i8], &'d [i8]) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Builder, Option<&'a [i8]>, Option<&'b [i8]>, &'c [i8], &'d [i8]) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<&'a [i8]>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<&'a [i8]>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_empty() : planus::Result<&'a [i8]>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<&'a [i8]>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + Default,
    ExampleRef<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Example>} + !{Into<Example>},
);

check_type!(Example2 => value : Option<Vec<i8>>);
check_type!(Example2 => value_null : Option<Vec<i8>>);
check_type!(Example2 => value_default_empty : Vec<i8>);
check_type!(Example2 => create(&mut planus::Builder, Vec<i8>, Vec<i8>, Vec<i8>) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, (), (), Vec<i8>) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, Option<Vec<i8>>, Option<Vec<i8>>, Vec<i8>) : planus::Offset<Example2>);
check_type!(+['a, 'b, 'c] Example2 => create(&mut planus::Builder, &'a [i8], &'b [i8], &'c [i8]) : planus::Offset<Example2>);
check_type!(+['a, 'b, 'c] Example2 => create(&mut planus::Builder, Option<&'a [i8]>, Option<&'b [i8]>, &'c [i8]) : planus::Offset<Example2>);

check_type!(+['a] Example2Ref<'a> => &self.value() : planus::Result<Option<&'a [i8]>>);
check_type!(+['a] Example2Ref<'a> => &self.value_null() : planus::Result<Option<&'a [i8]>>);
check_type!(+['a] Example2Ref<'a> => &self.value_default_empty() : planus::Result<&'a [i8]>);
check_type!(+['a] Example2Ref<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example2: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + Default,
    Example2Ref<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Example2>} + !{Into<Example2>},
);
