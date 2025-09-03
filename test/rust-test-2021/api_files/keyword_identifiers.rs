// Check that keywords used as field names produce the expected Rust identifiers.
// Rust keywords get a trailing _ appended by the code generator.
check_type!(Include => namespace : i32);
check_type!(Include => attribute : i32);
check_type!(Include => table : i32);
check_type!(Include => struct_ : i32);
check_type!(Include => enum_ : i32);
check_type!(Include => union_ : i32);
check_type!(Include => root_type : i32);
check_type!(Include => rpc_service : i32);
check_type!(Include => file_extension : i32);
check_type!(Include => file_identifier : i32);

// Check that keywords used as enum variants produce the expected Rust identifiers.
check_enum_variants!(NativeInclude: i8 {
  Namespace = 0,
  Attribute = 1,
  Table = 2,
  Struct = 3,
  Enum = 4,
  Union = 5,
});
