use std::borrow::Cow;

use indexmap::IndexMap;

use crate::{object_info::ObjectName, object_mapping::ObjectIndex, InspectableFlatbuffer, Object};

/// Maps into the lines Vec
type LineIndex = usize;
/// Maps into the allocation_paths IndexMap
type AllocationPathIndex = usize;

pub struct ObjectFormatting<'a> {
    pub root_object: Object<'a>,
    pub root_object_range: (usize, usize),
    pub lines: Vec<ObjectFormattingLine<'a>>,
    pub allocation_paths: IndexMap<Vec<(&'a str, ObjectIndex)>, LineIndex>,
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
        object: Object<'a>,
    },
    Padding,
}

pub enum BraceStyle<'a> {
    BraceBegin { field_name: &'a str },
    BraceEnd,
    LeafObject { field_name: &'a str },
}

impl<'a> ObjectFormatting<'a> {
    fn fmt_line(
        &self,
        line: &ObjectFormattingLine<'_>,
        flatbuffer: &InspectableFlatbuffer<'_>,
        show_padding: bool,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match &line.kind {
            ObjectFormattingKind::Object { style, object, .. } => match style {
                BraceStyle::BraceBegin { field_name } | BraceStyle::LeafObject { field_name } => {
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
        show_padding: bool,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for line in &self.lines {
            self.fmt_line(line, flatbuffer, show_padding, f)?;
        }
        Ok(())
    }

    pub fn to_string(&self, flatbuffer: &InspectableFlatbuffer<'_>) -> String {
        struct Helper<'a> {
            flatbuffer: &'a InspectableFlatbuffer<'a>,
            object: &'a ObjectFormatting<'a>,
        }

        impl<'a> std::fmt::Display for Helper<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.object.fmt_lines(self.flatbuffer, true, f)
            }
        }

        Helper {
            flatbuffer,
            object: self,
        }
        .to_string()
    }
}
