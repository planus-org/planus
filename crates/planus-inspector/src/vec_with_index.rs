#[derive(Debug)]
pub struct VecWithIndex<T> {
    index: usize,
    values: Vec<T>,
}

impl<T> VecWithIndex<T> {
    pub fn new(values: Vec<T>, index: usize) -> Self {
        assert!(index < values.len());
        Self { index, values }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns true if the index was changed
    pub fn try_set_index(&mut self, index: usize) -> bool {
        if index < self.values.len() && index != self.index {
            self.index = index;
            true
        } else {
            false
        }
    }

    pub fn set_index(&mut self, index: usize) {
        assert!(index < self.values.len());
        self.index = index;
    }

    pub fn cur(&self) -> &T {
        &self.values[self.index]
    }

    pub fn cur_mut(&mut self) -> &mut T {
        &mut self.values[self.index]
    }
}

impl<T> std::ops::Deref for VecWithIndex<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<T> std::ops::DerefMut for VecWithIndex<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}
