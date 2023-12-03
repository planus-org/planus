assert_traits!(
    InfallibleStructRef<'_>: {TryInto<InfallibleStruct>} + {Into<InfallibleStruct>},
    FallibleStructRef<'_>: {TryInto<FallibleStruct>} + !{Into<FallibleStruct>},
    InfallibleUnionRef<'_>: {TryInto<InfallibleUnion>} + {Into<InfallibleUnion>},
    FallibleUnionRef<'_>: {TryInto<FallibleUnion>} + !{Into<FallibleUnion>},
);
