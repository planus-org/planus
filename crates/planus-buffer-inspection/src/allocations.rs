use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use indexmap::IndexMap;

use crate::{
    object_formatting::{BraceStyle, ObjectFormatting, ObjectFormattingKind, ObjectFormattingLine},
    object_mapping::{ObjectIndex, ObjectMapping},
    ByteIndex,
};

pub type AllocationStart = ByteIndex;
pub type AllocationEnd = ByteIndex;
pub type AllocationIndex = usize;

#[derive(Default)]
pub struct Allocations<'a> {
    pub roots: IntervalTree<'a>,
    pub allocations: Vec<Allocation<'a>>,
}

#[derive(Default, Debug)]
pub struct IntervalTree<'a> {
    // Invariant: None of the allocation ranges can overlap
    allocations: BTreeMap<AllocationStart, (AllocationEnd, AllocationChildren<'a>)>,
}

impl<'a> IntervalTree<'a> {
    pub fn get_children(&self, offset: ByteIndex) -> Option<&AllocationChildren<'a>> {
        let (allocation_start, (allocation_end, children)) =
            self.allocations.range(..=offset).next_back()?;
        (*allocation_start..*allocation_end)
            .contains(&offset)
            .then_some(children)
    }

    // TODO: This should also return internal padding bytes
    // TODO: This should return children sorted after where they start
    pub fn get_all_children(&self) -> BTreeSet<&ChildMapping<'a>> {
        self.allocations
            .values()
            .flat_map(|(_, children)| children.children())
            .collect()
    }
}

#[derive(Debug)]
pub struct Allocation<'a> {
    pub object_index: ObjectIndex,
    pub start: AllocationStart,
    pub end: AllocationEnd,
    pub parents: Vec<AllocationIndex>,
    pub children: IntervalTree<'a>,
}

#[derive(Clone, Debug)]
pub enum AllocationChildren<'a> {
    Unique(ChildMapping<'a>),
    Overlapping(Vec<ChildMapping<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChildMapping<'a> {
    field_name: Cow<'a, str>,
    allocation_index: AllocationIndex,
}

impl<'a> AllocationChildren<'a> {
    pub fn extend(&mut self, other: &AllocationChildren<'a>) {
        if let AllocationChildren::Unique(child) = self {
            let child = std::mem::replace(
                child,
                ChildMapping {
                    field_name: "".into(),
                    allocation_index: 0,
                },
            );
            *self = AllocationChildren::Overlapping(vec![child]);
        }

        match self {
            AllocationChildren::Unique(..) => unreachable!(),
            AllocationChildren::Overlapping(vec) => match other {
                AllocationChildren::Unique(child) => {
                    vec.push(child.clone());
                }
                AllocationChildren::Overlapping(children) => {
                    vec.extend_from_slice(&children);
                }
            },
        }
    }

    pub fn children(&self) -> &[ChildMapping<'a>] {
        match self {
            AllocationChildren::Unique(child) => std::slice::from_ref(child),
            AllocationChildren::Overlapping(children) => children,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldAccess<'a> {
    pub object_index: ObjectIndex,
    pub field_name: Cow<'a, str>,
}

pub type FieldPath<'a> = Vec<FieldAccess<'a>>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SearchResult<'a> {
    pub root_object_index: ObjectIndex,
    pub field_path: FieldPath<'a>,
}

struct Interval {
    start: ByteIndex,
    end: ByteIndex,
}

impl Interval {
    pub fn overlaps(&self, other: &Interval) -> bool {
        !(self.end <= other.start || other.end <= self.start)
    }
}

impl<'a> Allocations<'a> {
    pub fn get(&self, offset: ByteIndex) -> Vec<SearchResult<'a>> {
        let mut out: Vec<SearchResult<'a>> = Vec::new();

        let root_allocations = self
            .roots
            .get_children(offset)
            .into_iter()
            .flat_map(|children| {
                children
                    .children()
                    .iter()
                    .map(|child| child.allocation_index)
            });

        let mut todo: Vec<(&IntervalTree<'_>, SearchResult<'a>)> = root_allocations
            .map(|root_allocation_index| {
                (
                    &self.allocations[root_allocation_index].children,
                    SearchResult {
                        root_object_index: self.allocations[root_allocation_index].object_index,
                        field_path: Vec::new(),
                    },
                )
            })
            .collect();

        while let Some((children, mut state)) = todo.pop() {
            if let Some((allocation_start, (allocation_end, children))) =
                children.allocations.range(..=offset).next_back()
            {
                if (*allocation_start..*allocation_end).contains(&offset) {
                    match children {
                        AllocationChildren::Unique(child) => {
                            let allocation = &self.allocations[child.allocation_index];
                            state.field_path.push(FieldAccess {
                                field_name: child.field_name.clone(),
                                object_index: allocation.object_index,
                            });
                            todo.push((&allocation.children, state));
                            continue;
                        }
                        AllocationChildren::Overlapping(children) => {
                            for child in children {
                                let allocation = &self.allocations[child.allocation_index];
                                let mut state = state.clone();
                                state.field_path.push(FieldAccess {
                                    field_name: child.field_name.clone(),
                                    object_index: allocation.object_index,
                                });
                                todo.push((&allocation.children, state));
                            }
                            continue;
                        }
                    }
                }
            }

            out.push(state);
        }

        out.sort();
        out
    }

    pub fn allocate(
        &mut self,
        object: ObjectIndex,
        allocation_start: AllocationStart,
        allocation_end: AllocationEnd,
    ) -> AllocationIndex {
        let allocation_index = self.allocations.len();
        self.allocations.push(Allocation {
            object_index: object,
            start: allocation_start,
            end: allocation_end,
            parents: Vec::new(),
            children: IntervalTree::default(),
        });
        allocation_index
    }

    pub fn insert_new_root(&mut self, allocation_index: AllocationIndex) {
        let node = &mut self.allocations[allocation_index];

        let allocation_start = node.start;
        let allocation_end = node.end;
        self.roots.insert_allocation(
            allocation_start,
            allocation_end,
            "".into(),
            allocation_index,
        );
    }

    pub fn insert_child(
        &mut self,
        parent_allocation_index: AllocationIndex,
        child_allocation_index: AllocationIndex,
        field_name: Cow<'a, str>,
    ) {
        let child_node = &mut self.allocations[child_allocation_index];
        if child_node.parents.contains(&parent_allocation_index) {
            return;
        }
        child_node.parents.push(parent_allocation_index);

        let allocation_start = child_node.start;
        let allocation_end = child_node.end;

        let mut node = &mut self.allocations[parent_allocation_index];
        assert!(node.parents.is_empty());
        node.start = node.start.min(allocation_start);
        node.end = node.end.max(allocation_end);

        node.children.insert_allocation(
            allocation_start,
            allocation_end,
            field_name,
            child_allocation_index,
        );
    }
}

impl<'a> IntervalTree<'a> {
    fn insert_allocation(
        &mut self,
        allocation_start: usize,
        allocation_end: usize,
        field_name: Cow<'a, str>,
        child_allocation_index: usize,
    ) {
        let mut unaltered_end = self.allocations.split_off(&allocation_end);

        // Fix up the allocation_start to get the previous element as well
        let mut search_allocation_start = allocation_start;

        if let Some((child_allocation_start, (child_allocation_end, _child))) =
            self.allocations.range(..=allocation_start).next_back()
        {
            if allocation_start < *child_allocation_end {
                search_allocation_start = *child_allocation_start;
            }
        }

        let nodes_to_fixup = self.allocations.split_off(&search_allocation_start);

        let mut nodes_to_fixup = nodes_to_fixup
            .into_iter()
            .map(|(allocation_start, (allocation_end, children))| {
                (
                    Interval {
                        start: allocation_start,
                        end: allocation_end,
                    },
                    children,
                )
            })
            .collect::<Vec<_>>();
        nodes_to_fixup.push((
            Interval {
                start: allocation_start,
                end: allocation_end,
            },
            AllocationChildren::Unique(ChildMapping {
                field_name,
                allocation_index: child_allocation_index,
            }),
        ));

        'outer: loop {
            for i in 0..nodes_to_fixup.len() - 1 {
                let (a_interval, _a) = &nodes_to_fixup[i];
                let (b_interval, _b) = &nodes_to_fixup[i + 1];
                if !a_interval.overlaps(b_interval) {
                    continue;
                }

                let (b_interval, b) = nodes_to_fixup.swap_remove(i + 1);
                let (a_interval, a) = nodes_to_fixup.swap_remove(i);

                let mut split_points = [
                    a_interval.start,
                    a_interval.end,
                    b_interval.start,
                    b_interval.end,
                ];
                split_points.sort();

                for window in split_points.windows(2) {
                    if window[0] == window[1] {
                        continue;
                    }

                    let interval = Interval {
                        start: window[0],
                        end: window[1],
                    };

                    match (
                        interval.overlaps(&a_interval),
                        interval.overlaps(&b_interval),
                    ) {
                        (true, true) => {
                            let mut combined = a.clone();
                            combined.extend(&b);
                            nodes_to_fixup.push((interval, combined));
                        }
                        (true, false) => {
                            nodes_to_fixup.push((interval, a.clone()));
                        }
                        (false, true) => {
                            nodes_to_fixup.push((interval, b.clone()));
                        }
                        (false, false) => unreachable!(),
                    }
                }

                nodes_to_fixup.sort_by_key(|(interval, _children)| interval.start);

                continue 'outer;
            }

            break;
        }

        self.allocations.extend(
            nodes_to_fixup
                .into_iter()
                .map(|(interval, children)| (interval.start, (interval.end, children))),
        );
        self.allocations.append(&mut unaltered_end);
    }
}

impl<'a> Allocation<'a> {
    pub fn to_formatting(&self, object_mapping: &ObjectMapping<'a>) -> ObjectFormatting<'a> {
        fn handler<'a, 'b>(
            child_mapping: &'b ChildMapping<'a>,
            object_mapping: &ObjectMapping<'a>,
            path: &mut Vec<FieldAccess<'a>>,
            out: &mut ObjectFormatting<'a>,
        ) {
            let allocation: &Allocation<'a> =
                &object_mapping.allocations.allocations[child_mapping.allocation_index];
            path.push(FieldAccess {
                object_index: allocation.object_index,
                field_name: child_mapping.field_name.clone(),
            });
            let allocation_path_index = out.allocation_paths.len();
            assert!(out
                .allocation_paths
                .insert(path.clone(), out.lines.len())
                .is_none());
            if allocation.children.allocations.is_empty() {
                out.lines.push(ObjectFormattingLine {
                    indentation: path.len() * 2,
                    kind: ObjectFormattingKind::Object {
                        allocation_path_index,
                        style: BraceStyle::LeafObject {
                            field_name: child_mapping.field_name.clone(),
                        },
                        object: object_mapping
                            .all_objects
                            .get_index(allocation.object_index)
                            .unwrap()
                            .0
                            .clone(),
                    },
                    byte_range: (allocation.start, allocation.end),
                });
            } else {
                out.lines.push(ObjectFormattingLine {
                    indentation: path.len() * 2,
                    kind: ObjectFormattingKind::Object {
                        allocation_path_index,
                        style: BraceStyle::BraceBegin {
                            field_name: child_mapping.field_name.clone(),
                        },
                        object: object_mapping
                            .all_objects
                            .get_index(allocation.object_index)
                            .unwrap()
                            .0
                            .clone(),
                    },
                    byte_range: (allocation.start, allocation.end),
                });
                for child_mapping in allocation.children.get_all_children() {
                    handler(child_mapping, object_mapping, path, out);
                }
                out.lines.push(ObjectFormattingLine {
                    indentation: path.len() * 2,
                    kind: ObjectFormattingKind::Object {
                        allocation_path_index,
                        style: BraceStyle::BraceEnd,
                        object: *object_mapping
                            .all_objects
                            .get_index(allocation.object_index)
                            .unwrap()
                            .0,
                    },
                    byte_range: (allocation.start, allocation.end),
                });
            }
            path.pop().unwrap();
        }
        let root_object = *object_mapping
            .all_objects
            .get_index(self.object_index)
            .unwrap()
            .0;
        let mut out = ObjectFormatting {
            lines: Vec::new(),
            allocation_paths: IndexMap::new(),
            root_object,
            root_object_range: (self.start, self.end),
        };

        out.allocation_paths.insert(Vec::new(), 0);
        out.lines.push(ObjectFormattingLine {
            indentation: 0,
            kind: ObjectFormattingKind::Object {
                allocation_path_index: 0,
                style: BraceStyle::RootObject,
                object: root_object,
            },
            byte_range: (self.start, self.end),
        });

        for child_mapping in self.children.get_all_children() {
            handler(child_mapping, object_mapping, &mut Vec::new(), &mut out);
        }

        out.lines.push(ObjectFormattingLine {
            indentation: 0,
            kind: ObjectFormattingKind::Object {
                allocation_path_index: 0,
                style: BraceStyle::BraceEnd,
                object: root_object,
            },
            byte_range: (self.start, self.end),
        });
        out
    }
}

/*

search_results
search_results_index
current_root_object
current_line: usize
current_byte: usize

fn hex_formatting_ranges(current_object, current_line) -> ((usize, usize), (usize, usize)) {
    let root_object = current_root_object.object;
    let current_object = current_root_object.lines[current_line].object;
    (
        (root_object.start, root_object.end),
        (current_object.start, current_object.end),
    )
}

fn info_formatting_ranges(current_object, current_line) -> ((usize, usize), (usize, usize)) {
    todo!()
}

fn cycle_interpretation() {
    search_results_index = (search_results_index + 1) % search_results.len();
    current_root_object = search_results[search_results_index].root_allocation.to_format();
    current_line = current_object.find_matching(search_results[search_results_index]);
}

fn set_current_line(new_current_line: usize)
    if new_current_line != current_line {
        current_line = new_current_line;
        let current_object = current_root_object.lines[current_line].object;
        current_byte = current_object.start;
        search_results = calculate_search_results();
        search_results_index = search_results.position(|search_result| search_result.matches(current_object.path)).unwrap();
    }
}

fn set_byte(new_current_byte)
    if new_current_byte != current_byte {
        current_byte = new_current_byte;
        search_results = calculate_search_results(current_byte);

        let current_object = current_root_object.lines[current_line].object;
        (search_result_index, current_root_object, current_line) = find_closest_match(search_results, current_object.path);
    }
}
*/
