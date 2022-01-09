use c::Obj as CObj;
use c::ObjRef as CObjRef;

check_type!(CObj => {
    here: Option<Box<c::Obj>>,
    a: Option<Box<a::Obj>>,
    a_a: Option<Box<a::a::Obj>>,
    a_b: Option<Box<a::b::Obj>>,
    b: Option<Box<b::Obj>>,
    b_a: Option<Box<b::a::Obj>>,
    b_b: Option<Box<b::b::Obj>>,
    c: Option<Box<c::Obj>>,
});
check_type!(CObj => create(
    &mut planus::Builder,
    CObj,
    a::Obj,
    a::a::Obj,
    a::b::Obj,
    b::Obj,
    b::a::Obj,
    b::b::Obj,
    c::Obj,
) : planus::Offset<CObj>);
check_type!(+['a] CObjRef<'a> => &self.here() : planus::Result<Option<CObjRef<'a>>>);
check_type!(+['a] CObjRef<'a> => &self.a() : planus::Result<Option<a::ObjRef<'a>>>);
check_type!(+['a] CObjRef<'a> => &self.a_a() : planus::Result<Option<a::a::ObjRef<'a>>>);
check_type!(+['a] CObjRef<'a> => &self.a_b() : planus::Result<Option<a::b::ObjRef<'a>>>);
check_type!(+['a] CObjRef<'a> => &self.b() : planus::Result<Option<b::ObjRef<'a>>>);
check_type!(+['a] CObjRef<'a> => &self.b_a() : planus::Result<Option<b::a::ObjRef<'a>>>);
check_type!(+['a] CObjRef<'a> => &self.b_b() : planus::Result<Option<b::b::ObjRef<'a>>>);
check_type!(+['a] CObjRef<'a> => &self.c() : planus::Result<Option<c::ObjRef<'a>>>);

use a::Obj as AObj;
use a::ObjRef as AObjRef;

check_type!(AObj => {
    here: Option<Box<AObj>>,
    a: Option<Box<a::a::Obj>>,
    a_a: Option<Box<a::a::Obj>>,
    a_b: Option<Box<a::b::Obj>>,
    b: Option<Box<a::b::Obj>>,
    b_a: Option<Box<b::a::Obj>>,
    b_b: Option<Box<b::b::Obj>>,
    c: Option<Box<c::Obj>>,
});
check_type!(AObj => create(
    &mut planus::Builder,
    AObj,
    a::a::Obj,
    a::a::Obj,
    a::b::Obj,
    a::b::Obj,
    b::a::Obj,
    b::b::Obj,
    c::Obj,
) : planus::Offset<AObj>);
check_type!(+['a] AObjRef<'a> => &self.here() : planus::Result<Option<AObjRef<'a>>>);
check_type!(+['a] AObjRef<'a> => &self.a() : planus::Result<Option<a::a::ObjRef<'a>>>);
check_type!(+['a] AObjRef<'a> => &self.a_a() : planus::Result<Option<a::a::ObjRef<'a>>>);
check_type!(+['a] AObjRef<'a> => &self.a_b() : planus::Result<Option<a::b::ObjRef<'a>>>);
check_type!(+['a] AObjRef<'a> => &self.b() : planus::Result<Option<a::b::ObjRef<'a>>>);
check_type!(+['a] AObjRef<'a> => &self.b_a() : planus::Result<Option<b::a::ObjRef<'a>>>);
check_type!(+['a] AObjRef<'a> => &self.b_b() : planus::Result<Option<b::b::ObjRef<'a>>>);
check_type!(+['a] AObjRef<'a> => &self.c() : planus::Result<Option<c::ObjRef<'a>>>);
