check_type!(Example => value : Inner);
check_type!(Example => value_null : Option<Inner>);
check_type!(Example => value_default_x : Inner);
check_type!(Example => value_default_y : Inner);
check_type!(Example => create(&mut planus::Buffer, Inner, Inner, Inner, Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, Inner, (), Inner, Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, Inner, Option<Inner>, Inner, Inner) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Inner>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<Inner>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_x() : planus::Result<Inner>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_y() : planus::Result<Inner>);

check_type!(Example2 => value_null : Option<Inner2>);
check_type!(Example2 => value_default_x : Inner2);
check_type!(Example2 => value_default_y : Inner2);
check_type!(Example2 => create(&mut planus::Buffer, Inner2, Inner2, Inner2) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Buffer, (), Inner2, Inner2) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Buffer, Option<Inner2>, Inner2, Inner2) : planus::Offset<Example2>);

check_type!(+['a] Example2Ref<'a> => &self.value_null() : planus::Result<Option<Inner2>>);
check_type!(+['a] Example2Ref<'a> => &self.value_default_x() : planus::Result<Inner2>);
check_type!(+['a] Example2Ref<'a> => &self.value_default_y() : planus::Result<Inner2>);
