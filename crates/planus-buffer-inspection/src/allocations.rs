use std::{borrow::Cow, collections::BTreeMap};

use crate::{object_mapping::ObjectIndex, ByteIndex};

pub type AllocationStart = ByteIndex;
pub type AllocationEnd = ByteIndex;
pub type AllocationIndex = usize;

#[derive(Default)]
pub struct Allocations<'a> {
    allocations: Vec<Allocation<'a>>,
}

#[derive(Debug)]
pub struct Allocation<'a> {
    pub object: Option<ObjectIndex>,
    pub start: AllocationStart,
    pub end: AllocationEnd,
    pub parents: Vec<AllocationIndex>,
    // Invariant: None of the allocation ranges can overlap
    pub children: BTreeMap<AllocationStart, (AllocationEnd, AllocationChildren<'a>)>,
}

#[derive(Clone, Debug)]
pub enum AllocationChildren<'a> {
    Unique(ChildMapping<'a>),
    Overlapping(Vec<ChildMapping<'a>>),
}

#[derive(Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub struct FieldAccess<'a> {
    pub field_name: &'a str,
    pub allocation: &'a Allocation<'a>,
}

pub type FieldPath<'a> = Vec<FieldAccess<'a>>;
pub struct SearchResult<'a> {
    pub root_object: &'a Allocation<'a>,
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
    pub fn get(&self, offset: ByteIndex) -> Vec<SearchResult<'_>> {
        let mut out = Vec::new();

        let buffer_allocation = &self.allocations[0];

        let mut todo: Vec<(&Allocation, Vec<(usize, &str)>)> =
            vec![(buffer_allocation, Vec::new())];

        while let Some((allocation, mut state)) = todo.pop() {
            if let Some((allocation_start, (allocation_end, children))) =
                allocation.children.range(..=offset).next_back()
            {
                if (*allocation_start..*allocation_end).contains(&offset) {
                    match children {
                        AllocationChildren::Unique(child) => {
                            let allocation = &self.allocations[child.allocation_index];
                            state.push((child.allocation_index, &child.field_name));
                            todo.push((allocation, state));
                            continue;
                        }
                        AllocationChildren::Overlapping(children) => {
                            for child in children {
                                let allocation = &self.allocations[child.allocation_index];
                                let mut state = state.clone();
                                state.push((child.allocation_index, &child.field_name));
                                todo.push((allocation, state));
                            }
                            continue;
                        }
                    }
                }
            }

            out.push(state);
        }

        out.sort();
        out.dedup();
        out.into_iter()
            .filter_map(|r| {
                let mut iter = r.iter();
                let (root_allocation_index, _) = iter.next()?;
                let field_path = iter
                    .map(|&(index, field_name)| FieldAccess {
                        field_name,
                        allocation: &self.allocations[index],
                    })
                    .collect();
                Some(SearchResult {
                    root_object: &self.allocations[*root_allocation_index],
                    field_path,
                })
            })
            .collect()
    }

    pub fn allocate(
        &mut self,
        object: Option<ObjectIndex>,
        allocation_start: AllocationStart,
        allocation_end: AllocationEnd,
    ) -> AllocationIndex {
        let allocation_index = self.allocations.len();
        self.allocations.push(Allocation {
            object,
            start: allocation_start,
            end: allocation_end,
            parents: Vec::new(),
            children: BTreeMap::new(),
        });
        allocation_index
    }

    pub fn insert_new_root(&mut self, allocation_index: AllocationIndex) {
        self.insert_child(0, allocation_index, "".into());
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

        let mut unaltered_end = node.children.split_off(&allocation_end);

        // Fix up the allocation_start to get the previous element as well
        let mut search_allocation_start = allocation_start;

        if let Some((child_allocation_start, (child_allocation_end, _child))) =
            node.children.range(..=allocation_start).next_back()
        {
            if allocation_start < *child_allocation_end {
                search_allocation_start = *child_allocation_start;
            }
        }

        let nodes_to_fixup = node.children.split_off(&search_allocation_start);

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

        node.children.extend(
            nodes_to_fixup
                .into_iter()
                .map(|(interval, children)| (interval.start, (interval.end, children))),
        );
        node.children.append(&mut unaltered_end);
    }
}
