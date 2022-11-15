check_type!(Example => { });
check_type!(Example => create(&mut planus::Builder) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    ExampleRef: Copy + Clone + Debug + !Eq + !Ord + !Hash + !Default + {TryInto<Example>} + !{Into<Example>},
);
