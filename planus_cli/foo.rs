pub mod evolution {
    pub mod v1 {
        #[derive(Clone, Debug)]
        pub struct TableA {
            pub a: f32,
            pub b: i32,
        }

        impl TableA {
            pub fn create(
                buffer: &mut planus::Buffer,
                a: impl WriteAs<f32>,
                b: impl WriteAs<i32>,
            ) -> planus::Offset<Self> {
                let prepared_a = a.prepare(buffer);
                let prepared_b = b.prepare(buffer);

                let mut table_writer = planus::table_writer::TableWriter::<6, 8>::new(buffer);

                if prepared_a.is_some() {
                    table_writer.calculate_size::<f32>(2);
                }

                if prepared_b.is_some() {
                    table_writer.calculate_size::<i32>(4);
                }

                unsafe {
                    if let Some(prepared_a) = prepared_a {
                        table_writer.write(1, &prepared_a);
                    }

                    if let Some(prepared_b) = prepared_b {
                        table_writer.write(2, &prepared_b);
                    }
                }

                table_writer.finish_calculating();
            }
        }

        impl planus::WriteAs<planus::Offset<TableA>> for TableA {
            fn prepare(&self, buffer: &mut planus::Buffer) -> planus::Offset<TableA> {
                TableA::create(buffer, &self.a, &self.b)
            }
        }

        impl planus::WriteAsOptional<planus::Offset<TableA>> for TableA {
            fn prepare(&self, buffer: &mut planus::Buffer) -> Option<planus::Offset<TableA>> {
                Some(planus::WriteAs::prepare(self, buffer))
            }
        }

        #[derive(Copy, Clone)]
        pub struct TableARef<'buf>(planus::table_reader::Table<'buf>);

        impl<'buf> TableARef<'buf> {
            pub fn a(&self) -> planus::Result<f32> {
                Ok(self.0.access(0, "TableA", "a")?.unwrap_or(0))
            }

            pub fn b(&self) -> planus::Result<i32> {
                Ok(self.0.access(1, "TableA", "b")?.unwrap_or(0))
            }
        }

        impl<'buf> std::fmt::Debug for TableARef {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut f = f.debug_struct("TableARef");
                if let Ok(a) = self.a() {
                    f.field("a", &a);
                }
                if let Ok(b) = self.b() {
                    f.field("b", &b);
                }
                f.finish()
            }
        }

        impl<'buf> planus::ToOwned for TableARef<'buf> {
            type Value = TableA;

            fn to_owned(&self) -> planus::Result<Self::Value> {
                Ok(TableA {
                    a: planus::ToOwned::to_owned(self.a()?)?,
                    b: planus::ToOwned::to_owned(self.b()?)?,
                })
            }
        }

        impl<'buf> planus::TableRead<'buf> for TableARef<'buf> {
            fn from_buffer(
                buffer: planus::BufferWithStartOffset<'buf>,
                offset: usize,
            ) -> Result<Self, planus::errors::ErrorKind> {
                Ok(Self(planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'buf> planus::VectorRead<'buf> for TableA {
            type Output = planus::Result<TableARef<'buf>>;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: planus::BufferWithStartOffset<'buf>,
                offset: usize,
            ) -> Self::Output {
                planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| planus::Error {
                    source_location: planus::errors::ErrorLocation {
                        type_: "[TableARef]".into(),
                        method: "get",
                        byte_offset: usize::MAX,
                    },
                    error_kind,
                })
            }
        }

        impl planus::VectorWrite<planus::Offset<TableA>> for TableA {
            type Value = planus::Offset<TableA>;
            const STRIDE: usize = 4;
            fn prepare(&self, buffer: &mut planus::Buffer) -> Self::Value {
                planus::WriteAs::prepare(self, buffer)
            }
        }

        #[derive(Clone, Debug)]
        pub struct TableB {
            pub a: i32,
        }

        impl TableB {
            pub fn create(
                buffer: &mut planus::Buffer,
                a: impl WriteAs<i32>,
            ) -> planus::Offset<Self> {
                let prepared_a = a.prepare(buffer);

                let mut table_writer = planus::table_writer::TableWriter::<4, 4>::new(buffer);

                if prepared_a.is_some() {
                    table_writer.calculate_size::<i32>(2);
                }

                unsafe {
                    if let Some(prepared_a) = prepared_a {
                        table_writer.write(1, &prepared_a);
                    }
                }

                table_writer.finish_calculating();
            }
        }

        impl planus::WriteAs<planus::Offset<TableB>> for TableB {
            fn prepare(&self, buffer: &mut planus::Buffer) -> planus::Offset<TableB> {
                TableB::create(buffer, &self.a)
            }
        }

        impl planus::WriteAsOptional<planus::Offset<TableB>> for TableB {
            fn prepare(&self, buffer: &mut planus::Buffer) -> Option<planus::Offset<TableB>> {
                Some(planus::WriteAs::prepare(self, buffer))
            }
        }

        #[derive(Copy, Clone)]
        pub struct TableBRef<'buf>(planus::table_reader::Table<'buf>);

        impl<'buf> TableBRef<'buf> {
            pub fn a(&self) -> planus::Result<i32> {
                Ok(self.0.access(0, "TableB", "a")?.unwrap_or(0))
            }
        }

        impl<'buf> std::fmt::Debug for TableBRef {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut f = f.debug_struct("TableBRef");
                if let Ok(a) = self.a() {
                    f.field("a", &a);
                }
                f.finish()
            }
        }

        impl<'buf> planus::ToOwned for TableBRef<'buf> {
            type Value = TableB;

            fn to_owned(&self) -> planus::Result<Self::Value> {
                Ok(TableB {
                    a: planus::ToOwned::to_owned(self.a()?)?,
                })
            }
        }

        impl<'buf> planus::TableRead<'buf> for TableBRef<'buf> {
            fn from_buffer(
                buffer: planus::BufferWithStartOffset<'buf>,
                offset: usize,
            ) -> Result<Self, planus::errors::ErrorKind> {
                Ok(Self(planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'buf> planus::VectorRead<'buf> for TableB {
            type Output = planus::Result<TableBRef<'buf>>;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: planus::BufferWithStartOffset<'buf>,
                offset: usize,
            ) -> Self::Output {
                planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| planus::Error {
                    source_location: planus::errors::ErrorLocation {
                        type_: "[TableBRef]".into(),
                        method: "get",
                        byte_offset: usize::MAX,
                    },
                    error_kind,
                })
            }
        }

        impl planus::VectorWrite<planus::Offset<TableB>> for TableB {
            type Value = planus::Offset<TableB>;
            const STRIDE: usize = 4;
            fn prepare(&self, buffer: &mut planus::Buffer) -> Self::Value {
                planus::WriteAs::prepare(self, buffer)
            }
        }

        #[derive(Copy, Clone, Debug)]
        #[repr(i8)]
        pub enum Enum {
            Self_ = 0,
            Queen = 1,
        }

        impl planus::ToOwned for Enum {
            type Value = Enum;
            #[inline]
            fn to_owned(&self) -> planus::Result<Self::Value> {
                Ok(*self)
            }
        }

        impl planus::Primitive for Enum {
            const ALIGNMENT: usize = 1;
            const SIZE: usize = 1;
        }

        unsafe impl planus::WriteAsPrimitive<Enum> for Enum {
            #[inline]
            unsafe fn write(&self, buffer: *mut u8, buffer_position: u32) {
                (*self as i8).write(buffer, buffer_position);
            }
        }

        impl planus::WriteAs<Enum> for Enum {
            #[inline]
            fn prepare(&self, _buffer: &mut planus::Buffer) -> Enum {
                *self
            }
        }

        impl planus::WriteAsOptional<Enum> for Enum {
            #[inline]
            fn prepare(&self, _buffer: &mut planus::Buffer) -> Option<Enum> {
                Some(*self)
            }
        }

        impl<'buf> planus::TableRead<'buf> for Enum {
            fn from_buffer(
                buffer: planus::BufferWithStartOffset<'buf>,
                offset: usize,
            ) -> Result<Self, planus::errors::ErrorKind> {
                let n: i8 = planus::TableRead::from_buffer(buffer, offset)?;
                match n {
                    0 => Ok(Enum::Self_),
                    1 => Ok(Enum::Queen),

                    _ => Err(planus::errors::ErrorKind::UnknownEnumTag { tag: n as i128 }),
                }
            }
        }

        #[derive(Copy, Clone, Debug)]
        pub struct Struct {
            pub a: i32,
            pub b: f64,
            pub c: Enum,
        }

        impl planus::Primitive for Struct {
            const ALIGNMENT: usize = 8;
            const SIZE: usize = 24;
        }

        unsafe impl planus::WriteAsPrimitive<Struct> for Struct {
            unsafe fn write(&self, buffer: *mut u8, buffer_position: u32) {
                self.a.write(buffer.add(0), buffer_position - 0);
                self.b.write(buffer.add(8), buffer_position - 8);
                self.c.write(buffer.add(16), buffer_position - 16);
            }
        }

        impl<'a> planus::WriteAs<'a, Struct> for Struct {
            type Prepared = &'a Self;
            fn prepare(&'a self, _buffer: &mut planus::Buffer) -> &'a Self {
                self
            }
        }

        impl<'a> planus::WriteAsOptional<'a, Struct> for Struct {
            type Prepared = &'a Self;
            fn prepare(&'a self, _buffer: &mut planus::Buffer) -> Option<&'a Self> {
                Some(self)
            }
        }

        #[derive(Copy, Clone)]
        pub struct StructRef<'buf>(&'buf [u8; 24]);

        impl<'buf> StructRef<'buf> {
            pub fn a(&self) -> i32 {
                let buffer: &[u8; 4] = std::convert::TryInto::try_into(&self.0[0..4]).unwrap();
                i32::from_le_bytes(*buffer)
            }

            pub fn b(&self) -> f64 {
                let buffer: &[u8; 8] = std::convert::TryInto::try_into(&self.0[8..16]).unwrap();
                f64::from_le_bytes(*buffer)
            }

            pub fn c(&self) -> planus::Result<Enum> {
                let buffer: &[u8; 1] = std::convert::TryInto::try_into(&self.0[16..17]).unwrap();
                i8::from_le_bytes(buffer).try_into()
            }
        }

        impl<'buf> std::fmt::Debug for StructRef {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut f = f.debug_struct("StructRef");
                f.field("a", &self.a());
                f.field("b", &self.b());
                if let Ok(value) = self.c() {
                    f.field("c", value);
                }
                f.finish()
            }
        }

        impl<'buf> planus::ToOwned for StructRef {
            type Value = Struct;
            fn to_owned(&self) -> planus::Result<Self::Value> {
                Ok(Struct {
                    a: self.a(),
                    b: self.b(),
                    c: self.c()?,
                })
            }
        }

        impl<'buf> planus::TableRead<'buf> for StructRef {
            fn from_buffer(
                buffer: planus::BufferWithStartOffset<'buf>,
                offset: usize,
            ) -> Result<Self, planus::errors::ErrorKind> {
                let buffer = buffer.advance_as_array::<24>(offset)?;
                Ok(Self(buffer))
            }
        }

        #[derive(Clone, Debug)]
        pub struct Root {
            pub a: i32,
            pub b: bool,
            pub c: Option<Union>,
            pub d: Enum,
            pub e: Option<Box<TableA>>,
            pub f: Option<Struct>,
            pub i: i32,
            pub j: Union,
            pub k: Enum,
            pub l: Enum,
            pub m: Option<Enum>,
            pub n: Option<Union>,
        }

        impl Root {
            pub fn create(
                buffer: &mut planus::Buffer,
                a: impl WriteAs<i32>,
                b: impl WriteAs<bool>,
                c: impl WriteAsOptionalUnion<Union>,
                d: impl WriteAs<Enum>,
                e: impl WriteAsOptional<Offset<TableA>>,
                f: impl WriteAsOptional<Struct>,
                i: impl WriteAs<i32>,
                j: impl WriteAsUnion<Union>,
                k: impl WriteAs<Enum>,
                l: impl WriteAs<Enum>,
                m: impl WriteAsOptional<Enum>,
                n: impl WriteAsOptionalUnion<Union>,
            ) -> planus::Offset<Self> {
                let prepared_a = a.prepare(buffer);
                let prepared_b = b.prepare(buffer);
                let prepared_c = c.prepare(buffer);
                let prepared_d = d.prepare(buffer);
                let prepared_e = e.prepare(buffer);
                let prepared_f = f.prepare(buffer);
                let prepared_i = i.prepare(buffer);
                let prepared_j = j.prepare(buffer);
                let prepared_k = k.prepare(buffer);
                let prepared_l = l.prepare(buffer);
                let prepared_m = m.prepare(buffer);
                let prepared_n = n.prepare(buffer);

                let mut table_writer = planus::table_writer::TableWriter::<30, 56>::new(buffer);

                if prepared_a.is_some() {
                    table_writer.calculate_size::<i32>(2);
                }

                if prepared_b.is_some() {
                    table_writer.calculate_size::<bool>(4);
                }

                if prepared_c.is_some() {
                    table_writer.calculate_size::<u8>(6);
                    table_writer.calculate_size::<Offset<Union>>(8);
                }

                if prepared_d.is_some() {
                    table_writer.calculate_size::<Enum>(10);
                }

                if prepared_e.is_some() {
                    table_writer.calculate_size::<Offset<TableA>>(12);
                }

                if prepared_f.is_some() {
                    table_writer.calculate_size::<Struct>(14);
                }

                if prepared_i.is_some() {
                    table_writer.calculate_size::<i32>(16);
                }

                if prepared_j.is_some() {
                    table_writer.calculate_size::<u8>(18);
                    table_writer.calculate_size::<Offset<Union>>(20);
                }

                if prepared_k.is_some() {
                    table_writer.calculate_size::<Enum>(22);
                }

                if prepared_l.is_some() {
                    table_writer.calculate_size::<Enum>(24);
                }

                if prepared_m.is_some() {
                    table_writer.calculate_size::<Enum>(26);
                }

                if prepared_n.is_some() {
                    table_writer.calculate_size::<u8>(28);
                    table_writer.calculate_size::<Offset<Union>>(30);
                }

                unsafe {
                    if let Some(prepared_f) = prepared_f {
                        table_writer.write(7, &prepared_f);
                    }

                    if let Some(prepared_a) = prepared_a {
                        table_writer.write(1, &prepared_a);
                    }

                    if let Some(prepared_c) = prepared_c {
                        table_writer.write(4, &prepared_c.offset);
                    }

                    if let Some(prepared_e) = prepared_e {
                        table_writer.write(6, &prepared_e);
                    }

                    if let Some(prepared_i) = prepared_i {
                        table_writer.write(8, &prepared_i);
                    }

                    if let Some(prepared_j) = prepared_j {
                        table_writer.write(10, &prepared_j.offset);
                    }

                    if let Some(prepared_n) = prepared_n {
                        table_writer.write(15, &prepared_n.offset);
                    }

                    if let Some(prepared_b) = prepared_b {
                        table_writer.write(2, &prepared_b);
                    }

                    if let Some(prepared_c) = prepared_c {
                        table_writer.write(3, &prepared_c.tag);
                    }

                    if let Some(prepared_d) = prepared_d {
                        table_writer.write(5, &prepared_d);
                    }

                    if let Some(prepared_j) = prepared_j {
                        table_writer.write(9, &prepared_j.tag);
                    }

                    if let Some(prepared_k) = prepared_k {
                        table_writer.write(11, &prepared_k);
                    }

                    if let Some(prepared_l) = prepared_l {
                        table_writer.write(12, &prepared_l);
                    }

                    if let Some(prepared_m) = prepared_m {
                        table_writer.write(13, &prepared_m);
                    }

                    if let Some(prepared_n) = prepared_n {
                        table_writer.write(14, &prepared_n.tag);
                    }
                }

                table_writer.finish_calculating();
            }
        }

        impl planus::WriteAs<planus::Offset<Root>> for Root {
            fn prepare(&self, buffer: &mut planus::Buffer) -> planus::Offset<Root> {
                Root::create(
                    buffer, &self.a, &self.b, &self.c, &self.d, &self.e, &self.f, &self.i, &self.j,
                    &self.k, &self.l, &self.m, &self.n,
                )
            }
        }

        impl planus::WriteAsOptional<planus::Offset<Root>> for Root {
            fn prepare(&self, buffer: &mut planus::Buffer) -> Option<planus::Offset<Root>> {
                Some(planus::WriteAs::prepare(self, buffer))
            }
        }

        #[derive(Copy, Clone)]
        pub struct RootRef<'buf>(planus::table_reader::Table<'buf>);

        impl<'buf> RootRef<'buf> {
            pub fn a(&self) -> planus::Result<i32> {
                Ok(self.0.access(0, "Root", "a")?.unwrap_or(0))
            }

            pub fn b(&self) -> planus::Result<bool> {
                Ok(self.0.access(1, "Root", "b")?.unwrap_or(false))
            }

            pub fn c(&self) -> planus::Result<Option<UnionRef<'a>>> {
                self.0.access_union(2, "Root", "c")
            }

            pub fn d(&self) -> planus::Result<Enum> {
                Ok(self.0.access(4, "Root", "d")?.unwrap_or(Enum::Self_))
            }

            pub fn e(&self) -> planus::Result<Option<TableARef<'a>>> {
                self.0.access(5, "Root", "e")
            }

            pub fn f(&self) -> planus::Result<StructRef<'a>> {
                self.0.access(6, "Root", "f")
            }

            pub fn i(&self) -> planus::Result<i32> {
                Ok(self.0.access(7, "Root", "i")?.unwrap_or(1234))
            }

            pub fn j(&self) -> planus::Result<UnionRef<'a>> {
                self.0.access_union(8, "Root", "j")
            }

            pub fn k(&self) -> planus::Result<Enum> {
                Ok(self.0.access(10, "Root", "k")?.unwrap_or(Enum::Self_))
            }

            pub fn l(&self) -> planus::Result<Enum> {
                Ok(self.0.access(11, "Root", "l")?.unwrap_or(Enum::Queen))
            }

            pub fn m(&self) -> planus::Result<Option<Enum>> {
                self.0.access(12, "Root", "m")
            }

            pub fn n(&self) -> planus::Result<Option<UnionRef<'a>>> {
                self.0.access_union(13, "Root", "n")
            }
        }

        impl<'buf> std::fmt::Debug for RootRef {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut f = f.debug_struct("RootRef");
                if let Ok(a) = self.a() {
                    f.field("a", &a);
                }
                if let Ok(b) = self.b() {
                    f.field("b", &b);
                }
                if let Ok(Some(c)) = self.c() {
                    f.field("c", &c);
                }
                if let Ok(d) = self.d() {
                    f.field("d", &d);
                }
                if let Ok(Some(e)) = self.e() {
                    f.field("e", &e);
                }
                if let Ok(f) = self.f() {
                    f.field("f", &f);
                }
                if let Ok(i) = self.i() {
                    f.field("i", &i);
                }
                if let Ok(j) = self.j() {
                    f.field("j", &j);
                }
                if let Ok(k) = self.k() {
                    f.field("k", &k);
                }
                if let Ok(l) = self.l() {
                    f.field("l", &l);
                }
                if let Ok(Some(m)) = self.m() {
                    f.field("m", &m);
                }
                if let Ok(Some(n)) = self.n() {
                    f.field("n", &n);
                }
                f.finish()
            }
        }

        impl<'buf> planus::ToOwned for RootRef<'buf> {
            type Value = Root;

            fn to_owned(&self) -> planus::Result<Self::Value> {
                Ok(Root {
                    a: planus::ToOwned::to_owned(self.a()?)?,
                    b: planus::ToOwned::to_owned(self.b()?)?,
                    c: if let Some(c) = self.c()? {
                        Some(planus::ToOwned::to_owned(c)?)
                    } else {
                        None
                    },
                    d: planus::ToOwned::to_owned(self.d()?)?,
                    e: if let Some(e) = self.e()? {
                        Some(Box::new(planus::ToOwned::to_owned(e)?))
                    } else {
                        None
                    },
                    f: if let Some(f) = self.f()? {
                        Some(planus::ToOwned::to_owned(f)?)
                    } else {
                        None
                    },
                    i: planus::ToOwned::to_owned(self.i()?)?,
                    j: planus::ToOwned::to_owned(self.j()?)?,
                    k: planus::ToOwned::to_owned(self.k()?)?,
                    l: planus::ToOwned::to_owned(self.l()?)?,
                    m: if let Some(m) = self.m()? {
                        Some(planus::ToOwned::to_owned(m)?)
                    } else {
                        None
                    },
                    n: if let Some(n) = self.n()? {
                        Some(planus::ToOwned::to_owned(n)?)
                    } else {
                        None
                    },
                })
            }
        }

        impl<'buf> planus::TableRead<'buf> for RootRef<'buf> {
            fn from_buffer(
                buffer: planus::BufferWithStartOffset<'buf>,
                offset: usize,
            ) -> Result<Self, planus::errors::ErrorKind> {
                Ok(Self(planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'buf> planus::VectorRead<'buf> for Root {
            type Output = planus::Result<RootRef<'buf>>;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: planus::BufferWithStartOffset<'buf>,
                offset: usize,
            ) -> Self::Output {
                planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| planus::Error {
                    source_location: planus::errors::ErrorLocation {
                        type_: "[RootRef]".into(),
                        method: "get",
                        byte_offset: usize::MAX,
                    },
                    error_kind,
                })
            }
        }

        impl planus::VectorWrite<planus::Offset<Root>> for Root {
            type Value = planus::Offset<Root>;
            const STRIDE: usize = 4;
            fn prepare(&self, buffer: &mut planus::Buffer) -> Self::Value {
                planus::WriteAs::prepare(self, buffer)
            }
        }
    }
}
