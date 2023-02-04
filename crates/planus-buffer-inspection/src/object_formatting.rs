use std::borrow::Cow;

use indexmap::IndexMap;

use crate::{
    allocations::SearchResult, object_info::ObjectName, object_mapping::ObjectMapping,
    InspectableFlatbuffer,
};

/// Maps into the lines Vec
type LineIndex = usize;
/// Maps into the allocation_paths IndexMap
type AllocationPathIndex = usize;

pub struct ObjectFormatting<'a> {
    pub lines: Vec<ObjectFormattingLine<'a>>,
    pub allocation_paths: IndexMap<SearchResult<'a>, LineIndex>,
}

pub struct ObjectFormattingLine<'a> {
    pub indentation: usize,
    pub kind: ObjectFormattingKind<'a>,
    pub byte_range: (usize, usize),
}

pub enum ObjectFormattingKind<'a> {
    Object {
        allocation_path_index: AllocationPathIndex,
        style: BraceStyle<'a>,
    },
    Padding,
}

pub enum BraceStyle<'a> {
    BraceBegin { field_name: Cow<'a, str> },
    BraceEnd,
    LeafObject { field_name: Cow<'a, str> },
}

impl<'a> ObjectFormatting<'a> {
    fn fmt_line(
        &self,
        line: &ObjectFormattingLine<'_>,
        flatbuffer: &InspectableFlatbuffer<'_>,
        object_mapping: &ObjectMapping<'_>,
        show_padding: bool,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match &line.kind {
            ObjectFormattingKind::Object {
                style,
                allocation_path_index,
            } => match style {
                BraceStyle::BraceBegin { field_name } | BraceStyle::LeafObject { field_name } => {
                    let (allocation_path, _) = self
                        .allocation_paths
                        .get_index(*allocation_path_index)
                        .unwrap();
                    let object_index = allocation_path
                        .field_path
                        .last()
                        .unwrap()
                        .allocation
                        .object
                        .unwrap();
                    let (object, _) = object_mapping.all_objects.get_index(object_index).unwrap();
                    let object_name = object.resolve_name(flatbuffer);
                    let object_address = line.byte_range.0;
                    write!(
                        f,
                        "{indentation:>indentation_count$}{field_name}: {object_name} @ {object_address}{curly}",
                        indentation = "",
                        indentation_count = line.indentation,
                        curly = if matches!(style, BraceStyle::BraceBegin { .. }) { " {" } else { "" },
                    )?;
                }
                BraceStyle::BraceEnd => {
                    write!(f, "{indentation:>width$}}}", indentation = "", width = 0)?;
                }
            },
            ObjectFormattingKind::Padding if show_padding => {
                write!(
                    f,
                    "{indentation:>width$}padding",
                    indentation = "",
                    width = 0
                )?;
            }
            ObjectFormattingKind::Padding => (),
        }
        Ok(())
    }

    fn fmt_lines(
        &self,
        flatbuffer: &InspectableFlatbuffer<'_>,
        object_mapping: &ObjectMapping<'_>,
        show_padding: bool,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for line in &self.lines {
            self.fmt_line(line, flatbuffer, object_mapping, show_padding, f)?;
        }
        Ok(())
    }
}
