check_type!(Example32 => value : f32);
check_type!(Example32 => value_null : Option<f32>);
check_type!(Example32 => value_default_zero : f32);
check_type!(Example32 => value_default_one : f32);
check_type!(Example32 => create(&mut planus::Builder, f32, f32, f32, f32) : planus::Offset<Example32>);
check_type!(Example32 => create(&mut planus::Builder, f32, (), f32, f32) : planus::Offset<Example32>);
check_type!(Example32 => create(&mut planus::Builder, f32, Option<f32>, f32, f32) : planus::Offset<Example32>);

check_type!(+['a] Example32Ref<'a> => &self.value() : planus::Result<f32>);
check_type!(+['a] Example32Ref<'a> => &self.value_null() : planus::Result<Option<f32>>);
check_type!(+['a] Example32Ref<'a> => &self.value_default_zero() : planus::Result<f32>);
check_type!(+['a] Example32Ref<'a> => &self.value_default_one() : planus::Result<f32>);
check_type!(+['a] Example32Ref<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example32: !Copy + Clone + Debug + !Eq + !Ord + !Hash + Default,
    Example32Ref<'_>: Copy + Clone + Debug + !Eq + !Ord + !Hash + !Default + {TryInto<Example32>} + !{Into<Example32>},
);

check_type!(Example64 => value : f64);
check_type!(Example64 => value_null : Option<f64>);
check_type!(Example64 => value_default_zero : f64);
check_type!(Example64 => value_default_one : f64);
check_type!(Example64 => create(&mut planus::Builder, f64, f64, f64, f64) : planus::Offset<Example64>);
check_type!(Example64 => create(&mut planus::Builder, f64, (), f64, f64) : planus::Offset<Example64>);
check_type!(Example64 => create(&mut planus::Builder, f64, Option<f64>, f64, f64) : planus::Offset<Example64>);

check_type!(+['a] Example64Ref<'a> => &self.value() : planus::Result<f64>);
check_type!(+['a] Example64Ref<'a> => &self.value_null() : planus::Result<Option<f64>>);
check_type!(+['a] Example64Ref<'a> => &self.value_default_zero() : planus::Result<f64>);
check_type!(+['a] Example64Ref<'a> => &self.value_default_one() : planus::Result<f64>);
check_type!(+['a] Example64Ref<'a> => impl planus::ReadAsRoot<'a>);

assert_traits!(
    Example64: !Copy + Clone + Debug + !Eq + !Ord + !Hash + Default,
    Example64Ref<'_>: Copy + Clone + Debug + !Eq + !Ord + !Hash + !Default + {TryInto<Example64>} + !{Into<Example64>},
);
