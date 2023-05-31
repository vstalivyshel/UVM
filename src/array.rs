impl<T: Copy + Default, const N: usize> Default for Array<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Array<T, const N: usize> {
    pub items: [T; N],
    pub size: usize,
}

impl<T: Copy + Default, const N: usize> Array<T, N> {
    pub fn new() -> Self {
        Self {
            items: [T::default(); N],
            size: 0,
        }
    }

    pub fn get_last(&self) -> T {
        self.items[if self.size < 1 { 0 } else { self.size - 1 }]
    }

    pub fn push(&mut self, item: T) {
        self.items[self.size] = item;
        self.size += 1;
    }

    pub fn get(&self, idx: usize) -> T {
        self.items[idx]
    }

    pub fn pop(&mut self) -> T {
        self.size -= 1;
        self.items[self.size]
    }

    pub fn replace(&mut self, idx: usize, item: T) {
        self.items[idx] = item;
    }
}
