use crate::{
    compare_events::compare_events,
    sweep_event::{SweepEventArena, SweepEventId},
};

pub(crate) struct MinHeapByCompareEvents(Vec<SweepEventId>);

impl MinHeapByCompareEvents {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, id: SweepEventId, _: &SweepEventArena) {
        self.0.push(id);
    }

    pub fn pop(&mut self, arena: &SweepEventArena) -> Option<SweepEventId> {
        let (index, se) = if let Some((index, se)) =
            self.0.iter().enumerate().min_by(|&(_, a_id), &(_, b_id)| {
                compare_events(&arena[*a_id], &arena[*b_id], arena)
            }) {
            (index, *se)
        } else {
            return None;
        };
        self.0.remove(index);
        Some(se)
    }
}
