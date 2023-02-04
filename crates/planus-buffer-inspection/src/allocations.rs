use std::collections::BTreeMap;

use crate::{object_mapping::ObjectIndex, ByteIndex};

pub type AllocationStart = ByteIndex;
pub type AllocationEnd = ByteIndex;
pub type AllocationIndex = usize;

#[derive(Default)]
pub struct Allocations {
    allocations: Vec<Allocation>,
}

#[derive(Debug)]
pub struct Allocation {
    pub object: Option<ObjectIndex>,
    pub start: AllocationStart,
    pub end: AllocationEnd,
    pub parents: Vec<AllocationIndex>,
    // Invariant: None of the allocation ranges can overlap
    pub children: BTreeMap<AllocationStart, (AllocationEnd, AllocationChildren)>,
}

#[derive(Clone, Debug)]
pub enum AllocationChildren {
    Unique(AllocationIndex),
    Overlapping(Vec<AllocationIndex>),
}

impl AllocationChildren {
    pub fn extend(&mut self, other: &AllocationChildren) {
        if let AllocationChildren::Unique(child) = *self {
            *self = AllocationChildren::Overlapping(vec![child])
        }

        match self {
            AllocationChildren::Unique(_) => unreachable!(),
            AllocationChildren::Overlapping(vec) => match other {
                AllocationChildren::Unique(child) => {
                    vec.push(*child);
                }
                AllocationChildren::Overlapping(children) => {
                    vec.extend_from_slice(children);
                }
            },
        }
    }

    pub fn children(&self) -> &[AllocationIndex] {
        match self {
            AllocationChildren::Unique(child) => std::slice::from_ref(child),
            AllocationChildren::Overlapping(children) => children,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SearchResult<T> {
    pub result: Vec<T>,
}

impl<T> SearchResult<T> {
    fn push(&mut self, object: T) {
        let _ = self.result.push(object);
    }

    fn map<U>(self, f: impl FnMut(T) -> U) -> SearchResult<U> {
        SearchResult {
            result: self.result.into_iter().map(f).collect(),
        }
    }
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

impl Allocations {
    pub fn get(&self, offset: ByteIndex) -> Vec<SearchResult<&Allocation>> {
        let mut out = Vec::new();

        let root_allocation = &self.allocations[0];
        let initial_state = SearchResult { result: Vec::new() };

        let mut todo = vec![(root_allocation, initial_state)];

        while let Some((allocation, mut state)) = todo.pop() {
            if let Some((allocation_start, (allocation_end, children))) =
                allocation.children.range(..=offset).next_back()
            {
                if (*allocation_start..*allocation_end).contains(&offset) {
                    match children {
                        AllocationChildren::Unique(child_index) => {
                            let child = &self.allocations[*child_index];
                            state.push(*child_index);
                            todo.push((child, state));
                            continue;
                        }
                        AllocationChildren::Overlapping(children) => {
                            for child_index in children {
                                let child = &self.allocations[*child_index];
                                let mut state = state.clone();
                                state.push(*child_index);
                                todo.push((child, state));
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
            .map(|r| r.map(|index| &self.allocations[index]))
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

    pub fn insert_child(
        &mut self,
        parent_allocation_index: AllocationIndex,
        child_allocation_index: AllocationIndex,
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
            AllocationChildren::Unique(child_allocation_index),
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
