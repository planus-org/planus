check_type!(Example => value : Option<Vec<String>>);
check_type!(Example => value_null : Option<Vec<String>>);
check_type!(Example => value_default_empty : Vec<String>);
check_type!(Example => value_required : Vec<String>);
check_type!(Example => create(&mut planus::Buffer, Vec<String>, Vec<String>, Vec<String>, Vec<String>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, (), (), Vec<String>, Vec<String>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, Option<Vec<String>>, Option<Vec<String>>, Vec<String>, Vec<String>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, Vec<planus::Offset<str>>, Vec<planus::Offset<str>>, Vec<planus::Offset<str>>, Vec<planus::Offset<str>>) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Buffer, &'a [&'a str], &'b [&'b str], &'c [&'c str], &'d [&'d str]) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Buffer, Option<&'a [&'a str]>, Option<&'b [&'b str]>, &'c [&'c str], &'d [&'d str]) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Buffer, Option<&'a [planus::Offset<str>]>, Option<&'b [planus::Offset<str>]>, &'c [planus::Offset<str>], &'d [planus::Offset<str>]) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<planus::Vector<'a, str>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<planus::Vector<'a, str>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_empty() : planus::Result<planus::Vector<'a, str>>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<planus::Vector<'a, str>>);
