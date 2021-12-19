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
