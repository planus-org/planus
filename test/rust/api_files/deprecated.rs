check_type!(Example => x : u32);
check_type!(Example => z : u32);
check_type!(Example => create(&mut planus::Builder, u32, u32) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.x() : planus::Result<u32>);
check_type!(+['a] ExampleRef<'a> => &self.z() : planus::Result<u32>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);
