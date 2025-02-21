check_type!(Empty => { });
check_type!(Empty => create(&mut planus::Builder) : planus::Offset<Empty>);

check_type!(+['a] EmptyRef<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Empty: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    EmptyRef: Copy + Clone + Debug + !Eq + !Ord + !Hash + !Default + {TryInto<Empty>} + !{Into<Empty>},
);
