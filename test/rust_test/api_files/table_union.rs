check_type!(Example => value : Option<Inner>);
check_type!(Example => value_null : Option<Inner>);
check_type!(Example => value_required : Inner);
check_type!(Example => create(&mut planus::Buffer, Inner, Inner, Inner) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c] Example => create(&mut planus::Buffer, &'a Inner, &'b Inner, &'c Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, Box<Inner>, Box<Inner>, Box<Inner>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, Option<Box<Inner>>, Option<Box<Inner>>, Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, Option<Inner>, Option<Inner>, Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, (), (), Inner) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<InnerRef<'a>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<InnerRef<'a>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<InnerRef<'a>>);

check_type!(Example2 => value : Option<Inner2>);
check_type!(Example2 => value_null : Option<Inner2>);
check_type!(Example2 => value_required : Inner2);
check_type!(Example2 => create(&mut planus::Buffer, Inner2, Inner2, Inner2) : planus::Offset<Example2>);
check_type!(+['a, 'b, 'c] Example2 => create(&mut planus::Buffer, &'a Inner2, &'b Inner2, &'c Inner2) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Buffer, Box<Inner2>, Box<Inner2>, Box<Inner2>) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Buffer, Option<Box<Inner2>>, Option<Box<Inner2>>, Inner2) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Buffer, Option<Inner2>, Option<Inner2>, Inner2) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Buffer, (), (), Inner2) : planus::Offset<Example2>);

check_type!(+['a] Example2Ref<'a> => &self.value() : planus::Result<Option<Inner2Ref>>);
check_type!(+['a] Example2Ref<'a> => &self.value_null() : planus::Result<Option<Inner2Ref>>);
check_type!(+['a] Example2Ref<'a> => &self.value_required() : planus::Result<Inner2Ref>);
