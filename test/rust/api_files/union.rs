check_type!(+['a] Union => create_foo(&mut planus::Builder, &'a str) : planus::UnionOffset<Union>);
check_type!(+['a] Union => create_foo(&mut planus::Builder, &'a String) : planus::UnionOffset<Union>);
check_type!(Union => create_foo(&mut planus::Builder, String) : planus::UnionOffset<Union>);

check_type!(+['a] Union => create_bar(&mut planus::Builder, &'a InnerStruct) : planus::UnionOffset<Union>);
check_type!(+['a] Union => create_bar(&mut planus::Builder, InnerStruct) : planus::UnionOffset<Union>);

check_type!(+['a] Union => create_baz(&mut planus::Builder, &'a InnerTable) : planus::UnionOffset<Union>);
check_type!(+['a] Union => create_baz(&mut planus::Builder, InnerTable) : planus::UnionOffset<Union>);

check_type!(+['a] Union => create_inner_struct(&mut planus::Builder, &'a InnerStruct) : planus::UnionOffset<Union>);
check_type!(+['a] Union => create_inner_struct(&mut planus::Builder, InnerStruct) : planus::UnionOffset<Union>);

check_type!(+['a] Union => create_inner_table(&mut planus::Builder, &'a InnerTable) : planus::UnionOffset<Union>);
check_type!(+['a] Union => create_inner_table(&mut planus::Builder, InnerTable) : planus::UnionOffset<Union>);

check_type!(Union => Foo(String) : Union);
check_type!(Union => Bar(InnerStruct) : Union);
check_type!(Union => Baz(InnerTable) : Union);
check_type!(Union => InnerStruct(InnerStruct) : Union);
check_type!(Union => InnerTable(InnerTable) : Union);

assert_traits!(Union: !Copy + Clone + Debug + Eq + Ord + Hash + !Default);

check_type!(+['a] UnionRef<'a> => Foo(&'a str) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Bar(InnerStructRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Baz(InnerTableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => InnerStruct(InnerStructRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => InnerTable(InnerTableRef<'a>) : UnionRef<'a>);
