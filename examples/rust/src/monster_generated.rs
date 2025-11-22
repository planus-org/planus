pub use root::*;

const _: () = ::planus::check_version_compatibility("planus-1.2.0");

/// The root namespace
///
/// Generated from these locations:
/// * File `examples/rust/monster.fbs`
#[no_implicit_prelude]
#[allow(clippy::needless_lifetimes)]
mod root {
    /// The namespace `MyGame`
    ///
    /// Generated from these locations:
    /// * File `examples/rust/monster.fbs`
    pub mod my_game {
        ///  Example IDL file for our monster's schema.
        ///
        /// Generated from these locations:
        /// * File `examples/rust/monster.fbs`
        pub mod sample {
            ///  The possible monster colors
            ///
            /// Generated from these locations:
            /// * Enum `Color` in the file `examples/rust/monster.fbs:6`
            #[derive(
                Copy,
                Clone,
                Debug,
                PartialEq,
                Eq,
                PartialOrd,
                Ord,
                Hash,
                ::serde::Serialize,
                ::serde::Deserialize,
            )]
            #[repr(i8)]
            pub enum Color {
                ///  Should be rendered the same color as blood
                Red = 0,

                ///  Any green will do
                Green = 1,

                ///  Must be `#89CFF0`
                Blue = 2,
            }

            impl Color {
                /// Array containing all valid variants of Color
                pub const ENUM_VALUES: [Self; 3] = [Self::Red, Self::Green, Self::Blue];
            }

            impl ::core::convert::TryFrom<i8> for Color {
                type Error = ::planus::errors::UnknownEnumTagKind;
                #[inline]
                fn try_from(
                    value: i8,
                ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTagKind>
                {
                    #[allow(clippy::match_single_binding)]
                    match value {
                        0 => ::core::result::Result::Ok(Color::Red),
                        1 => ::core::result::Result::Ok(Color::Green),
                        2 => ::core::result::Result::Ok(Color::Blue),

                        _ => ::core::result::Result::Err(::planus::errors::UnknownEnumTagKind {
                            tag: value as i128,
                        }),
                    }
                }
            }

            impl ::core::convert::From<Color> for i8 {
                #[inline]
                fn from(value: Color) -> Self {
                    value as i8
                }
            }

            /// # Safety
            /// The Planus compiler correctly calculates `ALIGNMENT` and `SIZE`.
            unsafe impl ::planus::Primitive for Color {
                const ALIGNMENT: usize = 1;
                const SIZE: usize = 1;
            }

            impl ::planus::WriteAsPrimitive<Color> for Color {
                #[inline]
                fn write<const N: usize>(
                    &self,
                    cursor: ::planus::Cursor<'_, N>,
                    buffer_position: u32,
                ) {
                    (*self as i8).write(cursor, buffer_position);
                }
            }

            impl ::planus::WriteAs<Color> for Color {
                type Prepared = Self;

                #[inline]
                fn prepare(&self, _builder: &mut ::planus::Builder) -> Color {
                    *self
                }
            }

            impl ::planus::WriteAsDefault<Color, Color> for Color {
                type Prepared = Self;

                #[inline]
                fn prepare(
                    &self,
                    _builder: &mut ::planus::Builder,
                    default: &Color,
                ) -> ::core::option::Option<Color> {
                    if self == default {
                        ::core::option::Option::None
                    } else {
                        ::core::option::Option::Some(*self)
                    }
                }
            }

            impl ::planus::WriteAsOptional<Color> for Color {
                type Prepared = Self;

                #[inline]
                fn prepare(
                    &self,
                    _builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<Color> {
                    ::core::option::Option::Some(*self)
                }
            }

            impl<'buf> ::planus::TableRead<'buf> for Color {
                #[inline]
                fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'buf>,
                    offset: usize,
                ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                    let n: i8 = ::planus::TableRead::from_buffer(buffer, offset)?;
                    ::core::result::Result::Ok(::core::convert::TryInto::try_into(n)?)
                }
            }

            impl<'buf> ::planus::VectorReadInner<'buf> for Color {
                type Error = ::planus::errors::UnknownEnumTag;
                const STRIDE: usize = 1;
                #[inline]
                unsafe fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'buf>,
                    offset: usize,
                ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTag>
                {
                    let value = unsafe { *buffer.buffer.get_unchecked(offset) as i8 };
                    let value: ::core::result::Result<Self, _> =
                        ::core::convert::TryInto::try_into(value);
                    value.map_err(|error_kind| {
                        error_kind.with_error_location(
                            "Color",
                            "VectorRead::from_buffer",
                            buffer.offset_from_start,
                        )
                    })
                }
            }

            /// # Safety
            /// The planus compiler generates implementations that initialize
            /// the bytes in `write_values`.
            unsafe impl ::planus::VectorWrite<Color> for Color {
                const STRIDE: usize = 1;

                type Value = Self;

                #[inline]
                fn prepare(&self, _builder: &mut ::planus::Builder) -> Self {
                    *self
                }

                #[inline]
                unsafe fn write_values(
                    values: &[Self],
                    bytes: *mut ::core::mem::MaybeUninit<u8>,
                    buffer_position: u32,
                ) {
                    let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 1];
                    for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                        ::planus::WriteAsPrimitive::write(
                            v,
                            ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                            buffer_position - i as u32,
                        );
                    }
                }
            }

            ///  Weapons or other equipment
            ///
            /// Generated from these locations:
            /// * Union `Equipment` in the file `examples/rust/monster.fbs:16`
            #[derive(
                Clone,
                Debug,
                PartialEq,
                PartialOrd,
                Eq,
                Ord,
                Hash,
                ::serde::Serialize,
                ::serde::Deserialize,
            )]
            pub enum Equipment {
                ///  Equipment of the weapon-type
                Weapon(::planus::alloc::boxed::Box<self::Weapon>),

                ///  Equipment of the shield-type
                Shield(::planus::alloc::boxed::Box<self::Shield>),
            }

            impl Equipment {
                /// Creates a [EquipmentBuilder] for serializing an instance of this table.
                #[inline]
                pub fn builder() -> EquipmentBuilder<::planus::Uninitialized> {
                    EquipmentBuilder(::planus::Uninitialized)
                }

                #[inline]
                pub fn create_weapon(
                    builder: &mut ::planus::Builder,
                    value: impl ::planus::WriteAsOffset<self::Weapon>,
                ) -> ::planus::UnionOffset<Self> {
                    ::planus::UnionOffset::new(1, value.prepare(builder).downcast())
                }

                #[inline]
                pub fn create_shield(
                    builder: &mut ::planus::Builder,
                    value: impl ::planus::WriteAsOffset<self::Shield>,
                ) -> ::planus::UnionOffset<Self> {
                    ::planus::UnionOffset::new(2, value.prepare(builder).downcast())
                }
            }

            impl ::planus::WriteAsUnion<Equipment> for Equipment {
                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Self> {
                    match self {
                        Self::Weapon(value) => Self::create_weapon(builder, value),
                        Self::Shield(value) => Self::create_shield(builder, value),
                    }
                }
            }

            impl ::planus::WriteAsOptionalUnion<Equipment> for Equipment {
                #[inline]
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::UnionOffset<Self>> {
                    ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
                }
            }

            /// Builder for serializing an instance of the [Equipment] type.
            ///
            /// Can be created using the [Equipment::builder] method.
            #[derive(Debug)]
            #[must_use]
            pub struct EquipmentBuilder<T>(T);

            impl EquipmentBuilder<::planus::Uninitialized> {
                /// Creates an instance of the [`Weapon` variant](Equipment#variant.Weapon).
                #[inline]
                pub fn weapon<T>(self, value: T) -> EquipmentBuilder<::planus::Initialized<1, T>>
                where
                    T: ::planus::WriteAsOffset<self::Weapon>,
                {
                    EquipmentBuilder(::planus::Initialized(value))
                }

                /// Creates an instance of the [`Shield` variant](Equipment#variant.Shield).
                #[inline]
                pub fn shield<T>(self, value: T) -> EquipmentBuilder<::planus::Initialized<2, T>>
                where
                    T: ::planus::WriteAsOffset<self::Shield>,
                {
                    EquipmentBuilder(::planus::Initialized(value))
                }
            }

            impl<const N: u8, T> EquipmentBuilder<::planus::Initialized<N, T>> {
                /// Finish writing the builder to get an [UnionOffset](::planus::UnionOffset) to a serialized [Equipment].
                #[inline]
                pub fn finish(
                    self,
                    builder: &mut ::planus::Builder,
                ) -> ::planus::UnionOffset<Equipment>
                where
                    Self: ::planus::WriteAsUnion<Equipment>,
                {
                    ::planus::WriteAsUnion::prepare(&self, builder)
                }
            }

            impl<T> ::planus::WriteAsUnion<Equipment> for EquipmentBuilder<::planus::Initialized<1, T>>
            where
                T: ::planus::WriteAsOffset<self::Weapon>,
            {
                #[inline]
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::planus::UnionOffset<Equipment> {
                    ::planus::UnionOffset::new(1, (self.0).0.prepare(builder).downcast())
                }
            }

            impl<T> ::planus::WriteAsOptionalUnion<Equipment> for EquipmentBuilder<::planus::Initialized<1, T>>
            where
                T: ::planus::WriteAsOffset<self::Weapon>,
            {
                #[inline]
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::UnionOffset<Equipment>> {
                    ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
                }
            }
            impl<T> ::planus::WriteAsUnion<Equipment> for EquipmentBuilder<::planus::Initialized<2, T>>
            where
                T: ::planus::WriteAsOffset<self::Shield>,
            {
                #[inline]
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::planus::UnionOffset<Equipment> {
                    ::planus::UnionOffset::new(2, (self.0).0.prepare(builder).downcast())
                }
            }

            impl<T> ::planus::WriteAsOptionalUnion<Equipment> for EquipmentBuilder<::planus::Initialized<2, T>>
            where
                T: ::planus::WriteAsOffset<self::Shield>,
            {
                #[inline]
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::UnionOffset<Equipment>> {
                    ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
                }
            }

            /// Reference to a deserialized [Equipment].
            #[derive(Copy, Clone, Debug)]
            pub enum EquipmentRef<'a> {
                Weapon(self::WeaponRef<'a>),
                Shield(self::ShieldRef<'a>),
            }

            impl<'a> ::core::convert::TryFrom<EquipmentRef<'a>> for Equipment {
                type Error = ::planus::Error;

                fn try_from(value: EquipmentRef<'a>) -> ::planus::Result<Self> {
                    ::core::result::Result::Ok(match value {
                        EquipmentRef::Weapon(value) => {
                            Self::Weapon(::planus::alloc::boxed::Box::new(
                                ::core::convert::TryFrom::try_from(value)?,
                            ))
                        }

                        EquipmentRef::Shield(value) => {
                            Self::Shield(::planus::alloc::boxed::Box::new(
                                ::core::convert::TryFrom::try_from(value)?,
                            ))
                        }
                    })
                }
            }

            impl<'a> ::planus::TableReadUnion<'a> for EquipmentRef<'a> {
                fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'a>,
                    tag: u8,
                    field_offset: usize,
                ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                    match tag {
                        1 => ::core::result::Result::Ok(Self::Weapon(
                            ::planus::TableRead::from_buffer(buffer, field_offset)?,
                        )),
                        2 => ::core::result::Result::Ok(Self::Shield(
                            ::planus::TableRead::from_buffer(buffer, field_offset)?,
                        )),
                        _ => ::core::result::Result::Err(
                            ::planus::errors::ErrorKind::UnknownUnionTag { tag },
                        ),
                    }
                }
            }

            impl<'a> ::planus::VectorReadUnion<'a> for EquipmentRef<'a> {
                const VECTOR_NAME: &'static str = "[EquipmentRef]";
            }

            ///  Vector in three dimensions
            ///
            /// Generated from these locations:
            /// * Struct `Vec3` in the file `examples/rust/monster.fbs:24`
            #[derive(
                Copy,
                Clone,
                Debug,
                PartialEq,
                PartialOrd,
                Default,
                ::serde::Serialize,
                ::serde::Deserialize,
            )]
            pub struct Vec3 {
                ///  East-west direction
                pub x: f32,

                ///  North-south direction
                pub y: f32,

                ///  Up-down direction
                pub z: f32,
            }

            /// # Safety
            /// The Planus compiler correctly calculates `ALIGNMENT` and `SIZE`.
            unsafe impl ::planus::Primitive for Vec3 {
                const ALIGNMENT: usize = 4;
                const SIZE: usize = 12;
            }

            #[allow(clippy::identity_op)]
            impl ::planus::WriteAsPrimitive<Vec3> for Vec3 {
                #[inline]
                fn write<const N: usize>(
                    &self,
                    cursor: ::planus::Cursor<'_, N>,
                    buffer_position: u32,
                ) {
                    let (cur, cursor) = cursor.split::<4, 8>();
                    self.x.write(cur, buffer_position - 0);
                    let (cur, cursor) = cursor.split::<4, 4>();
                    self.y.write(cur, buffer_position - 4);
                    let (cur, cursor) = cursor.split::<4, 0>();
                    self.z.write(cur, buffer_position - 8);
                    cursor.finish([]);
                }
            }

            impl ::planus::WriteAsOffset<Vec3> for Vec3 {
                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Vec3> {
                    unsafe {
                        builder.write_with(12, 3, |buffer_position, bytes| {
                            let bytes = bytes.as_mut_ptr();

                            ::planus::WriteAsPrimitive::write(
                                self,
                                ::planus::Cursor::new(
                                    &mut *(bytes as *mut [::core::mem::MaybeUninit<u8>; 12]),
                                ),
                                buffer_position,
                            );
                        });
                    }
                    builder.current_offset()
                }
            }

            impl ::planus::WriteAs<Vec3> for Vec3 {
                type Prepared = Self;
                #[inline]
                fn prepare(&self, _builder: &mut ::planus::Builder) -> Self {
                    *self
                }
            }

            impl ::planus::WriteAsOptional<Vec3> for Vec3 {
                type Prepared = Self;
                #[inline]
                fn prepare(
                    &self,
                    _builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<Self> {
                    ::core::option::Option::Some(*self)
                }
            }

            /// Reference to a deserialized [Vec3].
            #[derive(Copy, Clone)]
            pub struct Vec3Ref<'a>(::planus::ArrayWithStartOffset<'a, 12>);

            impl<'a> Vec3Ref<'a> {
                /// Getter for the [`x` field](Vec3#structfield.x).
                pub fn x(&self) -> f32 {
                    let buffer = self.0.advance_as_array::<4>(0).unwrap();

                    f32::from_le_bytes(*buffer.as_array())
                }

                /// Getter for the [`y` field](Vec3#structfield.y).
                pub fn y(&self) -> f32 {
                    let buffer = self.0.advance_as_array::<4>(4).unwrap();

                    f32::from_le_bytes(*buffer.as_array())
                }

                /// Getter for the [`z` field](Vec3#structfield.z).
                pub fn z(&self) -> f32 {
                    let buffer = self.0.advance_as_array::<4>(8).unwrap();

                    f32::from_le_bytes(*buffer.as_array())
                }
            }

            impl<'a> ::core::fmt::Debug for Vec3Ref<'a> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    let mut f = f.debug_struct("Vec3Ref");
                    f.field("x", &self.x());
                    f.field("y", &self.y());
                    f.field("z", &self.z());
                    f.finish()
                }
            }

            impl<'a> ::core::convert::From<::planus::ArrayWithStartOffset<'a, 12>> for Vec3Ref<'a> {
                fn from(array: ::planus::ArrayWithStartOffset<'a, 12>) -> Self {
                    Self(array)
                }
            }

            impl<'a> ::core::convert::From<Vec3Ref<'a>> for Vec3 {
                #[allow(unreachable_code)]
                fn from(value: Vec3Ref<'a>) -> Self {
                    Self {
                        x: value.x(),
                        y: value.y(),
                        z: value.z(),
                    }
                }
            }

            impl<'a, 'b> ::core::cmp::PartialEq<Vec3Ref<'a>> for Vec3Ref<'b> {
                fn eq(&self, other: &Vec3Ref<'_>) -> bool {
                    self.x() == other.x() && self.y() == other.y() && self.z() == other.z()
                }
            }

            impl<'a, 'b> ::core::cmp::PartialOrd<Vec3Ref<'a>> for Vec3Ref<'b> {
                fn partial_cmp(
                    &self,
                    other: &Vec3Ref<'_>,
                ) -> ::core::option::Option<::core::cmp::Ordering> {
                    match self.x().partial_cmp(&other.x()) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => (),
                        o => return o,
                    }

                    match self.y().partial_cmp(&other.y()) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => (),
                        o => return o,
                    }

                    self.z().partial_cmp(&other.z())
                }
            }

            impl<'a> ::planus::TableRead<'a> for Vec3Ref<'a> {
                #[inline]
                fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'a>,
                    offset: usize,
                ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                    let buffer = buffer.advance_as_array::<12>(offset)?;
                    ::core::result::Result::Ok(Self(buffer))
                }
            }

            impl<'a> ::planus::VectorRead<'a> for Vec3Ref<'a> {
                const STRIDE: usize = 12;

                #[inline]
                unsafe fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'a>,
                    offset: usize,
                ) -> Self {
                    Self(unsafe { buffer.unchecked_advance_as_array(offset) })
                }
            }

            /// # Safety
            /// The planus compiler generates implementations that initialize
            /// the bytes in `write_values`.
            unsafe impl ::planus::VectorWrite<Vec3> for Vec3 {
                const STRIDE: usize = 12;

                type Value = Vec3;

                #[inline]
                fn prepare(&self, _builder: &mut ::planus::Builder) -> Self::Value {
                    *self
                }

                #[inline]
                unsafe fn write_values(
                    values: &[Vec3],
                    bytes: *mut ::core::mem::MaybeUninit<u8>,
                    buffer_position: u32,
                ) {
                    let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 12];
                    for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                        ::planus::WriteAsPrimitive::write(
                            v,
                            ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                            buffer_position - (12 * i) as u32,
                        );
                    }
                }
            }

            ///  An enemy in the game
            ///
            /// Generated from these locations:
            /// * Table `Monster` in the file `examples/rust/monster.fbs:34`
            #[derive(
                Clone, Debug, PartialEq, PartialOrd, ::serde::Serialize, ::serde::Deserialize,
            )]
            pub struct Monster {
                ///  Position in the world
                pub pos: ::core::option::Option<self::Vec3>,
                ///  Amount of mana left
                pub mana: i16,
                ///  Amount of hp left
                pub hp: i16,
                ///  Name of monster
                pub name: ::core::option::Option<::planus::alloc::string::String>,
                ///  Inventory of monster
                pub inventory: ::core::option::Option<::planus::alloc::vec::Vec<u8>>,
                ///  Color of the monster's skin
                pub color: self::Color,
                ///  List of all weapons
                pub weapons: ::core::option::Option<::planus::alloc::vec::Vec<self::Weapon>>,
                ///  Currently equipped item
                pub equipped: ::core::option::Option<self::Equipment>,
                ///  Equipment that will be dropped on death
                pub drops: ::core::option::Option<::planus::alloc::vec::Vec<self::Equipment>>,
                ///  The projected path of the monster
                pub path: ::core::option::Option<::planus::alloc::vec::Vec<self::Vec3>>,
            }

            #[allow(clippy::derivable_impls)]
            impl ::core::default::Default for Monster {
                fn default() -> Self {
                    Self {
                        pos: ::core::default::Default::default(),
                        mana: 150,
                        hp: 100,
                        name: ::core::default::Default::default(),
                        inventory: ::core::default::Default::default(),
                        color: self::Color::Blue,
                        weapons: ::core::default::Default::default(),
                        equipped: ::core::default::Default::default(),
                        drops: ::core::default::Default::default(),
                        path: ::core::default::Default::default(),
                    }
                }
            }

            impl Monster {
                /// Creates a [MonsterBuilder] for serializing an instance of this table.
                #[inline]
                pub fn builder() -> MonsterBuilder<()> {
                    MonsterBuilder(())
                }

                #[allow(clippy::too_many_arguments)]
                pub fn create(
                    builder: &mut ::planus::Builder,
                    field_pos: impl ::planus::WriteAsOptional<self::Vec3>,
                    field_mana: impl ::planus::WriteAsDefault<i16, i16>,
                    field_hp: impl ::planus::WriteAsDefault<i16, i16>,
                    field_name: impl ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    field_inventory: impl ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
                    field_color: impl ::planus::WriteAsDefault<self::Color, self::Color>,
                    field_weapons: impl ::planus::WriteAsOptional<
                        ::planus::Offset<[::planus::Offset<self::Weapon>]>,
                    >,
                    field_equipped: impl ::planus::WriteAsOptionalUnion<self::Equipment>,
                    field_drops: impl ::planus::WriteAsOptionalUnionVector<self::Equipment>,
                    field_path: impl ::planus::WriteAsOptional<::planus::Offset<[self::Vec3]>>,
                ) -> ::planus::Offset<Self> {
                    let prepared_pos = field_pos.prepare(builder);
                    let prepared_mana = field_mana.prepare(builder, &150);
                    let prepared_hp = field_hp.prepare(builder, &100);
                    let prepared_name = field_name.prepare(builder);
                    let prepared_inventory = field_inventory.prepare(builder);
                    let prepared_color = field_color.prepare(builder, &self::Color::Blue);
                    let prepared_weapons = field_weapons.prepare(builder);
                    let prepared_equipped = field_equipped.prepare(builder);
                    let prepared_drops = field_drops.prepare(builder);
                    let prepared_path = field_path.prepare(builder);

                    let mut table_writer: ::planus::table_writer::TableWriter<30> =
                        ::core::default::Default::default();
                    if prepared_pos.is_some() {
                        table_writer.write_entry::<self::Vec3>(0);
                    }
                    if prepared_name.is_some() {
                        table_writer.write_entry::<::planus::Offset<str>>(3);
                    }
                    if prepared_inventory.is_some() {
                        table_writer.write_entry::<::planus::Offset<[u8]>>(5);
                    }
                    if prepared_weapons.is_some() {
                        table_writer
                            .write_entry::<::planus::Offset<[::planus::Offset<self::Weapon>]>>(7);
                    }
                    if prepared_equipped.is_some() {
                        table_writer.write_entry::<::planus::Offset<self::Equipment>>(9);
                    }
                    if prepared_drops.is_some() {
                        table_writer.write_entry::<::planus::Offset<[u8]>>(10);
                    }
                    if prepared_drops.is_some() {
                        table_writer
                            .write_entry::<::planus::Offset<[::planus::Offset<self::Equipment>]>>(
                                11,
                            );
                    }
                    if prepared_path.is_some() {
                        table_writer.write_entry::<::planus::Offset<[self::Vec3]>>(12);
                    }
                    if prepared_mana.is_some() {
                        table_writer.write_entry::<i16>(1);
                    }
                    if prepared_hp.is_some() {
                        table_writer.write_entry::<i16>(2);
                    }
                    if prepared_color.is_some() {
                        table_writer.write_entry::<self::Color>(6);
                    }
                    if prepared_equipped.is_some() {
                        table_writer.write_entry::<u8>(8);
                    }

                    unsafe {
                        table_writer.finish(builder, |object_writer| {
                            if let ::core::option::Option::Some(prepared_pos) = prepared_pos {
                                object_writer.write::<_, _, 12>(&prepared_pos);
                            }
                            if let ::core::option::Option::Some(prepared_name) = prepared_name {
                                object_writer.write::<_, _, 4>(&prepared_name);
                            }
                            if let ::core::option::Option::Some(prepared_inventory) =
                                prepared_inventory
                            {
                                object_writer.write::<_, _, 4>(&prepared_inventory);
                            }
                            if let ::core::option::Option::Some(prepared_weapons) = prepared_weapons
                            {
                                object_writer.write::<_, _, 4>(&prepared_weapons);
                            }
                            if let ::core::option::Option::Some(prepared_equipped) =
                                prepared_equipped
                            {
                                object_writer.write::<_, _, 4>(&prepared_equipped.offset());
                            }
                            if let ::core::option::Option::Some(prepared_drops) = prepared_drops {
                                object_writer.write::<_, _, 4>(&prepared_drops.tags_offset());
                            }
                            if let ::core::option::Option::Some(prepared_drops) = prepared_drops {
                                object_writer.write::<_, _, 4>(&prepared_drops.values_offset());
                            }
                            if let ::core::option::Option::Some(prepared_path) = prepared_path {
                                object_writer.write::<_, _, 4>(&prepared_path);
                            }
                            if let ::core::option::Option::Some(prepared_mana) = prepared_mana {
                                object_writer.write::<_, _, 2>(&prepared_mana);
                            }
                            if let ::core::option::Option::Some(prepared_hp) = prepared_hp {
                                object_writer.write::<_, _, 2>(&prepared_hp);
                            }
                            if let ::core::option::Option::Some(prepared_color) = prepared_color {
                                object_writer.write::<_, _, 1>(&prepared_color);
                            }
                            if let ::core::option::Option::Some(prepared_equipped) =
                                prepared_equipped
                            {
                                object_writer.write::<_, _, 1>(&prepared_equipped.tag());
                            }
                        });
                    }
                    builder.current_offset()
                }
            }

            impl ::planus::WriteAs<::planus::Offset<Monster>> for Monster {
                type Prepared = ::planus::Offset<Self>;

                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Monster> {
                    ::planus::WriteAsOffset::prepare(self, builder)
                }
            }

            impl ::planus::WriteAsOptional<::planus::Offset<Monster>> for Monster {
                type Prepared = ::planus::Offset<Self>;

                #[inline]
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::Offset<Monster>> {
                    ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
                }
            }

            impl ::planus::WriteAsOffset<Monster> for Monster {
                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Monster> {
                    Monster::create(
                        builder,
                        self.pos,
                        self.mana,
                        self.hp,
                        &self.name,
                        &self.inventory,
                        self.color,
                        &self.weapons,
                        &self.equipped,
                        &self.drops,
                        &self.path,
                    )
                }
            }

            /// Builder for serializing an instance of the [Monster] type.
            ///
            /// Can be created using the [Monster::builder] method.
            #[derive(Debug)]
            #[must_use]
            pub struct MonsterBuilder<State>(State);

            impl MonsterBuilder<()> {
                /// Setter for the [`pos` field](Monster#structfield.pos).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn pos<T0>(self, value: T0) -> MonsterBuilder<(T0,)>
                where
                    T0: ::planus::WriteAsOptional<self::Vec3>,
                {
                    MonsterBuilder((value,))
                }

                /// Sets the [`pos` field](Monster#structfield.pos) to null.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn pos_as_null(self) -> MonsterBuilder<((),)> {
                    self.pos(())
                }
            }

            impl<T0> MonsterBuilder<(T0,)> {
                /// Setter for the [`mana` field](Monster#structfield.mana).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn mana<T1>(self, value: T1) -> MonsterBuilder<(T0, T1)>
                where
                    T1: ::planus::WriteAsDefault<i16, i16>,
                {
                    let (v0,) = self.0;
                    MonsterBuilder((v0, value))
                }

                /// Sets the [`mana` field](Monster#structfield.mana) to the default value.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn mana_as_default(self) -> MonsterBuilder<(T0, ::planus::DefaultValue)> {
                    self.mana(::planus::DefaultValue)
                }
            }

            impl<T0, T1> MonsterBuilder<(T0, T1)> {
                /// Setter for the [`hp` field](Monster#structfield.hp).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn hp<T2>(self, value: T2) -> MonsterBuilder<(T0, T1, T2)>
                where
                    T2: ::planus::WriteAsDefault<i16, i16>,
                {
                    let (v0, v1) = self.0;
                    MonsterBuilder((v0, v1, value))
                }

                /// Sets the [`hp` field](Monster#structfield.hp) to the default value.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn hp_as_default(self) -> MonsterBuilder<(T0, T1, ::planus::DefaultValue)> {
                    self.hp(::planus::DefaultValue)
                }
            }

            impl<T0, T1, T2> MonsterBuilder<(T0, T1, T2)> {
                /// Setter for the [`name` field](Monster#structfield.name).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn name<T3>(self, value: T3) -> MonsterBuilder<(T0, T1, T2, T3)>
                where
                    T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                {
                    let (v0, v1, v2) = self.0;
                    MonsterBuilder((v0, v1, v2, value))
                }

                /// Sets the [`name` field](Monster#structfield.name) to null.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn name_as_null(self) -> MonsterBuilder<(T0, T1, T2, ())> {
                    self.name(())
                }
            }

            impl<T0, T1, T2, T3> MonsterBuilder<(T0, T1, T2, T3)> {
                /// Setter for the [`inventory` field](Monster#structfield.inventory).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn inventory<T4>(self, value: T4) -> MonsterBuilder<(T0, T1, T2, T3, T4)>
                where
                    T4: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
                {
                    let (v0, v1, v2, v3) = self.0;
                    MonsterBuilder((v0, v1, v2, v3, value))
                }

                /// Sets the [`inventory` field](Monster#structfield.inventory) to null.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn inventory_as_null(self) -> MonsterBuilder<(T0, T1, T2, T3, ())> {
                    self.inventory(())
                }
            }

            impl<T0, T1, T2, T3, T4> MonsterBuilder<(T0, T1, T2, T3, T4)> {
                /// Setter for the [`color` field](Monster#structfield.color).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn color<T5>(self, value: T5) -> MonsterBuilder<(T0, T1, T2, T3, T4, T5)>
                where
                    T5: ::planus::WriteAsDefault<self::Color, self::Color>,
                {
                    let (v0, v1, v2, v3, v4) = self.0;
                    MonsterBuilder((v0, v1, v2, v3, v4, value))
                }

                /// Sets the [`color` field](Monster#structfield.color) to the default value.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn color_as_default(
                    self,
                ) -> MonsterBuilder<(T0, T1, T2, T3, T4, ::planus::DefaultValue)> {
                    self.color(::planus::DefaultValue)
                }
            }

            impl<T0, T1, T2, T3, T4, T5> MonsterBuilder<(T0, T1, T2, T3, T4, T5)> {
                /// Setter for the [`weapons` field](Monster#structfield.weapons).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn weapons<T6>(self, value: T6) -> MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6)>
                where
                    T6: ::planus::WriteAsOptional<
                        ::planus::Offset<[::planus::Offset<self::Weapon>]>,
                    >,
                {
                    let (v0, v1, v2, v3, v4, v5) = self.0;
                    MonsterBuilder((v0, v1, v2, v3, v4, v5, value))
                }

                /// Sets the [`weapons` field](Monster#structfield.weapons) to null.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn weapons_as_null(self) -> MonsterBuilder<(T0, T1, T2, T3, T4, T5, ())> {
                    self.weapons(())
                }
            }

            impl<T0, T1, T2, T3, T4, T5, T6> MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6)> {
                /// Setter for the [`equipped` field](Monster#structfield.equipped).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn equipped<T7>(
                    self,
                    value: T7,
                ) -> MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, T7)>
                where
                    T7: ::planus::WriteAsOptionalUnion<self::Equipment>,
                {
                    let (v0, v1, v2, v3, v4, v5, v6) = self.0;
                    MonsterBuilder((v0, v1, v2, v3, v4, v5, v6, value))
                }

                /// Sets the [`equipped` field](Monster#structfield.equipped) to null.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn equipped_as_null(self) -> MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, ())> {
                    self.equipped(())
                }
            }

            impl<T0, T1, T2, T3, T4, T5, T6, T7> MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, T7)> {
                /// Setter for the [`drops` field](Monster#structfield.drops).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn drops<T8>(
                    self,
                    value: T8,
                ) -> MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, T7, T8)>
                where
                    T8: ::planus::WriteAsOptionalUnionVector<self::Equipment>,
                {
                    let (v0, v1, v2, v3, v4, v5, v6, v7) = self.0;
                    MonsterBuilder((v0, v1, v2, v3, v4, v5, v6, v7, value))
                }

                /// Sets the [`drops` field](Monster#structfield.drops) to null.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn drops_as_null(self) -> MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, T7, ())> {
                    self.drops(())
                }
            }

            impl<T0, T1, T2, T3, T4, T5, T6, T7, T8> MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, T7, T8)> {
                /// Setter for the [`path` field](Monster#structfield.path).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn path<T9>(
                    self,
                    value: T9,
                ) -> MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)>
                where
                    T9: ::planus::WriteAsOptional<::planus::Offset<[self::Vec3]>>,
                {
                    let (v0, v1, v2, v3, v4, v5, v6, v7, v8) = self.0;
                    MonsterBuilder((v0, v1, v2, v3, v4, v5, v6, v7, v8, value))
                }

                /// Sets the [`path` field](Monster#structfield.path) to null.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn path_as_null(
                    self,
                ) -> MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, T7, T8, ())> {
                    self.path(())
                }
            }

            impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9>
                MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)>
            {
                /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [Monster].
                #[inline]
                pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<Monster>
                where
                    Self: ::planus::WriteAsOffset<Monster>,
                {
                    ::planus::WriteAsOffset::prepare(&self, builder)
                }
            }

            impl<
                    T0: ::planus::WriteAsOptional<self::Vec3>,
                    T1: ::planus::WriteAsDefault<i16, i16>,
                    T2: ::planus::WriteAsDefault<i16, i16>,
                    T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    T4: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
                    T5: ::planus::WriteAsDefault<self::Color, self::Color>,
                    T6: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::Weapon>]>>,
                    T7: ::planus::WriteAsOptionalUnion<self::Equipment>,
                    T8: ::planus::WriteAsOptionalUnionVector<self::Equipment>,
                    T9: ::planus::WriteAsOptional<::planus::Offset<[self::Vec3]>>,
                > ::planus::WriteAs<::planus::Offset<Monster>>
                for MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)>
            {
                type Prepared = ::planus::Offset<Monster>;

                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Monster> {
                    ::planus::WriteAsOffset::prepare(self, builder)
                }
            }

            impl<
                    T0: ::planus::WriteAsOptional<self::Vec3>,
                    T1: ::planus::WriteAsDefault<i16, i16>,
                    T2: ::planus::WriteAsDefault<i16, i16>,
                    T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    T4: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
                    T5: ::planus::WriteAsDefault<self::Color, self::Color>,
                    T6: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::Weapon>]>>,
                    T7: ::planus::WriteAsOptionalUnion<self::Equipment>,
                    T8: ::planus::WriteAsOptionalUnionVector<self::Equipment>,
                    T9: ::planus::WriteAsOptional<::planus::Offset<[self::Vec3]>>,
                > ::planus::WriteAsOptional<::planus::Offset<Monster>>
                for MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)>
            {
                type Prepared = ::planus::Offset<Monster>;

                #[inline]
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::Offset<Monster>> {
                    ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
                }
            }

            impl<
                    T0: ::planus::WriteAsOptional<self::Vec3>,
                    T1: ::planus::WriteAsDefault<i16, i16>,
                    T2: ::planus::WriteAsDefault<i16, i16>,
                    T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    T4: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
                    T5: ::planus::WriteAsDefault<self::Color, self::Color>,
                    T6: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::Weapon>]>>,
                    T7: ::planus::WriteAsOptionalUnion<self::Equipment>,
                    T8: ::planus::WriteAsOptionalUnionVector<self::Equipment>,
                    T9: ::planus::WriteAsOptional<::planus::Offset<[self::Vec3]>>,
                > ::planus::WriteAsOffset<Monster>
                for MonsterBuilder<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)>
            {
                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Monster> {
                    let (v0, v1, v2, v3, v4, v5, v6, v7, v8, v9) = &self.0;
                    Monster::create(builder, v0, v1, v2, v3, v4, v5, v6, v7, v8, v9)
                }
            }

            /// Reference to a deserialized [Monster].
            #[derive(Copy, Clone)]
            pub struct MonsterRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

            impl<'a> MonsterRef<'a> {
                /// Getter for the [`pos` field](Monster#structfield.pos).
                #[inline]
                pub fn pos(&self) -> ::planus::Result<::core::option::Option<self::Vec3Ref<'a>>> {
                    self.0.access(0, "Monster", "pos")
                }

                /// Getter for the [`mana` field](Monster#structfield.mana).
                #[inline]
                pub fn mana(&self) -> ::planus::Result<i16> {
                    ::core::result::Result::Ok(self.0.access(1, "Monster", "mana")?.unwrap_or(150))
                }

                /// Getter for the [`hp` field](Monster#structfield.hp).
                #[inline]
                pub fn hp(&self) -> ::planus::Result<i16> {
                    ::core::result::Result::Ok(self.0.access(2, "Monster", "hp")?.unwrap_or(100))
                }

                /// Getter for the [`name` field](Monster#structfield.name).
                #[inline]
                pub fn name(
                    &self,
                ) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>>
                {
                    self.0.access(3, "Monster", "name")
                }

                /// Getter for the [`inventory` field](Monster#structfield.inventory).
                #[inline]
                pub fn inventory(&self) -> ::planus::Result<::core::option::Option<&'a [u8]>> {
                    self.0.access(5, "Monster", "inventory")
                }

                /// Getter for the [`color` field](Monster#structfield.color).
                #[inline]
                pub fn color(&self) -> ::planus::Result<self::Color> {
                    ::core::result::Result::Ok(
                        self.0
                            .access(6, "Monster", "color")?
                            .unwrap_or(self::Color::Blue),
                    )
                }

                /// Getter for the [`weapons` field](Monster#structfield.weapons).
                #[inline]
                pub fn weapons(
                    &self,
                ) -> ::planus::Result<
                    ::core::option::Option<
                        ::planus::Vector<'a, ::planus::Result<self::WeaponRef<'a>>>,
                    >,
                > {
                    self.0.access(7, "Monster", "weapons")
                }

                /// Getter for the [`equipped` field](Monster#structfield.equipped).
                #[inline]
                pub fn equipped(
                    &self,
                ) -> ::planus::Result<::core::option::Option<self::EquipmentRef<'a>>>
                {
                    self.0.access_union(8, "Monster", "equipped")
                }

                /// Getter for the [`drops` field](Monster#structfield.drops).
                #[inline]
                pub fn drops(
                    &self,
                ) -> ::planus::Result<
                    ::core::option::Option<::planus::UnionVector<'a, self::EquipmentRef<'a>>>,
                > {
                    self.0.access_union_vector(10, "Monster", "drops")
                }

                /// Getter for the [`path` field](Monster#structfield.path).
                #[inline]
                pub fn path(
                    &self,
                ) -> ::planus::Result<::core::option::Option<::planus::Vector<'a, self::Vec3Ref<'a>>>>
                {
                    self.0.access(12, "Monster", "path")
                }
            }

            impl<'a> ::core::fmt::Debug for MonsterRef<'a> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    let mut f = f.debug_struct("MonsterRef");
                    if let ::core::option::Option::Some(field_pos) = self.pos().transpose() {
                        f.field("pos", &field_pos);
                    }
                    f.field("mana", &self.mana());
                    f.field("hp", &self.hp());
                    if let ::core::option::Option::Some(field_name) = self.name().transpose() {
                        f.field("name", &field_name);
                    }
                    if let ::core::option::Option::Some(field_inventory) =
                        self.inventory().transpose()
                    {
                        f.field("inventory", &field_inventory);
                    }
                    f.field("color", &self.color());
                    if let ::core::option::Option::Some(field_weapons) = self.weapons().transpose()
                    {
                        f.field("weapons", &field_weapons);
                    }
                    if let ::core::option::Option::Some(field_equipped) =
                        self.equipped().transpose()
                    {
                        f.field("equipped", &field_equipped);
                    }
                    if let ::core::option::Option::Some(field_drops) = self.drops().transpose() {
                        f.field("drops", &field_drops);
                    }
                    if let ::core::option::Option::Some(field_path) = self.path().transpose() {
                        f.field("path", &field_path);
                    }
                    f.finish()
                }
            }

            impl<'a> ::core::convert::TryFrom<MonsterRef<'a>> for Monster {
                type Error = ::planus::Error;

                #[allow(unreachable_code)]
                fn try_from(value: MonsterRef<'a>) -> ::planus::Result<Self> {
                    ::core::result::Result::Ok(Self {
                        pos: value.pos()?.map(::core::convert::Into::into),
                        mana: ::core::convert::TryInto::try_into(value.mana()?)?,
                        hp: ::core::convert::TryInto::try_into(value.hp()?)?,
                        name: value.name()?.map(::core::convert::Into::into),
                        inventory: value.inventory()?.map(|v| v.to_vec()),
                        color: ::core::convert::TryInto::try_into(value.color()?)?,
                        weapons: if let ::core::option::Option::Some(weapons) = value.weapons()? {
                            ::core::option::Option::Some(weapons.to_vec_result()?)
                        } else {
                            ::core::option::Option::None
                        },
                        equipped: if let ::core::option::Option::Some(equipped) =
                            value.equipped()?
                        {
                            ::core::option::Option::Some(::core::convert::TryInto::try_into(
                                equipped,
                            )?)
                        } else {
                            ::core::option::Option::None
                        },
                        drops: if let ::core::option::Option::Some(drops) = value.drops()? {
                            ::core::option::Option::Some(drops.to_vec()?)
                        } else {
                            ::core::option::Option::None
                        },
                        path: if let ::core::option::Option::Some(path) = value.path()? {
                            ::core::option::Option::Some(path.to_vec()?)
                        } else {
                            ::core::option::Option::None
                        },
                    })
                }
            }

            impl<'a> ::planus::TableRead<'a> for MonsterRef<'a> {
                #[inline]
                fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'a>,
                    offset: usize,
                ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                    ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                        buffer, offset,
                    )?))
                }
            }

            impl<'a> ::planus::VectorReadInner<'a> for MonsterRef<'a> {
                type Error = ::planus::Error;
                const STRIDE: usize = 4;

                unsafe fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'a>,
                    offset: usize,
                ) -> ::planus::Result<Self> {
                    ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                        error_kind.with_error_location(
                            "[MonsterRef]",
                            "get",
                            buffer.offset_from_start,
                        )
                    })
                }
            }

            /// # Safety
            /// The planus compiler generates implementations that initialize
            /// the bytes in `write_values`.
            unsafe impl ::planus::VectorWrite<::planus::Offset<Monster>> for Monster {
                type Value = ::planus::Offset<Monster>;
                const STRIDE: usize = 4;
                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                    ::planus::WriteAs::prepare(self, builder)
                }

                #[inline]
                unsafe fn write_values(
                    values: &[::planus::Offset<Monster>],
                    bytes: *mut ::core::mem::MaybeUninit<u8>,
                    buffer_position: u32,
                ) {
                    let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                    for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                        ::planus::WriteAsPrimitive::write(
                            v,
                            ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                            buffer_position - (Self::STRIDE * i) as u32,
                        );
                    }
                }
            }

            impl<'a> ::planus::ReadAsRoot<'a> for MonsterRef<'a> {
                fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                    ::planus::TableRead::from_buffer(
                        ::planus::SliceWithStartOffset {
                            buffer: slice,
                            offset_from_start: 0,
                        },
                        0,
                    )
                    .map_err(|error_kind| {
                        error_kind.with_error_location("[MonsterRef]", "read_as_root", 0)
                    })
                }
            }

            ///  A weapon is equipment that can be used for attacking
            ///
            /// Generated from these locations:
            /// * Table `Weapon` in the file `examples/rust/monster.fbs:59`
            #[derive(
                Clone,
                Debug,
                PartialEq,
                PartialOrd,
                Eq,
                Ord,
                Hash,
                ::serde::Serialize,
                ::serde::Deserialize,
            )]
            pub struct Weapon {
                ///  The name of the weapon
                pub name: ::core::option::Option<::planus::alloc::string::String>,
                ///  The damage of the weapon
                pub damage: i16,
            }

            #[allow(clippy::derivable_impls)]
            impl ::core::default::Default for Weapon {
                fn default() -> Self {
                    Self {
                        name: ::core::default::Default::default(),
                        damage: 0,
                    }
                }
            }

            impl Weapon {
                /// Creates a [WeaponBuilder] for serializing an instance of this table.
                #[inline]
                pub fn builder() -> WeaponBuilder<()> {
                    WeaponBuilder(())
                }

                #[allow(clippy::too_many_arguments)]
                pub fn create(
                    builder: &mut ::planus::Builder,
                    field_name: impl ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    field_damage: impl ::planus::WriteAsDefault<i16, i16>,
                ) -> ::planus::Offset<Self> {
                    let prepared_name = field_name.prepare(builder);
                    let prepared_damage = field_damage.prepare(builder, &0);

                    let mut table_writer: ::planus::table_writer::TableWriter<8> =
                        ::core::default::Default::default();
                    if prepared_name.is_some() {
                        table_writer.write_entry::<::planus::Offset<str>>(0);
                    }
                    if prepared_damage.is_some() {
                        table_writer.write_entry::<i16>(1);
                    }

                    unsafe {
                        table_writer.finish(builder, |object_writer| {
                            if let ::core::option::Option::Some(prepared_name) = prepared_name {
                                object_writer.write::<_, _, 4>(&prepared_name);
                            }
                            if let ::core::option::Option::Some(prepared_damage) = prepared_damage {
                                object_writer.write::<_, _, 2>(&prepared_damage);
                            }
                        });
                    }
                    builder.current_offset()
                }
            }

            impl ::planus::WriteAs<::planus::Offset<Weapon>> for Weapon {
                type Prepared = ::planus::Offset<Self>;

                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Weapon> {
                    ::planus::WriteAsOffset::prepare(self, builder)
                }
            }

            impl ::planus::WriteAsOptional<::planus::Offset<Weapon>> for Weapon {
                type Prepared = ::planus::Offset<Self>;

                #[inline]
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::Offset<Weapon>> {
                    ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
                }
            }

            impl ::planus::WriteAsOffset<Weapon> for Weapon {
                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Weapon> {
                    Weapon::create(builder, &self.name, self.damage)
                }
            }

            /// Builder for serializing an instance of the [Weapon] type.
            ///
            /// Can be created using the [Weapon::builder] method.
            #[derive(Debug)]
            #[must_use]
            pub struct WeaponBuilder<State>(State);

            impl WeaponBuilder<()> {
                /// Setter for the [`name` field](Weapon#structfield.name).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn name<T0>(self, value: T0) -> WeaponBuilder<(T0,)>
                where
                    T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                {
                    WeaponBuilder((value,))
                }

                /// Sets the [`name` field](Weapon#structfield.name) to null.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn name_as_null(self) -> WeaponBuilder<((),)> {
                    self.name(())
                }
            }

            impl<T0> WeaponBuilder<(T0,)> {
                /// Setter for the [`damage` field](Weapon#structfield.damage).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn damage<T1>(self, value: T1) -> WeaponBuilder<(T0, T1)>
                where
                    T1: ::planus::WriteAsDefault<i16, i16>,
                {
                    let (v0,) = self.0;
                    WeaponBuilder((v0, value))
                }

                /// Sets the [`damage` field](Weapon#structfield.damage) to the default value.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn damage_as_default(self) -> WeaponBuilder<(T0, ::planus::DefaultValue)> {
                    self.damage(::planus::DefaultValue)
                }
            }

            impl<T0, T1> WeaponBuilder<(T0, T1)> {
                /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [Weapon].
                #[inline]
                pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<Weapon>
                where
                    Self: ::planus::WriteAsOffset<Weapon>,
                {
                    ::planus::WriteAsOffset::prepare(&self, builder)
                }
            }

            impl<
                    T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    T1: ::planus::WriteAsDefault<i16, i16>,
                > ::planus::WriteAs<::planus::Offset<Weapon>> for WeaponBuilder<(T0, T1)>
            {
                type Prepared = ::planus::Offset<Weapon>;

                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Weapon> {
                    ::planus::WriteAsOffset::prepare(self, builder)
                }
            }

            impl<
                    T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    T1: ::planus::WriteAsDefault<i16, i16>,
                > ::planus::WriteAsOptional<::planus::Offset<Weapon>> for WeaponBuilder<(T0, T1)>
            {
                type Prepared = ::planus::Offset<Weapon>;

                #[inline]
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::Offset<Weapon>> {
                    ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
                }
            }

            impl<
                    T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    T1: ::planus::WriteAsDefault<i16, i16>,
                > ::planus::WriteAsOffset<Weapon> for WeaponBuilder<(T0, T1)>
            {
                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Weapon> {
                    let (v0, v1) = &self.0;
                    Weapon::create(builder, v0, v1)
                }
            }

            /// Reference to a deserialized [Weapon].
            #[derive(Copy, Clone)]
            pub struct WeaponRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

            impl<'a> WeaponRef<'a> {
                /// Getter for the [`name` field](Weapon#structfield.name).
                #[inline]
                pub fn name(
                    &self,
                ) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>>
                {
                    self.0.access(0, "Weapon", "name")
                }

                /// Getter for the [`damage` field](Weapon#structfield.damage).
                #[inline]
                pub fn damage(&self) -> ::planus::Result<i16> {
                    ::core::result::Result::Ok(self.0.access(1, "Weapon", "damage")?.unwrap_or(0))
                }
            }

            impl<'a> ::core::fmt::Debug for WeaponRef<'a> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    let mut f = f.debug_struct("WeaponRef");
                    if let ::core::option::Option::Some(field_name) = self.name().transpose() {
                        f.field("name", &field_name);
                    }
                    f.field("damage", &self.damage());
                    f.finish()
                }
            }

            impl<'a> ::core::convert::TryFrom<WeaponRef<'a>> for Weapon {
                type Error = ::planus::Error;

                #[allow(unreachable_code)]
                fn try_from(value: WeaponRef<'a>) -> ::planus::Result<Self> {
                    ::core::result::Result::Ok(Self {
                        name: value.name()?.map(::core::convert::Into::into),
                        damage: ::core::convert::TryInto::try_into(value.damage()?)?,
                    })
                }
            }

            impl<'a> ::planus::TableRead<'a> for WeaponRef<'a> {
                #[inline]
                fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'a>,
                    offset: usize,
                ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                    ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                        buffer, offset,
                    )?))
                }
            }

            impl<'a> ::planus::VectorReadInner<'a> for WeaponRef<'a> {
                type Error = ::planus::Error;
                const STRIDE: usize = 4;

                unsafe fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'a>,
                    offset: usize,
                ) -> ::planus::Result<Self> {
                    ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                        error_kind.with_error_location(
                            "[WeaponRef]",
                            "get",
                            buffer.offset_from_start,
                        )
                    })
                }
            }

            /// # Safety
            /// The planus compiler generates implementations that initialize
            /// the bytes in `write_values`.
            unsafe impl ::planus::VectorWrite<::planus::Offset<Weapon>> for Weapon {
                type Value = ::planus::Offset<Weapon>;
                const STRIDE: usize = 4;
                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                    ::planus::WriteAs::prepare(self, builder)
                }

                #[inline]
                unsafe fn write_values(
                    values: &[::planus::Offset<Weapon>],
                    bytes: *mut ::core::mem::MaybeUninit<u8>,
                    buffer_position: u32,
                ) {
                    let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                    for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                        ::planus::WriteAsPrimitive::write(
                            v,
                            ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                            buffer_position - (Self::STRIDE * i) as u32,
                        );
                    }
                }
            }

            impl<'a> ::planus::ReadAsRoot<'a> for WeaponRef<'a> {
                fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                    ::planus::TableRead::from_buffer(
                        ::planus::SliceWithStartOffset {
                            buffer: slice,
                            offset_from_start: 0,
                        },
                        0,
                    )
                    .map_err(|error_kind| {
                        error_kind.with_error_location("[WeaponRef]", "read_as_root", 0)
                    })
                }
            }

            ///  A shield is equipment that can be used for defending
            ///
            /// Generated from these locations:
            /// * Table `Shield` in the file `examples/rust/monster.fbs:67`
            #[derive(
                Clone,
                Debug,
                PartialEq,
                PartialOrd,
                Eq,
                Ord,
                Hash,
                ::serde::Serialize,
                ::serde::Deserialize,
            )]
            pub struct Shield {
                ///  The name of the shield
                pub name: ::core::option::Option<::planus::alloc::string::String>,
                ///  The armor of the shield
                pub armor: i16,
            }

            #[allow(clippy::derivable_impls)]
            impl ::core::default::Default for Shield {
                fn default() -> Self {
                    Self {
                        name: ::core::default::Default::default(),
                        armor: 0,
                    }
                }
            }

            impl Shield {
                /// Creates a [ShieldBuilder] for serializing an instance of this table.
                #[inline]
                pub fn builder() -> ShieldBuilder<()> {
                    ShieldBuilder(())
                }

                #[allow(clippy::too_many_arguments)]
                pub fn create(
                    builder: &mut ::planus::Builder,
                    field_name: impl ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    field_armor: impl ::planus::WriteAsDefault<i16, i16>,
                ) -> ::planus::Offset<Self> {
                    let prepared_name = field_name.prepare(builder);
                    let prepared_armor = field_armor.prepare(builder, &0);

                    let mut table_writer: ::planus::table_writer::TableWriter<8> =
                        ::core::default::Default::default();
                    if prepared_name.is_some() {
                        table_writer.write_entry::<::planus::Offset<str>>(0);
                    }
                    if prepared_armor.is_some() {
                        table_writer.write_entry::<i16>(1);
                    }

                    unsafe {
                        table_writer.finish(builder, |object_writer| {
                            if let ::core::option::Option::Some(prepared_name) = prepared_name {
                                object_writer.write::<_, _, 4>(&prepared_name);
                            }
                            if let ::core::option::Option::Some(prepared_armor) = prepared_armor {
                                object_writer.write::<_, _, 2>(&prepared_armor);
                            }
                        });
                    }
                    builder.current_offset()
                }
            }

            impl ::planus::WriteAs<::planus::Offset<Shield>> for Shield {
                type Prepared = ::planus::Offset<Self>;

                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Shield> {
                    ::planus::WriteAsOffset::prepare(self, builder)
                }
            }

            impl ::planus::WriteAsOptional<::planus::Offset<Shield>> for Shield {
                type Prepared = ::planus::Offset<Self>;

                #[inline]
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::Offset<Shield>> {
                    ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
                }
            }

            impl ::planus::WriteAsOffset<Shield> for Shield {
                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Shield> {
                    Shield::create(builder, &self.name, self.armor)
                }
            }

            /// Builder for serializing an instance of the [Shield] type.
            ///
            /// Can be created using the [Shield::builder] method.
            #[derive(Debug)]
            #[must_use]
            pub struct ShieldBuilder<State>(State);

            impl ShieldBuilder<()> {
                /// Setter for the [`name` field](Shield#structfield.name).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn name<T0>(self, value: T0) -> ShieldBuilder<(T0,)>
                where
                    T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                {
                    ShieldBuilder((value,))
                }

                /// Sets the [`name` field](Shield#structfield.name) to null.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn name_as_null(self) -> ShieldBuilder<((),)> {
                    self.name(())
                }
            }

            impl<T0> ShieldBuilder<(T0,)> {
                /// Setter for the [`armor` field](Shield#structfield.armor).
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn armor<T1>(self, value: T1) -> ShieldBuilder<(T0, T1)>
                where
                    T1: ::planus::WriteAsDefault<i16, i16>,
                {
                    let (v0,) = self.0;
                    ShieldBuilder((v0, value))
                }

                /// Sets the [`armor` field](Shield#structfield.armor) to the default value.
                #[inline]
                #[allow(clippy::type_complexity)]
                pub fn armor_as_default(self) -> ShieldBuilder<(T0, ::planus::DefaultValue)> {
                    self.armor(::planus::DefaultValue)
                }
            }

            impl<T0, T1> ShieldBuilder<(T0, T1)> {
                /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [Shield].
                #[inline]
                pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<Shield>
                where
                    Self: ::planus::WriteAsOffset<Shield>,
                {
                    ::planus::WriteAsOffset::prepare(&self, builder)
                }
            }

            impl<
                    T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    T1: ::planus::WriteAsDefault<i16, i16>,
                > ::planus::WriteAs<::planus::Offset<Shield>> for ShieldBuilder<(T0, T1)>
            {
                type Prepared = ::planus::Offset<Shield>;

                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Shield> {
                    ::planus::WriteAsOffset::prepare(self, builder)
                }
            }

            impl<
                    T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    T1: ::planus::WriteAsDefault<i16, i16>,
                > ::planus::WriteAsOptional<::planus::Offset<Shield>> for ShieldBuilder<(T0, T1)>
            {
                type Prepared = ::planus::Offset<Shield>;

                #[inline]
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::Offset<Shield>> {
                    ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
                }
            }

            impl<
                    T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    T1: ::planus::WriteAsDefault<i16, i16>,
                > ::planus::WriteAsOffset<Shield> for ShieldBuilder<(T0, T1)>
            {
                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Shield> {
                    let (v0, v1) = &self.0;
                    Shield::create(builder, v0, v1)
                }
            }

            /// Reference to a deserialized [Shield].
            #[derive(Copy, Clone)]
            pub struct ShieldRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

            impl<'a> ShieldRef<'a> {
                /// Getter for the [`name` field](Shield#structfield.name).
                #[inline]
                pub fn name(
                    &self,
                ) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>>
                {
                    self.0.access(0, "Shield", "name")
                }

                /// Getter for the [`armor` field](Shield#structfield.armor).
                #[inline]
                pub fn armor(&self) -> ::planus::Result<i16> {
                    ::core::result::Result::Ok(self.0.access(1, "Shield", "armor")?.unwrap_or(0))
                }
            }

            impl<'a> ::core::fmt::Debug for ShieldRef<'a> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    let mut f = f.debug_struct("ShieldRef");
                    if let ::core::option::Option::Some(field_name) = self.name().transpose() {
                        f.field("name", &field_name);
                    }
                    f.field("armor", &self.armor());
                    f.finish()
                }
            }

            impl<'a> ::core::convert::TryFrom<ShieldRef<'a>> for Shield {
                type Error = ::planus::Error;

                #[allow(unreachable_code)]
                fn try_from(value: ShieldRef<'a>) -> ::planus::Result<Self> {
                    ::core::result::Result::Ok(Self {
                        name: value.name()?.map(::core::convert::Into::into),
                        armor: ::core::convert::TryInto::try_into(value.armor()?)?,
                    })
                }
            }

            impl<'a> ::planus::TableRead<'a> for ShieldRef<'a> {
                #[inline]
                fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'a>,
                    offset: usize,
                ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                    ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                        buffer, offset,
                    )?))
                }
            }

            impl<'a> ::planus::VectorReadInner<'a> for ShieldRef<'a> {
                type Error = ::planus::Error;
                const STRIDE: usize = 4;

                unsafe fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'a>,
                    offset: usize,
                ) -> ::planus::Result<Self> {
                    ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                        error_kind.with_error_location(
                            "[ShieldRef]",
                            "get",
                            buffer.offset_from_start,
                        )
                    })
                }
            }

            /// # Safety
            /// The planus compiler generates implementations that initialize
            /// the bytes in `write_values`.
            unsafe impl ::planus::VectorWrite<::planus::Offset<Shield>> for Shield {
                type Value = ::planus::Offset<Shield>;
                const STRIDE: usize = 4;
                #[inline]
                fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                    ::planus::WriteAs::prepare(self, builder)
                }

                #[inline]
                unsafe fn write_values(
                    values: &[::planus::Offset<Shield>],
                    bytes: *mut ::core::mem::MaybeUninit<u8>,
                    buffer_position: u32,
                ) {
                    let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                    for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                        ::planus::WriteAsPrimitive::write(
                            v,
                            ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                            buffer_position - (Self::STRIDE * i) as u32,
                        );
                    }
                }
            }

            impl<'a> ::planus::ReadAsRoot<'a> for ShieldRef<'a> {
                fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                    ::planus::TableRead::from_buffer(
                        ::planus::SliceWithStartOffset {
                            buffer: slice,
                            offset_from_start: 0,
                        },
                        0,
                    )
                    .map_err(|error_kind| {
                        error_kind.with_error_location("[ShieldRef]", "read_as_root", 0)
                    })
                }
            }
        }
    }
}
