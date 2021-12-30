pub mod my_game {
    pub mod sample {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #[repr(i8)]
        pub enum Color {
            Red = 0,
            Green = 1,
            Blue = 2,
        }

        impl TryFrom<i8> for Color {
            type Error = planus::errors::UnknownEnumTagKind;
            fn try_from(
                value: i8,
            ) -> std::result::Result<Self, planus::errors::UnknownEnumTagKind> {
                #[allow(clippy::match_single_binding)]
                match value {
                    0 => Ok(Color::Red),
                    1 => Ok(Color::Green),
                    2 => Ok(Color::Blue),

                    _ => Err(planus::errors::UnknownEnumTagKind { tag: value as i128 }),
                }
            }
        }

        impl From<Color> for i8 {
            fn from(value: Color) -> Self {
                value as i8
            }
        }

        impl planus::ToOwned for Color {
            type Value = Color;
            #[inline]
            fn to_owned(self) -> planus::Result<Self::Value> {
                Ok(self)
            }
        }

        impl planus::Primitive for Color {
            const ALIGNMENT: usize = 1;
            const SIZE: usize = 1;
        }

        impl planus::WriteAsPrimitive<Color> for Color {
            #[inline]
            fn write<const N: usize>(&self, cursor: planus::Cursor<'_, N>, buffer_position: u32) {
                (*self as i8).write(cursor, buffer_position);
            }
        }

        impl planus::WriteAs<Color> for Color {
            type Prepared = Self;

            #[inline]
            fn prepare(&self, _builder: &mut planus::Builder) -> Color {
                *self
            }
        }

        impl planus::WriteAsDefault<Color, Color> for Color {
            type Prepared = Self;

            #[inline]
            fn prepare(&self, _builder: &mut planus::Builder, default: &Color) -> Option<Color> {
                if self == default {
                    None
                } else {
                    Some(*self)
                }
            }
        }

        impl planus::WriteAsOptional<Color> for Color {
            type Prepared = Self;

            #[inline]
            fn prepare(&self, _builder: &mut planus::Builder) -> Option<Color> {
                Some(*self)
            }
        }

        impl<'buf> planus::TableRead<'buf> for Color {
            fn from_buffer(
                buffer: planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> std::result::Result<Self, planus::errors::ErrorKind> {
                let n: i8 = planus::TableRead::from_buffer(buffer, offset)?;
                Ok(n.try_into()?)
            }
        }

        impl<'buf> planus::VectorRead<'buf> for Color {
            type Output = std::result::Result<Self, planus::errors::UnknownEnumTag>;

            const STRIDE: usize = 1;
            #[inline]
            unsafe fn from_buffer(
                buffer: planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> Self::Output {
                let value = <i8 as planus::VectorRead>::from_buffer(buffer, offset);
                let value: std::result::Result<Self, _> = value.try_into();
                value.map_err(|error_kind| {
                    error_kind.with_error_location(
                        "Color",
                        "VectorRead::from_buffer",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        impl<'buf> planus::VectorWrite<Color> for Color {
            const STRIDE: usize = 1;

            type Value = Self;

            fn prepare(&self, _builder: &mut planus::Builder) -> Self::Value {
                *self
            }

            #[inline]
            unsafe fn write_values(
                values: &[Self],
                bytes: *mut std::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [std::mem::MaybeUninit<u8>; 1];
                for (i, v) in values.iter().enumerate() {
                    planus::WriteAsPrimitive::write(
                        v,
                        planus::Cursor::new(&mut *bytes.add(i)),
                        buffer_position - i as u32,
                    );
                }
            }
        }

        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
        pub enum Equipment {
            Weapon(Box<self::Weapon>),
        }

        impl Equipment {
            pub fn create_weapon(
                builder: &mut planus::Builder,
                value: impl planus::WriteAsOffset<self::Weapon>,
            ) -> planus::UnionOffset<Self> {
                planus::UnionOffset::new(1, value.prepare(builder).downcast())
            }
        }

        impl planus::WriteAsUnion<Equipment> for Equipment {
            fn prepare(&self, builder: &mut planus::Builder) -> planus::UnionOffset<Self> {
                match self {
                    Self::Weapon(value) => Self::create_weapon(builder, value),
                }
            }
        }

        impl planus::WriteAsOptionalUnion<Equipment> for Equipment {
            fn prepare(&self, builder: &mut planus::Builder) -> Option<planus::UnionOffset<Self>> {
                Some(planus::WriteAsUnion::prepare(self, builder))
            }
        }

        #[derive(Copy, Clone, Debug)]
        pub enum EquipmentRef<'a> {
            Weapon(self::WeaponRef<'a>),
        }

        impl<'a> planus::ToOwned for EquipmentRef<'a> {
            type Value = Equipment;

            fn to_owned(self) -> planus::Result<Equipment> {
                Ok(match self {
                    Self::Weapon(value) => {
                        Equipment::Weapon(Box::new(planus::ToOwned::to_owned(value)?))
                    }
                })
            }
        }

        impl<'a> planus::TableReadUnion<'a> for EquipmentRef<'a> {
            fn from_buffer(
                buffer: planus::SliceWithStartOffset<'a>,
                field_offset: usize,
                tag: u8,
            ) -> std::result::Result<Self, planus::errors::ErrorKind> {
                match tag {
                    1 => Ok(Self::Weapon(planus::TableRead::from_buffer(
                        buffer,
                        field_offset,
                    )?)),
                    _ => Err(planus::errors::ErrorKind::UnknownUnionTag { tag }),
                }
            }
        }

        #[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        pub struct Vec3 {
            pub x: f32,
            pub y: f32,
            pub z: f32,
        }

        impl planus::Primitive for Vec3 {
            const ALIGNMENT: usize = 4;
            const SIZE: usize = 12;
        }

        #[allow(clippy::identity_op)]
        impl planus::WriteAsPrimitive<Vec3> for Vec3 {
            fn write<const N: usize>(&self, cursor: planus::Cursor<'_, N>, buffer_position: u32) {
                let (cur, cursor) = cursor.split::<4, 8>();
                self.x.write(cur, buffer_position - 0);
                let (cur, cursor) = cursor.split::<4, 4>();
                self.y.write(cur, buffer_position - 4);
                let (cur, cursor) = cursor.split::<4, 0>();
                self.z.write(cur, buffer_position - 8);
                cursor.finish([]);
            }
        }

        impl planus::WriteAs<Vec3> for Vec3 {
            type Prepared = Self;
            fn prepare(&self, _builder: &mut planus::Builder) -> Self {
                *self
            }
        }

        impl planus::WriteAsOptional<Vec3> for Vec3 {
            type Prepared = Self;
            fn prepare(&self, _builder: &mut planus::Builder) -> Option<Self> {
                Some(*self)
            }
        }

        #[derive(Copy, Clone)]
        pub struct Vec3Ref<'a>(planus::ArrayWithStartOffset<'a, 12>);

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

        impl<'a> std::fmt::Debug for Vec3Ref<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut f = f.debug_struct("Vec3Ref");
                f.field("x", &self.x());
                f.field("y", &self.y());
                f.field("z", &self.z());
                f.finish()
            }
        }

        impl<'a> planus::ToOwned for Vec3Ref<'a> {
            type Value = Vec3;
            fn to_owned(self) -> planus::Result<Self::Value> {
                Ok(Vec3 {
                    x: self.x(),
                    y: self.y(),
                    z: self.z(),
                })
            }
        }

        impl<'a> planus::TableRead<'a> for Vec3Ref<'a> {
            fn from_buffer(
                buffer: planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> std::result::Result<Self, planus::errors::ErrorKind> {
                let buffer = buffer.advance_as_array::<12>(offset)?;
                Ok(Self(buffer))
            }
        }

        impl<'a> planus::VectorRead<'a> for Vec3 {
            const STRIDE: usize = 12;

            type Output = Vec3Ref<'a>;
            unsafe fn from_buffer(
                buffer: planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> Vec3Ref<'a> {
                Vec3Ref(buffer.unchecked_advance_as_array(offset))
            }
        }

        impl planus::VectorWrite<Vec3> for Vec3 {
            const STRIDE: usize = 12;

            type Value = Vec3;

            fn prepare(&self, _builder: &mut planus::Builder) -> Self::Value {
                *self
            }

            #[inline]
            unsafe fn write_values(
                values: &[Vec3],
                bytes: *mut std::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [std::mem::MaybeUninit<u8>; 12];
                for (i, v) in values.iter().enumerate() {
                    planus::WriteAsPrimitive::write(
                        v,
                        planus::Cursor::new(&mut *bytes.add(i)),
                        buffer_position - (12 * i) as u32,
                    );
                }
            }
        }

        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
        pub struct Monster {
            pub pos: Option<self::Vec3>,
            pub mana: i16,
            pub hp: i16,
            pub name: Option<String>,
            pub friendly: bool,
            pub inventory: Option<Vec<u8>>,
            pub color: self::Color,
            pub weapons: Option<Vec<self::Weapon>>,
            pub equipped: Option<self::Equipment>,
            pub path: Option<Vec<self::Vec3>>,
        }

        impl Monster {
            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut planus::Builder,
                pos: impl planus::WriteAsOptional<self::Vec3>,
                mana: impl planus::WriteAsDefault<i16, i16>,
                hp: impl planus::WriteAsDefault<i16, i16>,
                name: impl planus::WriteAsOptional<planus::Offset<str>>,
                friendly: impl planus::WriteAsDefault<bool, bool>,
                inventory: impl planus::WriteAsOptional<planus::Offset<[u8]>>,
                color: impl planus::WriteAsDefault<self::Color, self::Color>,
                weapons: impl planus::WriteAsOptional<planus::Offset<[planus::Offset<self::Weapon>]>>,
                equipped: impl planus::WriteAsOptionalUnion<self::Equipment>,
                path: impl planus::WriteAsOptional<planus::Offset<[self::Vec3]>>,
            ) -> planus::Offset<Self> {
                let prepared_pos = pos.prepare(builder);

                let prepared_mana = mana.prepare(builder, &150);

                let prepared_hp = hp.prepare(builder, &100);

                let prepared_name = name.prepare(builder);

                let prepared_friendly = friendly.prepare(builder, &false);

                let prepared_inventory = inventory.prepare(builder);

                let prepared_color = color.prepare(builder, &self::Color::Blue);

                let prepared_weapons = weapons.prepare(builder);

                let prepared_equipped = equipped.prepare(builder);

                let prepared_path = path.prepare(builder);

                let mut table_writer = planus::table_writer::TableWriter::<24, 39>::new(builder);

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
                    table_writer.calculate_size::<planus::Offset<str>>(8);
                }
                if prepared_friendly.is_some() {
                    table_writer.calculate_size::<bool>(10);
                }
                if prepared_inventory.is_some() {
                    table_writer.calculate_size::<planus::Offset<[u8]>>(12);
                }
                if prepared_color.is_some() {
                    table_writer.calculate_size::<self::Color>(14);
                }
                if prepared_weapons.is_some() {
                    table_writer
                        .calculate_size::<planus::Offset<[planus::Offset<self::Weapon>]>>(16);
                }
                if prepared_equipped.is_some() {
                    table_writer.calculate_size::<u8>(18);
                    table_writer.calculate_size::<planus::Offset<self::Equipment>>(20);
                }
                if prepared_path.is_some() {
                    table_writer.calculate_size::<planus::Offset<[self::Vec3]>>(22);
                }

                table_writer.finish_calculating();

                unsafe {
                    if let Some(prepared_pos) = prepared_pos {
                        table_writer.write::<_, _, 12>(0, &prepared_pos);
                    }
                    if let Some(prepared_name) = prepared_name {
                        table_writer.write::<_, _, 4>(3, &prepared_name);
                    }
                    if let Some(prepared_inventory) = prepared_inventory {
                        table_writer.write::<_, _, 4>(5, &prepared_inventory);
                    }
                    if let Some(prepared_weapons) = prepared_weapons {
                        table_writer.write::<_, _, 4>(7, &prepared_weapons);
                    }
                    if let Some(prepared_equipped) = prepared_equipped {
                        table_writer.write::<_, _, 4>(9, &prepared_equipped.offset);
                    }
                    if let Some(prepared_path) = prepared_path {
                        table_writer.write::<_, _, 4>(10, &prepared_path);
                    }
                    if let Some(prepared_mana) = prepared_mana {
                        table_writer.write::<_, _, 2>(1, &prepared_mana);
                    }
                    if let Some(prepared_hp) = prepared_hp {
                        table_writer.write::<_, _, 2>(2, &prepared_hp);
                    }
                    if let Some(prepared_friendly) = prepared_friendly {
                        table_writer.write::<_, _, 1>(4, &prepared_friendly);
                    }
                    if let Some(prepared_color) = prepared_color {
                        table_writer.write::<_, _, 1>(6, &prepared_color);
                    }
                    if let Some(prepared_equipped) = prepared_equipped {
                        table_writer.write::<_, _, 1>(8, &prepared_equipped.tag);
                    }
                }

                table_writer.finish()
            }
        }

        impl planus::WriteAs<planus::Offset<Monster>> for Monster {
            type Prepared = planus::Offset<Self>;

            fn prepare(&self, builder: &mut planus::Builder) -> planus::Offset<Monster> {
                planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl planus::WriteAsOptional<planus::Offset<Monster>> for Monster {
            type Prepared = planus::Offset<Self>;

            fn prepare(&self, builder: &mut planus::Builder) -> Option<planus::Offset<Monster>> {
                Some(planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl planus::WriteAsOffset<Monster> for Monster {
            fn prepare(&self, builder: &mut planus::Builder) -> planus::Offset<Monster> {
                Monster::create(
                    builder,
                    &self.pos,
                    &self.mana,
                    &self.hp,
                    &self.name,
                    &self.friendly,
                    &self.inventory,
                    &self.color,
                    &self.weapons,
                    &self.equipped,
                    &self.path,
                )
            }
        }

        #[derive(Copy, Clone)]
        pub struct MonsterRef<'a>(planus::table_reader::Table<'a>);

        impl<'a> MonsterRef<'a> {
            pub fn pos(&self) -> planus::Result<Option<self::Vec3Ref<'a>>> {
                self.0.access(0, "Monster", "pos")
            }

            pub fn mana(&self) -> planus::Result<i16> {
                Ok(self.0.access(1, "Monster", "mana")?.unwrap_or(150))
            }

            pub fn hp(&self) -> planus::Result<i16> {
                Ok(self.0.access(2, "Monster", "hp")?.unwrap_or(100))
            }

            pub fn name(&self) -> planus::Result<Option<&'a str>> {
                self.0.access(3, "Monster", "name")
            }

            pub fn friendly(&self) -> planus::Result<bool> {
                Ok(self.0.access(4, "Monster", "friendly")?.unwrap_or(false))
            }

            pub fn inventory(&self) -> planus::Result<Option<planus::Vector<'a, u8>>> {
                self.0.access(5, "Monster", "inventory")
            }

            pub fn color(&self) -> planus::Result<self::Color> {
                Ok(self
                    .0
                    .access(6, "Monster", "color")?
                    .unwrap_or(self::Color::Blue))
            }

            pub fn weapons(&self) -> planus::Result<Option<planus::Vector<'a, self::Weapon>>> {
                self.0.access(7, "Monster", "weapons")
            }

            pub fn equipped(&self) -> planus::Result<Option<self::EquipmentRef<'a>>> {
                self.0.access_union(8, "Monster", "equipped")
            }

            pub fn path(&self) -> planus::Result<Option<planus::Vector<'a, self::Vec3>>> {
                self.0.access(10, "Monster", "path")
            }
        }

        impl<'a> std::fmt::Debug for MonsterRef<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut f = f.debug_struct("MonsterRef");
                if let Ok(Some(pos)) = self.pos() {
                    f.field("pos", &pos);
                }
                if let Ok(mana) = self.mana() {
                    f.field("mana", &mana);
                }
                if let Ok(hp) = self.hp() {
                    f.field("hp", &hp);
                }
                if let Ok(Some(name)) = self.name() {
                    f.field("name", &name);
                }
                if let Ok(friendly) = self.friendly() {
                    f.field("friendly", &friendly);
                }
                if let Ok(Some(inventory)) = self.inventory() {
                    f.field("inventory", &inventory);
                }
                if let Ok(color) = self.color() {
                    f.field("color", &color);
                }
                if let Ok(Some(weapons)) = self.weapons() {
                    f.field("weapons", &weapons);
                }
                if let Ok(Some(equipped)) = self.equipped() {
                    f.field("equipped", &equipped);
                }
                if let Ok(Some(path)) = self.path() {
                    f.field("path", &path);
                }
                f.finish()
            }
        }

        impl<'a> planus::ToOwned for MonsterRef<'a> {
            type Value = Monster;

            fn to_owned(self) -> planus::Result<Self::Value> {
                Ok(Monster {
                    pos: if let Some(pos) = self.pos()? {
                        Some(planus::ToOwned::to_owned(pos)?)
                    } else {
                        None
                    },
                    mana: planus::ToOwned::to_owned(self.mana()?)?,
                    hp: planus::ToOwned::to_owned(self.hp()?)?,
                    name: if let Some(name) = self.name()? {
                        Some(planus::ToOwned::to_owned(name)?)
                    } else {
                        None
                    },
                    friendly: planus::ToOwned::to_owned(self.friendly()?)?,
                    inventory: if let Some(inventory) = self.inventory()? {
                        Some(planus::ToOwned::to_owned(inventory)?)
                    } else {
                        None
                    },
                    color: planus::ToOwned::to_owned(self.color()?)?,
                    weapons: if let Some(weapons) = self.weapons()? {
                        Some(planus::ToOwned::to_owned(weapons)?)
                    } else {
                        None
                    },
                    equipped: if let Some(equipped) = self.equipped()? {
                        Some(planus::ToOwned::to_owned(equipped)?)
                    } else {
                        None
                    },
                    path: if let Some(path) = self.path()? {
                        Some(planus::ToOwned::to_owned(path)?)
                    } else {
                        None
                    },
                })
            }
        }

        impl<'a> planus::TableRead<'a> for MonsterRef<'a> {
            fn from_buffer(
                buffer: planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> std::result::Result<Self, planus::errors::ErrorKind> {
                Ok(Self(planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> planus::VectorRead<'a> for Monster {
            type Output = planus::Result<MonsterRef<'a>>;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> Self::Output {
                planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location("[MonsterRef]", "get", buffer.offset_from_start)
                })
            }
        }

        impl planus::VectorWrite<planus::Offset<Monster>> for Monster {
            type Value = planus::Offset<Monster>;
            const STRIDE: usize = 4;
            fn prepare(&self, builder: &mut planus::Builder) -> Self::Value {
                planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[planus::Offset<Monster>],
                bytes: *mut std::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [std::mem::MaybeUninit<u8>; 4];
                for (i, v) in values.iter().enumerate() {
                    planus::WriteAsPrimitive::write(
                        v,
                        planus::Cursor::new(&mut *bytes.add(i)),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> planus::ReadAsRoot<'a> for MonsterRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> planus::Result<Self> {
                planus::TableRead::from_buffer(
                    planus::SliceWithStartOffset {
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

        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
        pub struct Weapon {
            pub name: Option<String>,
            pub damage: i16,
        }

        impl Weapon {
            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut planus::Builder,
                name: impl planus::WriteAsOptional<planus::Offset<str>>,
                damage: impl planus::WriteAsDefault<i16, i16>,
            ) -> planus::Offset<Self> {
                let prepared_name = name.prepare(builder);

                let prepared_damage = damage.prepare(builder, &0);

                let mut table_writer = planus::table_writer::TableWriter::<6, 6>::new(builder);

                if prepared_name.is_some() {
                    table_writer.calculate_size::<planus::Offset<str>>(2);
                }
                if prepared_damage.is_some() {
                    table_writer.calculate_size::<i16>(4);
                }

                table_writer.finish_calculating();

                unsafe {
                    if let Some(prepared_name) = prepared_name {
                        table_writer.write::<_, _, 4>(0, &prepared_name);
                    }
                    if let Some(prepared_damage) = prepared_damage {
                        table_writer.write::<_, _, 2>(1, &prepared_damage);
                    }
                }

                table_writer.finish()
            }
        }

        impl planus::WriteAs<planus::Offset<Weapon>> for Weapon {
            type Prepared = planus::Offset<Self>;

            fn prepare(&self, builder: &mut planus::Builder) -> planus::Offset<Weapon> {
                planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl planus::WriteAsOptional<planus::Offset<Weapon>> for Weapon {
            type Prepared = planus::Offset<Self>;

            fn prepare(&self, builder: &mut planus::Builder) -> Option<planus::Offset<Weapon>> {
                Some(planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl planus::WriteAsOffset<Weapon> for Weapon {
            fn prepare(&self, builder: &mut planus::Builder) -> planus::Offset<Weapon> {
                Weapon::create(builder, &self.name, &self.damage)
            }
        }

        #[derive(Copy, Clone)]
        pub struct WeaponRef<'a>(planus::table_reader::Table<'a>);

        impl<'a> WeaponRef<'a> {
            pub fn name(&self) -> planus::Result<Option<&'a str>> {
                self.0.access(0, "Weapon", "name")
            }

            pub fn damage(&self) -> planus::Result<i16> {
                Ok(self.0.access(1, "Weapon", "damage")?.unwrap_or(0))
            }
        }

        impl<'a> std::fmt::Debug for WeaponRef<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut f = f.debug_struct("WeaponRef");
                if let Ok(Some(name)) = self.name() {
                    f.field("name", &name);
                }
                if let Ok(damage) = self.damage() {
                    f.field("damage", &damage);
                }
                f.finish()
            }
        }

        impl<'a> planus::ToOwned for WeaponRef<'a> {
            type Value = Weapon;

            fn to_owned(self) -> planus::Result<Self::Value> {
                Ok(Weapon {
                    name: if let Some(name) = self.name()? {
                        Some(planus::ToOwned::to_owned(name)?)
                    } else {
                        None
                    },
                    damage: planus::ToOwned::to_owned(self.damage()?)?,
                })
            }
        }

        impl<'a> planus::TableRead<'a> for WeaponRef<'a> {
            fn from_buffer(
                buffer: planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> std::result::Result<Self, planus::errors::ErrorKind> {
                Ok(Self(planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> planus::VectorRead<'a> for Weapon {
            type Output = planus::Result<WeaponRef<'a>>;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> Self::Output {
                planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location("[WeaponRef]", "get", buffer.offset_from_start)
                })
            }
        }

        impl planus::VectorWrite<planus::Offset<Weapon>> for Weapon {
            type Value = planus::Offset<Weapon>;
            const STRIDE: usize = 4;
            fn prepare(&self, builder: &mut planus::Builder) -> Self::Value {
                planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[planus::Offset<Weapon>],
                bytes: *mut std::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [std::mem::MaybeUninit<u8>; 4];
                for (i, v) in values.iter().enumerate() {
                    planus::WriteAsPrimitive::write(
                        v,
                        planus::Cursor::new(&mut *bytes.add(i)),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> planus::ReadAsRoot<'a> for WeaponRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> planus::Result<Self> {
                planus::TableRead::from_buffer(
                    planus::SliceWithStartOffset {
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
