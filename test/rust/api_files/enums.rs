check_enum_variants!(EnumUint1: u8 { X = 0 });
check_enum_variants!(EnumUint2: u16 { X = 0 });
check_enum_variants!(EnumUint3: u32 { X = 0 });
check_enum_variants!(EnumUint4: u64 { X = 0 });
check_enum_variants!(EnumUint5: u8 { X = 0 });
check_enum_variants!(EnumUint6: u16 { X = 0 });
check_enum_variants!(EnumUint7: u32 { X = 0 });
check_enum_variants!(EnumUint8: u64 { X = 0 });

check_enum_variants!(EnumUint9: u8 { });
check_enum_variants!(EnumUint10: u8 { X = 0 });
check_enum_variants!(EnumUint11: u8 { Y = 1 });
check_enum_variants!(EnumUint12: u8 { Z = 255 });
check_enum_variants!(EnumUint13: u8 { X = 0, Y = 1 });
check_enum_variants!(EnumUint14: u8 { X = 0, Z = 255 });
check_enum_variants!(EnumUint15: u8 { Y = 1, Z = 255 });
check_enum_variants!(EnumUint16: u8 { X = 0, Y = 1, Z = 255 });

check_enum_variants!(EnumInt1: i8 { X = 0 });
check_enum_variants!(EnumInt2: i16 { X = 0 });
check_enum_variants!(EnumInt3: i32 { X = 0 });
check_enum_variants!(EnumInt4: i64 { X = 0 });
check_enum_variants!(EnumInt5: i8 { X = 0 });
check_enum_variants!(EnumInt6: i16 { X = 0 });
check_enum_variants!(EnumInt7: i32 { X = 0 });
check_enum_variants!(EnumInt8: i64 { X = 0 });

check_enum_variants!(EnumInt9: i8 { });
check_enum_variants!(EnumInt10: i8 { X = 0 });
check_enum_variants!(EnumInt11: i8 { Y = 1 });
check_enum_variants!(EnumInt12: i8 { Z = -1 });
check_enum_variants!(EnumInt13: i8 { X = 0, Y = 1 });
check_enum_variants!(EnumInt14: i8 { X = 0, Z = -1 });
check_enum_variants!(EnumInt15: i8 { Y = 1, Z = -1 });
check_enum_variants!(EnumInt16: i8 { X = 0, Y = 1, Z = -1 });

assert_traits!(
    EnumUint1: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint2: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint3: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint4: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint5: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint6: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint7: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint8: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint9: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint10: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint11: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint12: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint13: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint14: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint15: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumUint16: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt1: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt2: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt3: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt4: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt5: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt6: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt7: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt8: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt9: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt10: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt11: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt12: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt13: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt14: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt15: Copy + Debug + Eq + Ord + Hash + !Default,
    EnumInt16: Copy + Debug + Eq + Ord + Hash + !Default,
);
