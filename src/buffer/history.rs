//! Undo and Redo history stack.

#[derive(Debug, Clone)]
pub struct History {
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    max_entries: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new(100)
    }
}

impl History {
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_entries,
        }
    }

    /// Records current text before a modification. Clears redo stack.
    pub fn push(&mut self, text: String) {
        if self.undo_stack.last() == Some(&text) {
            return;
        }
        if self.undo_stack.len() >= self.max_entries {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(text);
        self.redo_stack.clear();
    }

    /// Pops the latest state for undo, pushing current into redo.
    pub fn undo(&mut self, current: String) -> Option<String> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current);
        Some(prev)
    }

    /// Pops the latest state for redo, pushing current into undo.
    pub fn redo(&mut self, current: String) -> Option<String> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current);
        Some(next)
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo() {
        let mut hist = History::new(10);
        let s0 = "Init".to_string();
        hist.push(s0.clone());
        let s1 = "Step 1".to_string();
        hist.push(s1.clone());
        let current = "Step 2".to_string();

        assert!(hist.can_undo());
        let restored = hist.undo(current.clone()).unwrap();
        assert_eq!(restored, s1);

        assert!(hist.can_redo());
        let redone = hist.redo(restored).unwrap();
        assert_eq!(redone, current);
    }
}
