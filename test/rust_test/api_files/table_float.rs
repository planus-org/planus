check_type!(Example => value : f32);
check_type!(Example => value_null : Option<f32>);
check_type!(Example => value_default_zero : f32);
check_type!(Example => value_default_one : f32);
check_type!(Example => create(&mut planus::Buffer, f32, f32, f32, f32) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, f32, (), f32, f32) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, f32, Option<f32>, f32, f32) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<f32>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<f32>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_zero() : planus::Result<f32>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_one() : planus::Result<f32>);
