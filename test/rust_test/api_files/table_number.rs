check_type!(Example => value : u32);
check_type!(Example => value_null : Option<u32>);
check_type!(Example => value_default_zero : u32);
check_type!(Example => value_default_one : u32);
check_type!(Example => create(&mut planus::Buffer, u32, u32, u32, u32) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, u32, (), u32, u32) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, u32, Option<u32>, u32, u32) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<u32>);
