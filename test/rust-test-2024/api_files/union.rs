// Union1
check_type!(+['a] Union1 => create_f0(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union1>);
check_type!(Union1 => create_f0(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union1>);

check_type!(+['a] Union1 => create_f1(&mut planus::Builder, &'a InnerF32) : planus::UnionOffset<Union1>);
check_type!(Union1 => create_f1(&mut planus::Builder, InnerF32) : planus::UnionOffset<Union1>);

check_type!(+['a] Union1 => create_f2(&mut planus::Builder, &'a InnerEnum) : planus::UnionOffset<Union1>);
check_type!(Union1 => create_f2(&mut planus::Builder, InnerEnum) : planus::UnionOffset<Union1>);

check_type!(+['a] Union1 => create_f3(&mut planus::Builder, &'a InnerTable) : planus::UnionOffset<Union1>);
check_type!(Union1 => create_f3(&mut planus::Builder, InnerTable) : planus::UnionOffset<Union1>);

check_type!(+['a] Union1 => create_f4(&mut planus::Builder, &'a str) : planus::UnionOffset<Union1>);
check_type!(+['a] Union1 => create_f4(&mut planus::Builder, &'a String) : planus::UnionOffset<Union1>);
check_type!(Union1 => create_f4(&mut planus::Builder, String) : planus::UnionOffset<Union1>);

check_type!(+['a] Union1 => create_inner_u8(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union1>);
check_type!(Union1 => create_inner_u8(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union1>);

check_type!(+['a] Union1 => create_inner_f32(&mut planus::Builder, &'a InnerF32) : planus::UnionOffset<Union1>);
check_type!(Union1 => create_inner_f32(&mut planus::Builder, InnerF32) : planus::UnionOffset<Union1>);

check_type!(+['a] Union1 => create_inner_enum(&mut planus::Builder, &'a InnerEnum) : planus::UnionOffset<Union1>);
check_type!(Union1 => create_inner_enum(&mut planus::Builder, InnerEnum) : planus::UnionOffset<Union1>);

check_type!(+['a] Union1 => create_inner_table(&mut planus::Builder, &'a InnerTable) : planus::UnionOffset<Union1>);
check_type!(Union1 => create_inner_table(&mut planus::Builder, InnerTable) : planus::UnionOffset<Union1>);

check_type!(Union1 => F0(InnerU8) : Union1);
check_type!(Union1 => F1(InnerF32) : Union1);
check_type!(Union1 => F2(InnerEnum) : Union1);
check_type!(Union1 => F3(Box<InnerTable>) : Union1);
check_type!(Union1 => F4(String) : Union1);
check_type!(Union1 => InnerU8(InnerU8) : Union1);
check_type!(Union1 => InnerF32(InnerF32) : Union1);
check_type!(Union1 => InnerEnum(InnerEnum) : Union1);
check_type!(Union1 => InnerTable(Box<InnerTable>) : Union1);

check_type!(+['a] Union1Ref<'a> => F0(InnerU8Ref<'a>) : Union1Ref<'a>);
check_type!(+['a] Union1Ref<'a> => F1(InnerF32Ref<'a>) : Union1Ref<'a>);
check_type!(+['a] Union1Ref<'a> => F2(InnerEnumRef<'a>) : Union1Ref<'a>);
check_type!(+['a] Union1Ref<'a> => F3(InnerTableRef<'a>) : Union1Ref<'a>);
check_type!(+['a] Union1Ref<'a> => F4(&'a str) : Union1Ref<'a>);
check_type!(+['a] Union1Ref<'a> => InnerU8(InnerU8Ref<'a>) : Union1Ref<'a>);
check_type!(+['a] Union1Ref<'a> => InnerF32(InnerF32Ref<'a>) : Union1Ref<'a>);
check_type!(+['a] Union1Ref<'a> => InnerEnum(InnerEnumRef<'a>) : Union1Ref<'a>);
check_type!(+['a] Union1Ref<'a> => InnerTable(InnerTableRef<'a>) : Union1Ref<'a>);

assert_traits!(
    Union1: !Copy + Clone + Debug + PartialEq + PartialOrd + !Eq + !Ord + !Hash + !Default,
    Union1Ref<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Union1>} + !{Into<Union1>},
);

// Union2
check_type!(+['a] Union2 => create_f0(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union2>);
check_type!(Union2 => create_f0(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union2>);

check_type!(+['a] Union2 => create_f1(&mut planus::Builder, &'a InnerF32) : planus::UnionOffset<Union2>);
check_type!(Union2 => create_f1(&mut planus::Builder, InnerF32) : planus::UnionOffset<Union2>);

check_type!(+['a] Union2 => create_inner_u8(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union2>);
check_type!(Union2 => create_inner_u8(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union2>);

check_type!(+['a] Union2 => create_inner_f32(&mut planus::Builder, &'a InnerF32) : planus::UnionOffset<Union2>);
check_type!(Union2 => create_inner_f32(&mut planus::Builder, InnerF32) : planus::UnionOffset<Union2>);

check_type!(Union2 => F0(InnerU8) : Union2);
check_type!(Union2 => F1(InnerF32) : Union2);
check_type!(Union2 => InnerU8(InnerU8) : Union2);
check_type!(Union2 => InnerF32(InnerF32) : Union2);

check_type!(+['a] Union2Ref<'a> => F0(InnerU8Ref<'a>) : Union2Ref<'a>);
check_type!(+['a] Union2Ref<'a> => F1(InnerF32Ref<'a>) : Union2Ref<'a>);
check_type!(+['a] Union2Ref<'a> => InnerU8(InnerU8Ref<'a>) : Union2Ref<'a>);
check_type!(+['a] Union2Ref<'a> => InnerF32(InnerF32Ref<'a>) : Union2Ref<'a>);

assert_traits!(
    Union2: !Copy + Clone + Debug + PartialEq + PartialOrd + !Eq + !Ord + !Hash + !Default,
    Union2Ref<'_>: Copy + Clone + Debug + PartialEq + PartialOrd + !Eq + !Ord + !Hash + !Default + {Into<Union2>},
);

// Union3
check_type!(+['a] Union3 => create_f0(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union3>);
check_type!(Union3 => create_f0(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union3>);

check_type!(+['a] Union3 => create_f2(&mut planus::Builder, &'a InnerEnum) : planus::UnionOffset<Union3>);
check_type!(Union3 => create_f2(&mut planus::Builder, InnerEnum) : planus::UnionOffset<Union3>);

check_type!(+['a] Union3 => create_inner_u8(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union3>);
check_type!(Union3 => create_inner_u8(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union3>);

check_type!(+['a] Union3 => create_inner_enum(&mut planus::Builder, &'a InnerEnum) : planus::UnionOffset<Union3>);
check_type!(Union3 => create_inner_enum(&mut planus::Builder, InnerEnum) : planus::UnionOffset<Union3>);

check_type!(Union3 => F0(InnerU8) : Union3);
check_type!(Union3 => F2(InnerEnum) : Union3);
check_type!(Union3 => InnerU8(InnerU8) : Union3);
check_type!(Union3 => InnerEnum(InnerEnum) : Union3);

check_type!(+['a] Union3Ref<'a> => F0(InnerU8Ref<'a>) : Union3Ref<'a>);
check_type!(+['a] Union3Ref<'a> => F2(InnerEnumRef<'a>) : Union3Ref<'a>);
check_type!(+['a] Union3Ref<'a> => InnerU8(InnerU8Ref<'a>) : Union3Ref<'a>);
check_type!(+['a] Union3Ref<'a> => InnerEnum(InnerEnumRef<'a>) : Union3Ref<'a>);

assert_traits!(
    Union3: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    Union3Ref<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Union3>} + !{Into<Union3>},
);

// Union4
check_type!(+['a] Union4 => create_f0(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union4>);
check_type!(Union4 => create_f0(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union4>);

check_type!(+['a] Union4 => create_f3(&mut planus::Builder, &'a InnerTable) : planus::UnionOffset<Union4>);
check_type!(Union4 => create_f3(&mut planus::Builder, InnerTable) : planus::UnionOffset<Union4>);

check_type!(+['a] Union4 => create_inner_u8(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union4>);
check_type!(Union4 => create_inner_u8(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union4>);

check_type!(+['a] Union4 => create_inner_table(&mut planus::Builder, &'a InnerTable) : planus::UnionOffset<Union4>);
check_type!(Union4 => create_inner_table(&mut planus::Builder, InnerTable) : planus::UnionOffset<Union4>);

check_type!(Union4 => F0(InnerU8) : Union4);
check_type!(Union4 => F3(Box<InnerTable>) : Union4);
check_type!(Union4 => InnerU8(InnerU8) : Union4);
check_type!(Union4 => InnerTable(Box<InnerTable>) : Union4);

check_type!(+['a] Union4Ref<'a> => F0(InnerU8Ref<'a>) : Union4Ref<'a>);
check_type!(+['a] Union4Ref<'a> => F3(InnerTableRef<'a>) : Union4Ref<'a>);
check_type!(+['a] Union4Ref<'a> => InnerU8(InnerU8Ref<'a>) : Union4Ref<'a>);
check_type!(+['a] Union4Ref<'a> => InnerTable(InnerTableRef<'a>) : Union4Ref<'a>);

assert_traits!(
    Union4: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    Union4Ref<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Union4>} + !{Into<Union4>},
);

// Union5
check_type!(+['a] Union5 => create_f0(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union5>);
check_type!(Union5 => create_f0(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union5>);

check_type!(+['a] Union5 => create_f4(&mut planus::Builder, &'a str) : planus::UnionOffset<Union5>);
check_type!(+['a] Union5 => create_f4(&mut planus::Builder, &'a String) : planus::UnionOffset<Union5>);
check_type!(Union5 => create_f4(&mut planus::Builder, String) : planus::UnionOffset<Union5>);

check_type!(+['a] Union5 => create_inner_u8(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union5>);
check_type!(Union5 => create_inner_u8(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union5>);

check_type!(Union5 => F0(InnerU8) : Union5);
check_type!(Union5 => F4(String) : Union5);
check_type!(Union5 => InnerU8(InnerU8) : Union5);

check_type!(+['a] Union5Ref<'a> => F0(InnerU8Ref<'a>) : Union5Ref<'a>);
check_type!(+['a] Union5Ref<'a> => F4(&'a str) : Union5Ref<'a>);
check_type!(+['a] Union5Ref<'a> => InnerU8(InnerU8Ref<'a>) : Union5Ref<'a>);

assert_traits!(
    Union5: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    Union5Ref<'_>: Copy + Clone + Debug + !PartialEq + !PartialOrd + !Eq + !Ord + !Hash + !Default + {TryInto<Union5>} + !{Into<Union5>},
);

// Union6
check_type!(+['a] Union6 => create_f0(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union6>);
check_type!(Union6 => create_f0(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union6>);

check_type!(+['a] Union6 => create_inner_u8(&mut planus::Builder, &'a InnerU8) : planus::UnionOffset<Union6>);
check_type!(Union6 => create_inner_u8(&mut planus::Builder, InnerU8) : planus::UnionOffset<Union6>);

check_type!(Union6 => F0(InnerU8) : Union6);
check_type!(Union6 => InnerU8(InnerU8) : Union6);

check_type!(+['a] Union6Ref<'a> => F0(InnerU8Ref<'a>) : Union6Ref<'a>);
check_type!(+['a] Union6Ref<'a> => InnerU8(InnerU8Ref<'a>) : Union6Ref<'a>);

assert_traits!(
    Union6: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    Union6Ref<'_>: Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default + {Into<Union6>},
);

// Union7
assert_traits!(
    Union7: !Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    Union7Ref: Copy + Clone + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default + {Into<Union7>},
);
