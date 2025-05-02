use crate::{Action, Id, Preselection};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NormalizeDidAction {
    pub did_id: Id,
}

impl Action for NormalizeDidAction {
    fn preselect(&self, preselection: &mut Preselection, _index: usize) {
        preselection.spent_dids.insert(self.did_id);
    }
}
