check_type!(Example => { x: u32 , z: u32 });
check_type!(Example => create(&mut planus::Builder, u32, u32) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.x() : planus::Result<u32>);
check_type!(+['a] ExampleRef<'a> => &self.z() : planus::Result<u32>);
check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    ExampleRef<'_>: Copy + Clone + Debug + !Eq + !Ord + !Hash + !Default + {TryInto<Example>} + !{Into<Example>},
);
