check_type!(+['a] Union => create_foo(&mut planus::Builder, &'a str) : planus::UnionOffset<Union>);
check_type!(+['a] Union => create_foo(&mut planus::Builder, &'a String) : planus::UnionOffset<Union>);
check_type!(Union => create_foo(&mut planus::Builder, String) : planus::UnionOffset<Union>);
check_type!(Union => Foo(String) : Union);

assert_traits!(Union: !Copy + Clone + Debug + Eq + Ord + Hash + !Default);

check_type!(+['a] UnionRef<'a> => Foo(&'a str) : UnionRef<'a>);
