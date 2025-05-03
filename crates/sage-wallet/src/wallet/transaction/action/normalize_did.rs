use crate::{Action, Id, Summary};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NormalizeDidAction {
    pub did_id: Id,
}

impl Action for NormalizeDidAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_dids.insert(self.did_id);
    }
}
