use crate::model::Project;

const MAX_HISTORY: usize = 50;

pub struct History {
    undo_stack: Vec<Project>,
    redo_stack: Vec<Project>,
}

impl History {
    pub fn new() -> Self {
        History {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Call before committing a mutation, passing the state *before* the change.
    pub fn push(&mut self, before: Project) {
        self.undo_stack.push(before);
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current: Project) -> Option<Project> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current);
        Some(prev)
    }

    pub fn redo(&mut self, current: Project) -> Option<Project> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current);
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
