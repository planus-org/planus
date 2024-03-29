use std::collections::{HashSet, VecDeque};

use planus_types::intermediate::{Declaration, DeclarationIndex, Declarations};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WasChanged {
    NoChange,
    Changed,
}

pub trait DeclarationAnalysis {
    type State;

    fn new_state(
        &mut self,
        declarations: &Declarations,
        decl_id: DeclarationIndex,
        declaration: &Declaration,
    ) -> Self::State;
    fn update_state(
        &mut self,
        declarations: &Declarations,
        decl_id: DeclarationIndex,
        declaration: &Declaration,
        states: &mut [Self::State],
    ) -> WasChanged;
}

pub fn run_analysis<A: DeclarationAnalysis>(
    declarations: &Declarations,
    analysis: &mut A,
) -> Vec<A::State> {
    let mut states = declarations
        .declarations
        .values()
        .enumerate()
        .map(|(decl_id, decl)| analysis.new_state(declarations, DeclarationIndex(decl_id), decl))
        .collect::<Vec<_>>();

    let mut queue = (0..declarations.declarations.len()).collect::<Vec<_>>();
    queue.sort_by_key(|&decl_id| declarations.children[decl_id].len());
    let mut queue = VecDeque::from(queue);
    // Will always contain the exact same as the queue
    let mut queue_lookup = (0..declarations.declarations.len()).collect::<HashSet<_>>();

    while let Some(decl_id) = queue.pop_front() {
        queue_lookup.remove(&decl_id);

        let notify_parents = analysis.update_state(
            declarations,
            DeclarationIndex(decl_id),
            &declarations.declarations[decl_id],
            &mut states,
        );
        if notify_parents == WasChanged::Changed {
            for &parent in &declarations.parents[decl_id] {
                if queue_lookup.insert(parent.0) {
                    queue.push_back(parent.0);
                }
            }
        }
    }

    states
}
