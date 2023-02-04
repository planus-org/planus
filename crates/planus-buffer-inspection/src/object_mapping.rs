use std::collections::HashMap;

use indexmap::{map::Entry, IndexMap};
use planus_types::intermediate::{DeclarationIndex, DeclarationKind};

use crate::{
    allocations::{AllocationIndex, Allocations, AllocationsBuilder},
    children::{Byterange, Children},
    ByteIndex, InspectableFlatbuffer, Object, OffsetObject,
};

pub type ObjectIndex = usize;

pub struct ObjectMapping<'a> {
    pub root_object: OffsetObject<'a>,
    pub all_objects: IndexMap<Object<'a>, AllocationIndex>,
    pub vtable_locations: HashMap<ByteIndex, AllocationIndex>,
    pub allocations: Allocations<'a>,
    pub parents: HashMap<ObjectIndex, Vec<ObjectIndex>>,
}

impl<'a> InspectableFlatbuffer<'a> {
    pub fn calculate_object_mapping(
        &self,
        root_table_index: DeclarationIndex,
    ) -> ObjectMapping<'a> {
        assert!(matches!(
            self.declarations.get_declaration(root_table_index).1.kind,
            DeclarationKind::Table(_)
        ));

        let root_object = OffsetObject {
            offset: 0,
            kind: crate::OffsetObjectKind::Table(root_table_index),
        };

        let mut builder = ObjectMappingBuilder::default();

        let root_allocation_index = builder.handle_node(Object::Offset(root_object), self);
        builder.allocations.insert_new_root(root_allocation_index);

        ObjectMapping {
            root_object,
            all_objects: builder.all_objects,
            vtable_locations: builder.vtable_locations,
            allocations: builder.allocations.finish(),
            parents: builder.parents,
        }
    }
}

#[derive(Default)]
pub struct ObjectMappingBuilder<'a> {
    pub all_objects: IndexMap<Object<'a>, AllocationIndex>,
    pub vtable_locations: HashMap<ByteIndex, AllocationIndex>,
    pub allocations: AllocationsBuilder<'a>,
    pub parents: HashMap<ObjectIndex, Vec<ObjectIndex>>,
}

impl<'a> ObjectMappingBuilder<'a> {
    fn handle_node(
        &mut self,
        object: Object<'a>,
        buffer: &InspectableFlatbuffer<'a>,
    ) -> AllocationIndex {
        let object_index;
        let allocation_index;

        match self.all_objects.entry(object) {
            Entry::Occupied(entry) => {
                return *entry.get();
            }
            Entry::Vacant(entry) => {
                object_index = entry.index();
                let range = object.byterange(buffer);
                allocation_index = self
                    .allocations
                    .allocate(object_index, range.start, range.end);
                entry.insert(allocation_index);
            }
        }

        for (child_name, child) in object.children(buffer) {
            let child_allocation_index = self.handle_node(child, buffer);
            if matches!(object, Object::Offset(_)) {
                self.allocations.insert_new_root(child_allocation_index);
            } else {
                self.allocations
                    .insert_child(allocation_index, child_allocation_index, child_name);
            }
        }
        allocation_index
    }
}
