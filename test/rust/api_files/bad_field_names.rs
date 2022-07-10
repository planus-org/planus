check_type!(Table => value : u8);
check_type!(Table => f : u8);
check_type!(Table => n : u8);
check_type!(Table => n_ : u16);
check_type!(Table => __ : u8);
check_type!(Table => ___ : u16);
check_type!(Table => builder : u8);
check_type!(Table => builder_ : u16);
check_type!(Table => value : u8);
check_type!(Table => value_ : u16);
check_type!(Table => values : u8);
check_type!(Table => bytes : u8);
check_type!(Table => offset : u8);
check_type!(Table => cursor : u8);
check_type!(Table => buffer : u8);
check_type!(Table => buffer_position : u8);
check_type!(Table => struct_offset : u8);
check_type!(Table => field_offset : u8);
check_type!(Table => field_offset_ : u16);
check_type!(Table => tag : u8);
check_type!(Table => offset_from_start : u8);
check_type!(Table => slice : u8);
check_type!(Table => default : u8);

check_type!(+['a] TableRef<'a> => &self.value() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.value() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.f() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.n() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.n_() : planus::Result<u16>);
check_type!(+['a] TableRef<'a> => &self.__() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.___() : planus::Result<u16>);
check_type!(+['a] TableRef<'a> => &self.builder() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.builder_() : planus::Result<u16>);
check_type!(+['a] TableRef<'a> => &self.value() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.value_() : planus::Result<u16>);
check_type!(+['a] TableRef<'a> => &self.values() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.bytes() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.offset() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.cursor() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.buffer() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.buffer_position() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.struct_offset() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.field_offset() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.field_offset_() : planus::Result<u16>);
check_type!(+['a] TableRef<'a> => &self.tag() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.offset_from_start() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.slice() : planus::Result<u8>);
check_type!(+['a] TableRef<'a> => &self.default() : planus::Result<u8>);

assert_traits!(
    Table: !Copy + Clone + Debug + Eq + Ord + Hash + Default,
    TableRef<'_>: Copy + Clone + Debug + !Eq + !Ord + !Hash + !Default + {TryInto<Table>} + !{Into<Table>},
);

check_type!(Struct => value : u8);
check_type!(Struct => f : u8);
check_type!(Struct => n : u8);
check_type!(Struct => n_ : u16);
check_type!(Struct => __ : u8);
check_type!(Struct => ___ : u16);
check_type!(Struct => builder : u8);
check_type!(Struct => builder_ : u16);
check_type!(Struct => value : u8);
check_type!(Struct => value_ : u16);
check_type!(Struct => values : u8);
check_type!(Struct => bytes : u8);
check_type!(Struct => offset : u8);
check_type!(Struct => cursor : u8);
check_type!(Struct => buffer : u8);
check_type!(Struct => buffer_position : u8);
check_type!(Struct => struct_offset : u8);
check_type!(Struct => field_offset : u8);
check_type!(Struct => field_offset_ : u16);
check_type!(Struct => tag : u8);
check_type!(Struct => offset_from_start : u8);
check_type!(Struct => slice : u8);
check_type!(Struct => default : u8);

check_type!(+['a] StructRef<'a> => &self.value() : u8);
check_type!(+['a] StructRef<'a> => &self.value() : u8);
check_type!(+['a] StructRef<'a> => &self.f() : u8);
check_type!(+['a] StructRef<'a> => &self.n() : u8);
check_type!(+['a] StructRef<'a> => &self.n_() : u16);
check_type!(+['a] StructRef<'a> => &self.__() : u8);
check_type!(+['a] StructRef<'a> => &self.___() : u16);
check_type!(+['a] StructRef<'a> => &self.builder() : u8);
check_type!(+['a] StructRef<'a> => &self.builder_() : u16);
check_type!(+['a] StructRef<'a> => &self.value() : u8);
check_type!(+['a] StructRef<'a> => &self.value_() : u16);
check_type!(+['a] StructRef<'a> => &self.values() : u8);
check_type!(+['a] StructRef<'a> => &self.bytes() : u8);
check_type!(+['a] StructRef<'a> => &self.offset() : u8);
check_type!(+['a] StructRef<'a> => &self.cursor() : u8);
check_type!(+['a] StructRef<'a> => &self.buffer() : u8);
check_type!(+['a] StructRef<'a> => &self.buffer_position() : u8);
check_type!(+['a] StructRef<'a> => &self.struct_offset() : u8);
check_type!(+['a] StructRef<'a> => &self.field_offset() : u8);
check_type!(+['a] StructRef<'a> => &self.field_offset_() : u16);
check_type!(+['a] StructRef<'a> => &self.tag() : u8);
check_type!(+['a] StructRef<'a> => &self.offset_from_start() : u8);
check_type!(+['a] StructRef<'a> => &self.slice() : u8);
check_type!(+['a] StructRef<'a> => &self.default() : u8);

assert_traits!(
    Struct: Copy + Clone + Debug + Eq + Ord + Hash + Default,
    StructRef<'_>: Copy + Clone + Debug + Eq + Ord + Hash + !Default + {Into<Struct>},
);

check_enum_variants!(Enum: u8 {
    F = 0,
    N = 1,
    N_ = 2,
    __ = 3,
    ___ = 4,
    Builder = 5,
    Builder_ = 6,
    Value = 7,
    Value_ = 8,
    Values = 9,
    Bytes = 10,
    Offset = 11,
    Cursor = 12,
    Buffer = 13,
    BufferPosition = 14,
    StructOffset = 15,
    FieldOffset = 16,
    FieldOffset_ = 17,
    Tag = 18,
    OffsetFromStart = 19,
    Slice = 20,
    Default = 21,
});

assert_traits!(Enum: Copy + Clone + Debug + Eq + Ord + Hash + !Default);

check_type!(Union => Value(Box<Table>) : Union);
check_type!(Union => F(Box<Table>) : Union);
check_type!(Union => N(Box<Table>) : Union);
check_type!(Union => N_(Box<Table2>) : Union);
check_type!(Union => __(Box<Table>) : Union);
check_type!(Union => ___(Box<Table2>) : Union);
check_type!(Union => Builder(Box<Table>) : Union);
check_type!(Union => Builder_(Box<Table2>) : Union);
check_type!(Union => Value(Box<Table>) : Union);
check_type!(Union => Value_(Box<Table2>) : Union);
check_type!(Union => Values(Box<Table>) : Union);
check_type!(Union => Bytes(Box<Table>) : Union);
check_type!(Union => Offset(Box<Table>) : Union);
check_type!(Union => Cursor(Box<Table>) : Union);
check_type!(Union => Buffer(Box<Table>) : Union);
check_type!(Union => BufferPosition(Box<Table>) : Union);
check_type!(Union => StructOffset(Box<Table>) : Union);
check_type!(Union => FieldOffset(Box<Table>) : Union);
check_type!(Union => FieldOffset_(Box<Table2>) : Union);
check_type!(Union => Tag(Box<Table>) : Union);
check_type!(Union => OffsetFromStart(Box<Table>) : Union);
check_type!(Union => Slice(Box<Table>) : Union);
check_type!(Union => Default(Box<Table>) : Union);

check_type!(+['a] UnionRef<'a> => Value(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => F(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => N(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => N_(Table2Ref<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => __(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => ___(Table2Ref<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Builder(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Builder_(Table2Ref<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Value(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Value_(Table2Ref<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Values(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Bytes(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Offset(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Cursor(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Buffer(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => BufferPosition(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => StructOffset(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => FieldOffset(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => FieldOffset_(Table2Ref<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Tag(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => OffsetFromStart(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Slice(TableRef<'a>) : UnionRef<'a>);
check_type!(+['a] UnionRef<'a> => Default(TableRef<'a>) : UnionRef<'a>);

assert_traits!(
    Union: !Copy + Clone + Debug + Eq + Ord + Hash + !Default,
    UnionRef<'_>: Copy + Clone + Debug + !Eq + !Ord + !Hash + !Default + {TryInto<Union>} + !{Into<Union>},
);
