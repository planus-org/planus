use std::collections::BTreeMap;

use indexmap::IndexSet;
use planus_types::intermediate::{DeclarationIndex, DeclarationKind};

use crate::{
    children::{Byterange, Children},
    ByteIndex, InspectableFlatbuffer, Object, OffsetObject,
};

pub type ObjectIndex = usize;

pub struct ObjectMapping<'a> {
    pub root_object: OffsetObject<'a>,
    pub all_objects: IndexSet<Object<'a>>,
    pub primary_byte_mapping: Vec<ObjectIndex>,
    pub secondary_byte_mapping: BTreeMap<ByteIndex, Vec<ObjectIndex>>,
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

        let root_offset_object = OffsetObject {
            offset: 0,
            kind: crate::OffsetObjectKind::Table(root_table_index),
        };
        let root_object = Object::Offset(root_offset_object);

        let mut todo = vec![(root_object, 0)];
        let mut all_objects = IndexSet::new();
        let mut primary_byte_mapping = vec![usize::MAX; self.buffer.len()];
        let mut secondary_byte_mapping: BTreeMap<ByteIndex, Vec<ObjectIndex>> = BTreeMap::new();
        let mut parents: BTreeMap<ObjectIndex, Vec<ObjectIndex>> = BTreeMap::new();
        all_objects.insert(root_object);
        for byte in root_offset_object.byterange(self).into_iter().flatten() {
            primary_byte_mapping[byte] = 0;
        }

        while let Some((parent_object, parent_object_index)) = todo.pop() {
            for (_name, child) in parent_object.children(self) {
                let (child_index, inserted) = all_objects.insert_full(child);
                if inserted {
                    parents
                        .entry(child_index)
                        .or_default()
                        .push(parent_object_index);
                    for byte in child.byterange(self).into_iter().flatten() {
                        if primary_byte_mapping[byte] == usize::MAX {
                            primary_byte_mapping[byte] = child_index;
                        } else {
                            secondary_byte_mapping
                                .entry(byte)
                                .or_default()
                                .push(child_index);
                        }
                    }
                    todo.push((child, child_index));
                }
            }
        }

        ObjectMapping {
            root_object: root_offset_object,
            all_objects,
            primary_byte_mapping,
            secondary_byte_mapping,
            parents,
        }
    }
}

impl<'a> ObjectMapping<'a> {
    pub fn get_bytes_for_pos<'b>(
        &'b self,
        byte_index: ByteIndex,
    ) -> impl 'b + Iterator<Item = Object<'a>> {
        let primary_mapping = self.primary_byte_mapping[byte_index];
        let secondary_mappings = self
            .secondary_byte_mapping
            .get(&byte_index)
            .cloned()
            .unwrap_or_default();
        let primary_mapping = (primary_mapping != usize::MAX).then_some(primary_mapping);

        primary_mapping
            .into_iter()
            .chain(secondary_mappings.into_iter())
            .map(|index| *self.all_objects.get_index(index).unwrap())
    }
}
