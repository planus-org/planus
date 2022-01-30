check_type!(Example => { x: u32 , z: u32 });
check_type!(Example => create(&mut planus::Builder, u32, u32) : planus::Offset<Example>);

assert_traits!(Example: !Copy + Clone + Debug + Eq + Ord + Hash + Default);

check_type!(+['a] ExampleRef<'a> => &self.x() : planus::Result<u32>);
check_type!(+['a] ExampleRef<'a> => &self.z() : planus::Result<u32>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);
