use planus::exports::{str as pstr, String as PString};

check_type!(Example => value : Option<PString>);
check_type!(Example => value_null : Option<PString>);
check_type!(Example => value_default_empty : PString);
check_type!(Example => value_default_test : PString);
check_type!(Example => value_required : PString);
check_type!(Example => create(&mut planus::Builder, String, String, String, String, String) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, (), (), String, String, String) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<String>, Option<String>, String, String, String) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, PString, PString, PString, PString, PString) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, (), (), PString, PString, PString) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<PString>, Option<PString>, PString, PString, PString) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd, 'e] Example => create(&mut planus::Builder, &'a str, &'b str, &'c str, &'d str, &'e str) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd, 'e] Example => create(&mut planus::Builder, Option<&'a str>, Option<&'b str>, &'c str, &'d str, &'e str) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd, 'e] Example => create(&mut planus::Builder, &'a pstr, &'b pstr, &'c pstr, &'d pstr, &'e pstr) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd, 'e] Example => create(&mut planus::Builder, Option<&'a pstr>, Option<&'b pstr>, &'c pstr, &'d pstr, &'e pstr) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<&'a pstr>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<&'a pstr>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_empty() : planus::Result<&'a pstr>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_test() : planus::Result<&'a pstr>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<&'a pstr>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(Example: !Copy + Clone + Debug + Eq + Ord + Hash + !Default);

check_type!(ExampleDefault => value_default_empty : PString);
check_type!(ExampleDefault => value_default_test : PString);
check_type!(ExampleDefault => create(&mut planus::Builder, String, String) : planus::Offset<ExampleDefault>);
check_type!(ExampleDefault => create(&mut planus::Builder, PString, PString) : planus::Offset<ExampleDefault>);
check_type!(+['a, 'b] ExampleDefault => create(&mut planus::Builder, &'a str, &'b str) : planus::Offset<ExampleDefault>);
check_type!(+['a, 'b] ExampleDefault => create(&mut planus::Builder, &'a pstr, &'b pstr) : planus::Offset<ExampleDefault>);

check_type!(+['a] ExampleDefaultRef<'a> => &self.value_default_empty() : planus::Result<&'a pstr>);
check_type!(+['a] ExampleDefaultRef<'a> => &self.value_default_test() : planus::Result<&'a pstr>);
check_type!(+['a] ExampleDefaultRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(ExampleDefault: !Copy + Clone + Debug + Eq + Ord + Hash + Default);
