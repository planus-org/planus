check_type!(Example => value : Option<Vec<f32>>);
check_type!(Example => value_null : Option<Vec<f32>>);
check_type!(Example => value_default_empty : Vec<f32>);
check_type!(Example => value_required : Vec<f32>);
check_type!(Example => create(&mut planus::Buffer, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, (), (), Vec<f32>, Vec<f32>) : planus::Offset<Example>);
check_type!(Example => create(&mut planus::Buffer, Option<Vec<f32>>, Option<Vec<f32>>, Vec<f32>, Vec<f32>) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Buffer, &'a [f32], &'b [f32], &'c [f32], &'d [f32]) : planus::Offset<Example>);
check_type!(+['a, 'b, 'c, 'd] Example => create(&mut planus::Buffer, Option<&'a [f32]>, Option<&'b [f32]>, &'c [f32], &'d [f32]) : planus::Offset<Example>);

check_type!(+['a] ExampleRef<'a> => &self.value() : planus::Result<Option<planus::Vector<'a, f32>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_null() : planus::Result<Option<planus::Vector<'a, f32>>>);
check_type!(+['a] ExampleRef<'a> => &self.value_default_empty() : planus::Result<planus::Vector<'a, f32>>);
check_type!(+['a] ExampleRef<'a> => &self.value_required() : planus::Result<planus::Vector<'a, f32>>);
