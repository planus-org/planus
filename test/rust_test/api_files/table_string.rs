check_type!(Example => value : Option<String>);
check_type!(Example => value_null : Option<String>);
check_type!(Example => value_default_empty : String);
check_type!(Example => value_default_test : String);
check_type!(Example => value_required : String);
check_type!(Example => create(&mut planus::Builder, String, String, String, String, String) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, (), (), String, String, String) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<String>, Option<String>, String, String, String) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd, 'e] Example => create(&mut planus::Builder, &'a str, &'b str, &'c str, &'d str, &'e str) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd, 'e] Example => create(&mut planus::Builder, Option<&'a str>, Option<&'b str>, &'c str, &'d str, &'e str) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<&'a str>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<&'a str>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_empty() : planus::Result<&'a str>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_test() : planus::Result<&'a str>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<&'a str>);
