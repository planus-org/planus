pub use root::*;

const _: () = ::planus::check_version_compatibility("planus-0.3.1");

#[no_implicit_prelude]
mod root {
    pub mod my_game {
        pub mod sample {
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
                Red = 0,
                Green = 1,
                Blue = 2,
            }

            impl ::core::convert::TryFrom<i8> for Color {
                type Error = ::planus::errors::UnknownEnumTagKind;
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
                fn from(value: Color) -> Self {
                    value as i8
                }
            }

            impl ::planus::Primitive for Color {
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
                    let value = <i8 as ::planus::VectorRead>::from_buffer(buffer, offset);
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

            impl ::planus::VectorWrite<Color> for Color {
                const STRIDE: usize = 1;

                type Value = Self;

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
                            ::planus::Cursor::new(&mut *bytes.add(i)),
                            buffer_position - i as u32,
                        );
                    }
                }
            }

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
                Weapon(::planus::alloc::boxed::Box<self::Weapon>),
            }

            impl Equipment {
                pub fn create_weapon(
                    builder: &mut ::planus::Builder,
                    value: impl ::planus::WriteAsOffset<self::Weapon>,
                ) -> ::planus::UnionOffset<Self> {
                    ::planus::UnionOffset::new(1, value.prepare(builder).downcast())
                }
            }

            impl ::planus::WriteAsUnion<Equipment> for Equipment {
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Self> {
                    match self {
                        Self::Weapon(value) => Self::create_weapon(builder, value),
                    }
                }
            }

            impl ::planus::WriteAsOptionalUnion<Equipment> for Equipment {
                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::UnionOffset<Self>> {
                    ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
                }
            }

            #[derive(Copy, Clone, Debug)]
            pub enum EquipmentRef<'a> {
                Weapon(self::WeaponRef<'a>),
            }

            impl<'a> ::core::convert::TryFrom<EquipmentRef<'a>> for Equipment {
                type Error = ::planus::Error;

                fn try_from(value: EquipmentRef<'a>) -> ::planus::Result<Self> {
                    ::core::result::Result::Ok(match value {
                        EquipmentRef::Weapon(value) => {
                            Equipment::Weapon(::planus::alloc::boxed::Box::new(
                                ::core::convert::TryFrom::try_from(value)?,
                            ))
                        }
                    })
                }
            }

            impl<'a> ::planus::TableReadUnion<'a> for EquipmentRef<'a> {
                fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'a>,
                    field_offset: usize,
                    tag: u8,
                ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                    match tag {
                        1 => ::core::result::Result::Ok(Self::Weapon(
                            ::planus::TableRead::from_buffer(buffer, field_offset)?,
                        )),
                        _ => ::core::result::Result::Err(
                            ::planus::errors::ErrorKind::UnknownUnionTag { tag },
                        ),
                    }
                }
            }

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
                pub x: f32,
                pub y: f32,
                pub z: f32,
            }

            impl ::planus::Primitive for Vec3 {
                const ALIGNMENT: usize = 4;
                const SIZE: usize = 12;
            }

            #[allow(clippy::identity_op)]
            impl ::planus::WriteAsPrimitive<Vec3> for Vec3 {
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
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Vec3> {
                    unsafe {
                        builder.write_with(12, 4, |buffer_position, bytes| {
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
                fn prepare(&self, _builder: &mut ::planus::Builder) -> Self {
                    *self
                }
            }

            impl ::planus::WriteAsOptional<Vec3> for Vec3 {
                type Prepared = Self;
                fn prepare(
                    &self,
                    _builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<Self> {
                    ::core::option::Option::Some(*self)
                }
            }

            #[derive(Copy, Clone)]
            pub struct Vec3Ref<'a>(::planus::ArrayWithStartOffset<'a, 12>);

            impl<'a> Vec3Ref<'a> {
                pub fn x(&self) -> f32 {
                    let buffer = self.0.advance_as_array::<4>(0).unwrap();

                    f32::from_le_bytes(*buffer.as_array())
                }

                pub fn y(&self) -> f32 {
                    let buffer = self.0.advance_as_array::<4>(4).unwrap();

                    f32::from_le_bytes(*buffer.as_array())
                }

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

            impl<'a> ::core::convert::TryFrom<Vec3Ref<'a>> for Vec3 {
                type Error = ::planus::Error;

                #[allow(unreachable_code)]
                fn try_from(value: Vec3Ref<'a>) -> ::planus::Result<Self> {
                    ::core::result::Result::Ok(Vec3 {
                        x: value.x(),
                        y: value.y(),
                        z: value.z(),
                    })
                }
            }

            impl<'a> ::planus::TableRead<'a> for Vec3Ref<'a> {
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

                unsafe fn from_buffer(
                    buffer: ::planus::SliceWithStartOffset<'a>,
                    offset: usize,
                ) -> Self {
                    Self(buffer.unchecked_advance_as_array(offset))
                }
            }

            impl ::planus::VectorWrite<Vec3> for Vec3 {
                const STRIDE: usize = 12;

                type Value = Vec3;

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
                            ::planus::Cursor::new(&mut *bytes.add(i)),
                            buffer_position - (12 * i) as u32,
                        );
                    }
                }
            }

            #[derive(
                Clone, Debug, PartialEq, PartialOrd, ::serde::Serialize, ::serde::Deserialize,
            )]
            pub struct Monster {
                pub pos: ::core::option::Option<self::Vec3>,
                pub mana: i16,
                pub hp: i16,
                pub name: ::core::option::Option<::planus::alloc::string::String>,
                pub inventory: ::core::option::Option<::planus::alloc::vec::Vec<u8>>,
                pub color: self::Color,
                pub weapons: ::core::option::Option<::planus::alloc::vec::Vec<self::Weapon>>,
                pub equipped: ::core::option::Option<self::Equipment>,
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
                        path: ::core::default::Default::default(),
                    }
                }
            }

            impl Monster {
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

                    let prepared_path = field_path.prepare(builder);

                    let mut table_writer =
                        ::planus::table_writer::TableWriter::<24, 39>::new(builder);

                    if prepared_pos.is_some() {
                        table_writer.calculate_size::<self::Vec3>(2);
                    }
                    if prepared_mana.is_some() {
                        table_writer.calculate_size::<i16>(4);
                    }
                    if prepared_hp.is_some() {
                        table_writer.calculate_size::<i16>(6);
                    }
                    if prepared_name.is_some() {
                        table_writer.calculate_size::<::planus::Offset<str>>(8);
                    }
                    if prepared_inventory.is_some() {
                        table_writer.calculate_size::<::planus::Offset<[u8]>>(12);
                    }
                    if prepared_color.is_some() {
                        table_writer.calculate_size::<self::Color>(14);
                    }
                    if prepared_weapons.is_some() {
                        table_writer
                            .calculate_size::<::planus::Offset<[::planus::Offset<self::Weapon>]>>(
                                16,
                            );
                    }
                    if prepared_equipped.is_some() {
                        table_writer.calculate_size::<u8>(18);
                        table_writer.calculate_size::<::planus::Offset<self::Equipment>>(20);
                    }
                    if prepared_path.is_some() {
                        table_writer.calculate_size::<::planus::Offset<[self::Vec3]>>(22);
                    }

                    table_writer.finish_calculating();

                    unsafe {
                        if let ::core::option::Option::Some(prepared_pos) = prepared_pos {
                            table_writer.write::<_, _, 12>(0, &prepared_pos);
                        }
                        if let ::core::option::Option::Some(prepared_name) = prepared_name {
                            table_writer.write::<_, _, 4>(3, &prepared_name);
                        }
                        if let ::core::option::Option::Some(prepared_inventory) = prepared_inventory
                        {
                            table_writer.write::<_, _, 4>(5, &prepared_inventory);
                        }
                        if let ::core::option::Option::Some(prepared_weapons) = prepared_weapons {
                            table_writer.write::<_, _, 4>(7, &prepared_weapons);
                        }
                        if let ::core::option::Option::Some(prepared_equipped) = prepared_equipped {
                            table_writer.write::<_, _, 4>(9, &prepared_equipped.offset());
                        }
                        if let ::core::option::Option::Some(prepared_path) = prepared_path {
                            table_writer.write::<_, _, 4>(10, &prepared_path);
                        }
                        if let ::core::option::Option::Some(prepared_mana) = prepared_mana {
                            table_writer.write::<_, _, 2>(1, &prepared_mana);
                        }
                        if let ::core::option::Option::Some(prepared_hp) = prepared_hp {
                            table_writer.write::<_, _, 2>(2, &prepared_hp);
                        }
                        if let ::core::option::Option::Some(prepared_color) = prepared_color {
                            table_writer.write::<_, _, 1>(6, &prepared_color);
                        }
                        if let ::core::option::Option::Some(prepared_equipped) = prepared_equipped {
                            table_writer.write::<_, _, 1>(8, &prepared_equipped.tag());
                        }
                    }

                    table_writer.finish()
                }
            }

            impl ::planus::WriteAs<::planus::Offset<Monster>> for Monster {
                type Prepared = ::planus::Offset<Self>;

                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Monster> {
                    ::planus::WriteAsOffset::prepare(self, builder)
                }
            }

            impl ::planus::WriteAsOptional<::planus::Offset<Monster>> for Monster {
                type Prepared = ::planus::Offset<Self>;

                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::Offset<Monster>> {
                    ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
                }
            }

            impl ::planus::WriteAsOffset<Monster> for Monster {
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Monster> {
                    Monster::create(
                        builder,
                        &self.pos,
                        &self.mana,
                        &self.hp,
                        &self.name,
                        &self.inventory,
                        &self.color,
                        &self.weapons,
                        &self.equipped,
                        &self.path,
                    )
                }
            }

            #[derive(Copy, Clone)]
            pub struct MonsterRef<'a>(::planus::table_reader::Table<'a>);

            impl<'a> MonsterRef<'a> {
                pub fn pos(&self) -> ::planus::Result<::core::option::Option<self::Vec3Ref<'a>>> {
                    self.0.access(0, "Monster", "pos")
                }

                pub fn mana(&self) -> ::planus::Result<i16> {
                    ::core::result::Result::Ok(self.0.access(1, "Monster", "mana")?.unwrap_or(150))
                }

                pub fn hp(&self) -> ::planus::Result<i16> {
                    ::core::result::Result::Ok(self.0.access(2, "Monster", "hp")?.unwrap_or(100))
                }

                pub fn name(
                    &self,
                ) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>>
                {
                    self.0.access(3, "Monster", "name")
                }

                pub fn inventory(
                    &self,
                ) -> ::planus::Result<::core::option::Option<::planus::Vector<'a, u8>>>
                {
                    self.0.access(5, "Monster", "inventory")
                }

                pub fn color(&self) -> ::planus::Result<self::Color> {
                    ::core::result::Result::Ok(
                        self.0
                            .access(6, "Monster", "color")?
                            .unwrap_or(self::Color::Blue),
                    )
                }

                pub fn weapons(
                    &self,
                ) -> ::planus::Result<
                    ::core::option::Option<
                        ::planus::Vector<'a, ::planus::Result<self::WeaponRef<'a>>>,
                    >,
                > {
                    self.0.access(7, "Monster", "weapons")
                }

                pub fn equipped(
                    &self,
                ) -> ::planus::Result<::core::option::Option<self::EquipmentRef<'a>>>
                {
                    self.0.access_union(8, "Monster", "equipped")
                }

                pub fn path(
                    &self,
                ) -> ::planus::Result<::core::option::Option<::planus::Vector<'a, self::Vec3Ref<'a>>>>
                {
                    self.0.access(10, "Monster", "path")
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
                        pos: if let ::core::option::Option::Some(pos) = value.pos()? {
                            ::core::option::Option::Some(::core::convert::TryInto::try_into(pos)?)
                        } else {
                            ::core::option::Option::None
                        },
                        mana: ::core::convert::TryInto::try_into(value.mana()?)?,
                        hp: ::core::convert::TryInto::try_into(value.hp()?)?,
                        name: if let ::core::option::Option::Some(name) = value.name()? {
                            ::core::option::Option::Some(::core::convert::TryInto::try_into(name)?)
                        } else {
                            ::core::option::Option::None
                        },
                        inventory: if let ::core::option::Option::Some(inventory) =
                            value.inventory()?
                        {
                            ::core::option::Option::Some(inventory.to_vec()?)
                        } else {
                            ::core::option::Option::None
                        },
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
                        path: if let ::core::option::Option::Some(path) = value.path()? {
                            ::core::option::Option::Some(path.to_vec()?)
                        } else {
                            ::core::option::Option::None
                        },
                    })
                }
            }

            impl<'a> ::planus::TableRead<'a> for MonsterRef<'a> {
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

            impl ::planus::VectorWrite<::planus::Offset<Monster>> for Monster {
                type Value = ::planus::Offset<Monster>;
                const STRIDE: usize = 4;
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
                            ::planus::Cursor::new(&mut *bytes.add(i)),
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
                pub name: ::core::option::Option<::planus::alloc::string::String>,
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
                #[allow(clippy::too_many_arguments)]
                pub fn create(
                    builder: &mut ::planus::Builder,
                    field_name: impl ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                    field_damage: impl ::planus::WriteAsDefault<i16, i16>,
                ) -> ::planus::Offset<Self> {
                    let prepared_name = field_name.prepare(builder);

                    let prepared_damage = field_damage.prepare(builder, &0);

                    let mut table_writer =
                        ::planus::table_writer::TableWriter::<6, 6>::new(builder);

                    if prepared_name.is_some() {
                        table_writer.calculate_size::<::planus::Offset<str>>(2);
                    }
                    if prepared_damage.is_some() {
                        table_writer.calculate_size::<i16>(4);
                    }

                    table_writer.finish_calculating();

                    unsafe {
                        if let ::core::option::Option::Some(prepared_name) = prepared_name {
                            table_writer.write::<_, _, 4>(0, &prepared_name);
                        }
                        if let ::core::option::Option::Some(prepared_damage) = prepared_damage {
                            table_writer.write::<_, _, 2>(1, &prepared_damage);
                        }
                    }

                    table_writer.finish()
                }
            }

            impl ::planus::WriteAs<::planus::Offset<Weapon>> for Weapon {
                type Prepared = ::planus::Offset<Self>;

                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Weapon> {
                    ::planus::WriteAsOffset::prepare(self, builder)
                }
            }

            impl ::planus::WriteAsOptional<::planus::Offset<Weapon>> for Weapon {
                type Prepared = ::planus::Offset<Self>;

                fn prepare(
                    &self,
                    builder: &mut ::planus::Builder,
                ) -> ::core::option::Option<::planus::Offset<Weapon>> {
                    ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
                }
            }

            impl ::planus::WriteAsOffset<Weapon> for Weapon {
                fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Weapon> {
                    Weapon::create(builder, &self.name, &self.damage)
                }
            }

            #[derive(Copy, Clone)]
            pub struct WeaponRef<'a>(::planus::table_reader::Table<'a>);

            impl<'a> WeaponRef<'a> {
                pub fn name(
                    &self,
                ) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>>
                {
                    self.0.access(0, "Weapon", "name")
                }

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
                        name: if let ::core::option::Option::Some(name) = value.name()? {
                            ::core::option::Option::Some(::core::convert::TryInto::try_into(name)?)
                        } else {
                            ::core::option::Option::None
                        },
                        damage: ::core::convert::TryInto::try_into(value.damage()?)?,
                    })
                }
            }

            impl<'a> ::planus::TableRead<'a> for WeaponRef<'a> {
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

            impl ::planus::VectorWrite<::planus::Offset<Weapon>> for Weapon {
                type Value = ::planus::Offset<Weapon>;
                const STRIDE: usize = 4;
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
                            ::planus::Cursor::new(&mut *bytes.add(i)),
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
        }
    }
}
