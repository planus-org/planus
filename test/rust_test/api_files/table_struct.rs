check_type!(Example => value : Option<Inner>);
check_type!(Example => value_null : Option<Inner>);
check_type!(Example => value_required : Inner);
check_type!(Example => create(&mut planus::Builder, Inner, Inner, Inner) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c] Example => create(&mut planus::Builder, &'a Inner, &'b Inner, &'c Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Box<Inner>, Box<Inner>, Box<Inner>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<Box<Inner>>, Option<Box<Inner>>, Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, Option<Inner>, Option<Inner>, Inner) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Builder, (), (), Inner) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<InnerRef<'a>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<InnerRef<'a>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<InnerRef<'a>>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);
