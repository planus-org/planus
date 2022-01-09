check_type!(Example => value : Option<Vec<Inner>>);
check_type!(Example => value_null : Option<Vec<Inner>>);
check_type!(Example => value_default_empty : Vec<Inner>);
check_type!(Example => value_required : Vec<Inner>);
check_type!(Example => create(&mut planus::Builder, Vec<Inner>, Vec<Inner>, Vec<Inner>, Vec<Inner>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, (), (), Vec<Inner>, Vec<Inner>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<Vec<Inner>>, Option<Vec<Inner>>, Vec<Inner>, Vec<Inner>) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Builder, &'a [Inner], &'b [Inner], &'c [Inner], &'d [Inner]) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Builder, Option<&'a [Inner]>, Option<&'b [Inner]>, &'c [Inner], &'d [Inner]) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<planus::Vector<'a, planus::Result<InnerRef<'a>>>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<planus::Vector<'a, planus::Result<InnerRef<'a>>>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_empty() : planus::Result<planus::Vector<'a, planus::Result<InnerRef<'a>>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<planus::Vector<'a, planus::Result<InnerRef<'a>>>>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);
