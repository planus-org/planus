use std::borrow::Cow;

use indexmap::IndexMap;
use planus_types::intermediate::{DeclarationIndex, DeclarationKind};

use crate::{
    children::{Byterange, Children},
    object_info::ObjectName,
    ByteIndex, InspectableFlatbuffer, Object, OffsetObject, OffsetObjectKind,
};

pub type ObjectIndex = usize;
pub type LineIndex = usize;

pub struct ObjectMapping<'a> {
    pub root_object: Object<'a>,
    pub root_objects: IndexMap<Object<'a>, (ByteIndex, ByteIndex)>,
    pub root_intervals: rust_lapper::Lapper<ByteIndex, ObjectIndex>,
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

        let mut builder = ObjectMappingBuilder::default();

        builder.process_root_object(Object::Offset(root_offset_object), self);

        ObjectMapping {
            root_object: root_offset_object.follow_offset(self).unwrap(),
            root_objects: builder.root_objects,
            root_intervals: rust_lapper::Lapper::new(builder.root_intervals),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Interpretation {
    pub root_object_index: ObjectIndex,
    pub lines: Vec<LineIndex>,
}

#[derive(Debug)]
pub struct LineTree<'a> {
    field_name: Option<Cow<'a, str>>,
    object: Object<'a>,
    start_line_index: LineIndex,
    end_line_index: Option<LineIndex>,
    range: (ByteIndex, ByteIndex),
    children: Vec<LineTree<'a>>,
}

#[derive(Clone, Debug)]
pub struct Line<'a> {
    pub indentation: usize,
    pub start_line_index: LineIndex,
    pub end_line_index: LineIndex,
    pub parent_line_index: LineIndex,
    pub name: String,
    pub line: String,
    pub object_width: usize,
    pub start: ByteIndex,
    pub end: ByteIndex,
    pub object: Object<'a>,
}

impl<'a> LineTree<'a> {
    fn get_interpretations(
        &self,
        root_object_index: ObjectIndex,
        byte_index: ByteIndex,
        lines: &mut Vec<LineIndex>,
        callback: &mut impl FnMut(Interpretation),
    ) -> bool {
        if !(self.range.0..self.range.1).contains(&byte_index) {
            return false;
        }

        lines.push(self.start_line_index);

        let mut found = false;
        for child in &self.children {
            found |= child.get_interpretations(root_object_index, byte_index, lines, callback);
        }
        if !found {
            callback(Interpretation {
                root_object_index,
                lines: lines.clone(),
            });
        }
        lines.pop();
        true
    }

    fn to_strings_helper(
        &self,
        depth: usize,
        parent_line_index: LineIndex,
        buffer: &InspectableFlatbuffer<'a>,
        out: &mut Vec<Line<'a>>,
    ) {
        debug_assert_eq!(out.len(), self.start_line_index);

        let mut line = String::new();
        let name = self.object.print_object(buffer);

        if let Object::Offset(OffsetObject {
            kind: OffsetObjectKind::VTable(_),
            ..
        }) = &self.object
        {
            line.push_str("#vtable");
        } else {
            if let Some(field_name) = &self.field_name {
                line.push_str(field_name);
                line.push_str(": ");
            }

            line.push_str(&name);

            if self.end_line_index.is_some() {
                line.push_str(" {");
            } else if self.object.have_braces() {
                line.push_str(" {}");
            }
        }

        out.push(Line {
            object_width: line.len(),
            line,
            name,
            indentation: 2 * depth,
            start_line_index: self.start_line_index,
            end_line_index: self.end_line_index.unwrap_or(self.start_line_index),
            parent_line_index,
            object: self.object,
            start: self.range.0,
            end: self.range.1,
        });

        for child in &self.children {
            let index = out.len();
            child.to_strings_helper(depth + 1, self.start_line_index, buffer, out);
            out[self.start_line_index].object_width = out[self.start_line_index]
                .object_width
                .max(out[index].object_width + 2);
        }

        if let Some(end_line) = self.end_line_index {
            debug_assert_eq!(out.len(), end_line);
            out.push(Line {
                object_width: out[self.start_line_index].object_width,
                name: String::new(),
                indentation: 2 * depth,
                start_line_index: self.start_line_index,
                end_line_index: self.end_line_index.unwrap_or(self.start_line_index),
                line: "}".to_string(),
                parent_line_index,
                object: self.object,
                start: self.range.0,
                end: self.range.1,
            });
        }
    }

    pub fn flatten(&self, buffer: &InspectableFlatbuffer<'a>) -> Vec<Line<'a>> {
        let mut out = Vec::new();
        self.to_strings_helper(0, 0, buffer, &mut out);
        out
    }

    pub fn last_line(&self) -> usize {
        if let Some(end_line) = self.end_line_index {
            end_line
        } else if let Some(last) = self.children.last() {
            last.last_line()
        } else {
            self.start_line_index
        }
    }
}

impl<'a> ObjectMapping<'a> {
    pub fn get_interpretations(
        &self,
        byte_index: ByteIndex,
        buffer: &InspectableFlatbuffer<'a>,
    ) -> Vec<Interpretation> {
        let mut interpretations = Vec::new();
        self.get_interpretations_cb(byte_index, buffer, |interpretation| {
            interpretations.push(interpretation);
        });
        interpretations
    }

    pub fn get_interpretations_cb(
        &self,
        byte_index: ByteIndex,
        buffer: &InspectableFlatbuffer<'a>,
        mut callback: impl FnMut(Interpretation),
    ) {
        for root_object_index in self.root_intervals.find(byte_index, byte_index + 1) {
            self.line_tree(root_object_index.val, buffer)
                .get_interpretations(
                    root_object_index.val,
                    byte_index,
                    &mut Vec::new(),
                    &mut callback,
                );
        }
    }

    pub fn line_tree(
        &self,
        root_object_index: ObjectIndex,
        buffer: &InspectableFlatbuffer<'a>,
    ) -> LineTree<'a> {
        fn handler<'a>(
            field_name: Option<Cow<'a, str>>,
            current: Object<'a>,
            buffer: &InspectableFlatbuffer<'a>,
            next_line: &mut LineIndex,
        ) -> LineTree<'a> {
            let current_line = *next_line;
            *next_line += 1;
            let mut children = Vec::new();
            let mut range = current.byterange(buffer);
            current.children(buffer, |field_name, child| {
                let child = handler(field_name, child, buffer, next_line);
                range.0 = range.0.min(child.range.0);
                range.1 = range.1.max(child.range.1);
                children.push(child);
            });

            let mut end_line = None;

            if !children.is_empty() {
                end_line = Some(*next_line);
                *next_line += 1;
            }

            LineTree {
                field_name,
                object: current,
                start_line_index: current_line,
                end_line_index: end_line,
                range,
                children,
            }
        }
        handler(
            None,
            *self.root_objects.get_index(root_object_index).unwrap().0,
            buffer,
            &mut 0,
        )
    }
}

#[derive(Default)]
struct ObjectMappingBuilder<'a> {
    root_objects: IndexMap<Object<'a>, (ByteIndex, ByteIndex)>,
    root_intervals: Vec<rust_lapper::Interval<ByteIndex, ObjectIndex>>,
}

impl<'a> ObjectMappingBuilder<'a> {
    fn process_root_object(&mut self, current: Object<'a>, buffer: &InspectableFlatbuffer<'a>) {
        if self.root_objects.contains_key(&current) {
            return;
        }

        if let Object::Offset(offset_object) = current {
            if let Ok(inner) = offset_object.follow_offset(buffer) {
                self.process_root_object(inner, buffer);
            }
        }

        let mut range = current.byterange(buffer);

        current.children(buffer, |child_name, child| {
            std::mem::drop(child_name);
            self.process_child_object(child, &mut range, buffer);
        });
        let (index, old) = self.root_objects.insert_full(current, range);
        assert!(old.is_none());
        self.root_intervals.push(rust_lapper::Interval {
            start: range.0,
            stop: range.1,
            val: index,
        });
    }

    fn process_child_object(
        &mut self,
        current: Object<'a>,
        range: &mut (u32, u32),
        buffer: &InspectableFlatbuffer<'a>,
    ) {
        let crange = current.byterange(buffer);
        range.0 = range.0.min(crange.0);
        range.1 = range.1.max(crange.1);

        if let Object::Offset(offset_object) = current {
            if let Ok(inner) = offset_object.follow_offset(buffer) {
                self.process_root_object(inner, buffer);
            }
        }

        current.children(buffer, |child_name, child| {
            std::mem::drop(child_name);
            self.process_child_object(child, range, buffer);
        });
    }
}
