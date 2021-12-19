check_type!(Example => value : bool);
check_type!(Example => value_null : Option<bool>);
check_type!(Example => value_default_false : bool);
check_type!(Example => value_default_true : bool);
check_type!(Example => create(&mut planus::Buffer, bool, bool, bool, bool) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, bool, (), bool, bool) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, bool, Option<bool>, bool, bool) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<bool>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<bool>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_false() : planus::Result<bool>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_true() : planus::Result<bool>);
