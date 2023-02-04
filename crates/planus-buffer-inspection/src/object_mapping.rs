use std::collections::{BTreeMap, HashMap};

use indexmap::{map::Entry, IndexMap};
use planus_types::intermediate::{DeclarationIndex, DeclarationKind};

use crate::{
    allocations::{AllocationIndex, Allocations},
    children::{Byterange, Children},
    ByteIndex, InspectableFlatbuffer, Object, OffsetObject,
};

pub type ObjectIndex = usize;

pub struct ObjectMapping<'a> {
    pub root_object: OffsetObject<'a>,
    pub all_objects: IndexMap<Object<'a>, AllocationIndex>,
    pub vtable_locations: HashMap<ByteIndex, AllocationIndex>,
    pub allocations: Allocations,
    pub parents: BTreeMap<ObjectIndex, Vec<ObjectIndex>>,
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

        let mut result = ObjectMapping {
            root_object,
            vtable_locations: Default::default(),
            all_objects: Default::default(),
            allocations: Default::default(),
            parents: Default::default(),
        };

        let buffer_allocation_index = result.allocations.allocate(None, 0, self.buffer.len());

        let root_allocation_index =
            result.handle_node(Object::Offset(root_object), self, buffer_allocation_index);
        result
            .allocations
            .insert_child(buffer_allocation_index, root_allocation_index);

        result
    }
}

impl<'a> ObjectMapping<'a> {
    fn handle_node(
        &mut self,
        object: Object<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        buffer_allocation_index: AllocationIndex,
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
                allocation_index =
                    self.allocations
                        .allocate(Some(object_index), range.start, range.end);
                entry.insert(allocation_index);
            }
        }

        for (_child_name, child) in object.children(buffer) {
            let child_allocation_index = self.handle_node(child, buffer, buffer_allocation_index);
            let should_insert = match object {
                Object::Offset(_) => false,
                Object::VTable(_)
                | Object::Table(_)
                | Object::Struct(_)
                | Object::Vector(_)
                | Object::Array(_)
                | Object::UnionTag(_)
                | Object::Enum(_)
                | Object::String(_) => true,
                Object::Union(_) | Object::Integer(_) | Object::Float(_) | Object::Bool(_) => {
                    unreachable!()
                }
            };
            if should_insert {
                self.allocations
                    .insert_child(allocation_index, child_allocation_index);
            } else {
                self.allocations
                    .insert_child(buffer_allocation_index, child_allocation_index);
            }
        }
        allocation_index
    }
}