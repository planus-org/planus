check_type!(ExampleU8 => value : u8);
check_type!(ExampleU8 => value_null : Option<u8>);
check_type!(ExampleU8 => value_default_zero : u8);
check_type!(ExampleU8 => value_default_one : u8);
check_type!(ExampleU8 => create(&mut planus::Builder, u8, u8, u8, u8) : planus::Offset<ExampleU8>);
check_type!(ExampleU8 => create(&mut planus::Builder, u8, (), u8, u8) : planus::Offset<ExampleU8>);
check_type!(ExampleU8 => create(&mut planus::Builder, u8, Option<u8>, u8, u8) : planus::Offset<ExampleU8>);

check_type!(+['a] ExampleU8Ref<'a> => &self.value() : planus::Result<u8>);
check_type!(+['a] ExampleU8Ref<'a> => &self.value_null() : planus::Result<Option<u8>>);
check_type!(+['a] ExampleU8Ref<'a> => &self.value_default_zero() : planus::Result<u8>);
check_type!(+['a] ExampleU8Ref<'a> => &self.value_default_one() : planus::Result<u8>);
check_type!(+['a] ExampleU8Ref<'a> => impl planus::ReadAsRoot<'a>);

check_type!(ExampleU16 => value : u16);
check_type!(ExampleU16 => value_null : Option<u16>);
check_type!(ExampleU16 => value_default_zero : u16);
check_type!(ExampleU16 => value_default_one : u16);
check_type!(ExampleU16 => create(&mut planus::Builder, u16, u16, u16, u16) : planus::Offset<ExampleU16>);
check_type!(ExampleU16 => create(&mut planus::Builder, u16, (), u16, u16) : planus::Offset<ExampleU16>);
check_type!(ExampleU16 => create(&mut planus::Builder, u16, Option<u16>, u16, u16) : planus::Offset<ExampleU16>);

check_type!(+['a] ExampleU16Ref<'a> => &self.value() : planus::Result<u16>);
check_type!(+['a] ExampleU16Ref<'a> => &self.value_null() : planus::Result<Option<u16>>);
check_type!(+['a] ExampleU16Ref<'a> => &self.value_default_zero() : planus::Result<u16>);
check_type!(+['a] ExampleU16Ref<'a> => &self.value_default_one() : planus::Result<u16>);
check_type!(+['a] ExampleU16Ref<'a> => impl planus::ReadAsRoot<'a>);

check_type!(ExampleU32 => value : u32);
check_type!(ExampleU32 => value_null : Option<u32>);
check_type!(ExampleU32 => value_default_zero : u32);
check_type!(ExampleU32 => value_default_one : u32);
check_type!(ExampleU32 => create(&mut planus::Builder, u32, u32, u32, u32) : planus::Offset<ExampleU32>);
check_type!(ExampleU32 => create(&mut planus::Builder, u32, (), u32, u32) : planus::Offset<ExampleU32>);
check_type!(ExampleU32 => create(&mut planus::Builder, u32, Option<u32>, u32, u32) : planus::Offset<ExampleU32>);

check_type!(+['a] ExampleU32Ref<'a> => &self.value() : planus::Result<u32>);
check_type!(+['a] ExampleU32Ref<'a> => &self.value_null() : planus::Result<Option<u32>>);
check_type!(+['a] ExampleU32Ref<'a> => &self.value_default_zero() : planus::Result<u32>);
check_type!(+['a] ExampleU32Ref<'a> => &self.value_default_one() : planus::Result<u32>);
check_type!(+['a] ExampleU32Ref<'a> => impl planus::ReadAsRoot<'a>);

check_type!(ExampleU64 => value : u64);
check_type!(ExampleU64 => value_null : Option<u64>);
check_type!(ExampleU64 => value_default_zero : u64);
check_type!(ExampleU64 => value_default_one : u64);
check_type!(ExampleU64 => create(&mut planus::Builder, u64, u64, u64, u64) : planus::Offset<ExampleU64>);
check_type!(ExampleU64 => create(&mut planus::Builder, u64, (), u64, u64) : planus::Offset<ExampleU64>);
check_type!(ExampleU64 => create(&mut planus::Builder, u64, Option<u64>, u64, u64) : planus::Offset<ExampleU64>);

check_type!(+['a] ExampleU64Ref<'a> => &self.value() : planus::Result<u64>);
check_type!(+['a] ExampleU64Ref<'a> => &self.value_null() : planus::Result<Option<u64>>);
check_type!(+['a] ExampleU64Ref<'a> => &self.value_default_zero() : planus::Result<u64>);
check_type!(+['a] ExampleU64Ref<'a> => &self.value_default_one() : planus::Result<u64>);
check_type!(+['a] ExampleU64Ref<'a> => impl planus::ReadAsRoot<'a>);

check_type!(ExampleI8 => value : i8);
check_type!(ExampleI8 => value_null : Option<i8>);
check_type!(ExampleI8 => value_default_zero : i8);
check_type!(ExampleI8 => value_default_one : i8);
check_type!(ExampleI8 => create(&mut planus::Builder, i8, i8, i8, i8) : planus::Offset<ExampleI8>);
check_type!(ExampleI8 => create(&mut planus::Builder, i8, (), i8, i8) : planus::Offset<ExampleI8>);
check_type!(ExampleI8 => create(&mut planus::Builder, i8, Option<i8>, i8, i8) : planus::Offset<ExampleI8>);

check_type!(+['a] ExampleI8Ref<'a> => &self.value() : planus::Result<i8>);
check_type!(+['a] ExampleI8Ref<'a> => &self.value_null() : planus::Result<Option<i8>>);
check_type!(+['a] ExampleI8Ref<'a> => &self.value_default_zero() : planus::Result<i8>);
check_type!(+['a] ExampleI8Ref<'a> => &self.value_default_one() : planus::Result<i8>);
check_type!(+['a] ExampleI8Ref<'a> => impl planus::ReadAsRoot<'a>);

check_type!(ExampleI16 => value : i16);
check_type!(ExampleI16 => value_null : Option<i16>);
check_type!(ExampleI16 => value_default_zero : i16);
check_type!(ExampleI16 => value_default_one : i16);
check_type!(ExampleI16 => create(&mut planus::Builder, i16, i16, i16, i16) : planus::Offset<ExampleI16>);
check_type!(ExampleI16 => create(&mut planus::Builder, i16, (), i16, i16) : planus::Offset<ExampleI16>);
check_type!(ExampleI16 => create(&mut planus::Builder, i16, Option<i16>, i16, i16) : planus::Offset<ExampleI16>);

check_type!(+['a] ExampleI16Ref<'a> => &self.value() : planus::Result<i16>);
check_type!(+['a] ExampleI16Ref<'a> => &self.value_null() : planus::Result<Option<i16>>);
check_type!(+['a] ExampleI16Ref<'a> => &self.value_default_zero() : planus::Result<i16>);
check_type!(+['a] ExampleI16Ref<'a> => &self.value_default_one() : planus::Result<i16>);
check_type!(+['a] ExampleI16Ref<'a> => impl planus::ReadAsRoot<'a>);

check_type!(ExampleI32 => value : i32);
check_type!(ExampleI32 => value_null : Option<i32>);
check_type!(ExampleI32 => value_default_zero : i32);
check_type!(ExampleI32 => value_default_one : i32);
check_type!(ExampleI32 => create(&mut planus::Builder, i32, i32, i32, i32) : planus::Offset<ExampleI32>);
check_type!(ExampleI32 => create(&mut planus::Builder, i32, (), i32, i32) : planus::Offset<ExampleI32>);
check_type!(ExampleI32 => create(&mut planus::Builder, i32, Option<i32>, i32, i32) : planus::Offset<ExampleI32>);

check_type!(+['a] ExampleI32Ref<'a> => &self.value() : planus::Result<i32>);
check_type!(+['a] ExampleI32Ref<'a> => &self.value_null() : planus::Result<Option<i32>>);
check_type!(+['a] ExampleI32Ref<'a> => &self.value_default_zero() : planus::Result<i32>);
check_type!(+['a] ExampleI32Ref<'a> => &self.value_default_one() : planus::Result<i32>);
check_type!(+['a] ExampleI32Ref<'a> => impl planus::ReadAsRoot<'a>);

check_type!(ExampleI64 => value : i64);
check_type!(ExampleI64 => value_null : Option<i64>);
check_type!(ExampleI64 => value_default_zero : i64);
check_type!(ExampleI64 => value_default_one : i64);
check_type!(ExampleI64 => create(&mut planus::Builder, i64, i64, i64, i64) : planus::Offset<ExampleI64>);
check_type!(ExampleI64 => create(&mut planus::Builder, i64, (), i64, i64) : planus::Offset<ExampleI64>);
check_type!(ExampleI64 => create(&mut planus::Builder, i64, Option<i64>, i64, i64) : planus::Offset<ExampleI64>);

check_type!(+['a] ExampleI64Ref<'a> => &self.value() : planus::Result<i64>);
check_type!(+['a] ExampleI64Ref<'a> => &self.value_null() : planus::Result<Option<i64>>);
check_type!(+['a] ExampleI64Ref<'a> => &self.value_default_zero() : planus::Result<i64>);
check_type!(+['a] ExampleI64Ref<'a> => &self.value_default_one() : planus::Result<i64>);
check_type!(+['a] ExampleI64Ref<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    ExampleU8: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    ExampleU16: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    ExampleU32: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    ExampleU64: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    ExampleI8: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    ExampleI16: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    ExampleI32: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    ExampleI64: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
);
