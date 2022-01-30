check_type!(Example => { x: i32, y: u32 });
check_type!(Example => create(&mut planus::Builder, i32, u32) : planus::Offset<Example>);

assert_traits!(Example: !Copy + Clone + Debug + Eq + Ord + Hash + Default);

check_type!(+['a] ExampleRef<'a> => &self.x() : planus::Result<i32>);
check_type!(+['a] ExampleRef<'a> => &self.y() : planus::Result<u32>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);

check_type!(Example2 => { x: i32, y: Option<U> });
check_type!(Example2 => create(&mut planus::Builder, i32, U) : planus::Offset<Example2>);
check_type!(Example2 => create(&mut planus::Builder, i32, ()) : planus::Offset<Example2>);

assert_traits!(Example2: !Copy + Clone + Debug + Eq + Ord + Hash + Default);

check_type!(+['a] Example2Ref<'a> => &self.x() : planus::Result<i32>);
check_type!(+['a] Example2Ref<'a> => &self.y() : planus::Result<Option<URef>>);
check_type!(+['a] Example2Ref<'a> => impl planus::ReadAsRoot<'a>);
