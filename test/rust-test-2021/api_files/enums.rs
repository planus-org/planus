check_enum_variants!(EnumUint1: u8 { X = 0 });
check_enum_variants!(EnumUint2: u16 { X = 0 });
check_enum_variants!(EnumUint3: u32 { X = 0 });
check_enum_variants!(EnumUint4: u64 { X = 0 });
check_enum_variants!(EnumUint5: u8 { X = 0 });
check_enum_variants!(EnumUint6: u16 { X = 0 });
check_enum_variants!(EnumUint7: u32 { X = 0 });
check_enum_variants!(EnumUint8: u64 { X = 0 });

assert_eq!(EnumUint1::ENUM_VALUES, [EnumUint1::X]);
assert_eq!(EnumUint2::ENUM_VALUES, [EnumUint2::X]);
assert_eq!(EnumUint3::ENUM_VALUES, [EnumUint3::X]);
assert_eq!(EnumUint4::ENUM_VALUES, [EnumUint4::X]);
assert_eq!(EnumUint5::ENUM_VALUES, [EnumUint5::X]);
assert_eq!(EnumUint6::ENUM_VALUES, [EnumUint6::X]);
assert_eq!(EnumUint7::ENUM_VALUES, [EnumUint7::X]);
assert_eq!(EnumUint8::ENUM_VALUES, [EnumUint8::X]);

check_enum_variants!(EnumUint9: u8 { });
check_enum_variants!(EnumUint10: u8 { X = 0 });
check_enum_variants!(EnumUint11: u8 { Y = 1 });
check_enum_variants!(EnumUint12: u8 { Z = 255 });
check_enum_variants!(EnumUint13: u8 { X = 0, Y = 1 });
check_enum_variants!(EnumUint14: u8 { X = 0, Z = 255 });
check_enum_variants!(EnumUint15: u8 { Y = 1, Z = 255 });
check_enum_variants!(EnumUint16: u8 { X = 0, Y = 1, Z = 255 });

assert_eq!(EnumUint9::ENUM_VALUES, []);
assert_eq!(EnumUint10::ENUM_VALUES, [EnumUint10::X]);
assert_eq!(EnumUint11::ENUM_VALUES, [EnumUint11::Y]);
assert_eq!(EnumUint12::ENUM_VALUES, [EnumUint12::Z]);
assert_eq!(EnumUint13::ENUM_VALUES, [EnumUint13::X, EnumUint13::Y]);
assert_eq!(EnumUint14::ENUM_VALUES, [EnumUint14::X, EnumUint14::Z]);
assert_eq!(EnumUint15::ENUM_VALUES, [EnumUint15::Y, EnumUint15::Z]);
assert_eq!(
    EnumUint16::ENUM_VALUES,
    [EnumUint16::X, EnumUint16::Y, EnumUint16::Z]
);

check_enum_variants!(EnumInt1: i8 { X = 0 });
check_enum_variants!(EnumInt2: i16 { X = 0 });
check_enum_variants!(EnumInt3: i32 { X = 0 });
check_enum_variants!(EnumInt4: i64 { X = 0 });
check_enum_variants!(EnumInt5: i8 { X = 0 });
check_enum_variants!(EnumInt6: i16 { X = 0 });
check_enum_variants!(EnumInt7: i32 { X = 0 });
check_enum_variants!(EnumInt8: i64 { X = 0 });

assert_eq!(EnumInt1::ENUM_VALUES, [EnumInt1::X]);
assert_eq!(EnumInt2::ENUM_VALUES, [EnumInt2::X]);
assert_eq!(EnumInt3::ENUM_VALUES, [EnumInt3::X]);
assert_eq!(EnumInt4::ENUM_VALUES, [EnumInt4::X]);
assert_eq!(EnumInt5::ENUM_VALUES, [EnumInt5::X]);
assert_eq!(EnumInt6::ENUM_VALUES, [EnumInt6::X]);
assert_eq!(EnumInt7::ENUM_VALUES, [EnumInt7::X]);
assert_eq!(EnumInt8::ENUM_VALUES, [EnumInt8::X]);

check_enum_variants!(EnumInt9: i8 { });
check_enum_variants!(EnumInt10: i8 { X = 0 });
check_enum_variants!(EnumInt11: i8 { Y = 1 });
check_enum_variants!(EnumInt12: i8 { Z = -1 });
check_enum_variants!(EnumInt13: i8 { X = 0, Y = 1 });
check_enum_variants!(EnumInt14: i8 { X = 0, Z = -1 });
check_enum_variants!(EnumInt15: i8 { Y = 1, Z = -1 });
check_enum_variants!(EnumInt16: i8 { X = 0, Y = 1, Z = -1 });

assert_eq!(EnumInt9::ENUM_VALUES, []);
assert_eq!(EnumInt10::ENUM_VALUES, [EnumInt10::X]);
assert_eq!(EnumInt11::ENUM_VALUES, [EnumInt11::Y]);
assert_eq!(EnumInt12::ENUM_VALUES, [EnumInt12::Z]);
assert_eq!(EnumInt13::ENUM_VALUES, [EnumInt13::X, EnumInt13::Y]);
assert_eq!(EnumInt14::ENUM_VALUES, [EnumInt14::X, EnumInt14::Z]);
assert_eq!(EnumInt15::ENUM_VALUES, [EnumInt15::Y, EnumInt15::Z]);
assert_eq!(
    EnumInt16::ENUM_VALUES,
    [EnumInt16::X, EnumInt16::Y, EnumInt16::Z]
);

assert_traits!(
    EnumUint1: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint2: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint3: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint4: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint5: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint6: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint7: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint8: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint9: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint10: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint11: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint12: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint13: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint14: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint15: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumUint16: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt1: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt2: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt3: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt4: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt5: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt6: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt7: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt8: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt9: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt10: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt11: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt12: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt13: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt14: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt15: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
    EnumInt16: Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + !Default,
);
