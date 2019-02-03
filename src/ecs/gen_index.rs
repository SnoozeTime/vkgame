use serde_derive::{Serialize, Deserialize};

/// Used to index entities in a generationIndexArray
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct GenerationalIndex {
    index: usize,
    generation: u64,
} 

impl GenerationalIndex {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
}


#[derive(Debug)]
struct AllocatorEntry {
    is_live: bool,
    generation: u64,
}

/// Will allocate a new generational index.
/// --------------------------------------
#[derive(Debug)]
pub struct GenerationalIndexAllocator {
    entries: Vec<AllocatorEntry>,
    free: Vec<usize>,
}

impl GenerationalIndexAllocator {
    pub fn new() -> Self {
        GenerationalIndexAllocator {
            entries: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn allocate(&mut self) -> GenerationalIndex {
        // first, if one free, take it.
        match self.free.pop() {
            Some(idx) => {
                // Increment generation for the entry.
                // Unwrap here. This should never fail.
                let entry = self.entries.get_mut(idx)
                    .expect("Entry and free vector do not match");

                entry.is_live = true;
                entry.generation += 1;

                GenerationalIndex {
                    index: idx,
                    generation: entry.generation,
                }
            },
            None => {

                self.entries.push(AllocatorEntry { 
                    is_live: true,
                    generation: 0,
                });

                GenerationalIndex {
                    index: self.entries.len() - 1,
                    generation: 0,
                }
            },
        }
    }

    pub fn deallocate(&mut self, index: GenerationalIndex) -> bool {
        // make sure the entry exists.
        let idx = index.index();
        match self.entries.get_mut(idx) {
            Some(entry) => {
                if entry.is_live && entry.generation == index.generation() {
                    entry.is_live = false;
                    self.free.push(idx);
                    true
                } else {
                    false
                }
            },
            None => false,
        }
    }

    pub fn is_live(&self, index: &GenerationalIndex) -> bool {
        match self.entries.get(index.index()) {
            Some(x) => x.is_live,
            None => false,
        }
    }
}

// -------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct ArrayEntry<T> {
    value: T,
    generation: u64,
}

impl<T> ArrayEntry<T> {

    pub fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerationalIndexArray<T>(pub Vec<Option<ArrayEntry<T>>>);
impl<T> GenerationalIndexArray<T> {

    pub fn set(&mut self, index: &GenerationalIndex, value: T) {
        self.0[index.index()] = Some(ArrayEntry {
            value,
            generation: index.generation(),
        });
    }
    
    pub fn empty(&mut self, index: &GenerationalIndex) {
        self.0[index.index()] = None;
    }

    pub fn get(&self, index: &GenerationalIndex) -> Option<&T> {
        self.0.get(index.index()) // option<option<arrayentry<T>>>
            .and_then(|option| option.as_ref().map(|entry| &entry.value))
    }

    pub fn get_mut(&mut self, index: &GenerationalIndex) -> Option<&mut T> {
        self.0.get_mut(index.index()) // option<option<arrayentry<T>>>
            .and_then(|option| option.as_mut().map(|entry| &mut entry.value))
    }
}

impl<T> std::ops::Deref for GenerationalIndexArray<T> {
    type Target = Vec<Option<ArrayEntry<T>>>;
    fn deref(&self) -> &Vec<Option<ArrayEntry<T>>> {
        &self.0
    }
}

impl<T> std::ops::DerefMut for GenerationalIndexArray<T> {
    fn deref_mut(&mut self) -> &mut Vec<Option<ArrayEntry<T>>> {
        &mut self.0
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn allocator_test() {
        let mut alloc = GenerationalIndexAllocator::new();

        let idx1 = alloc.allocate();
        let idx2 = alloc.allocate();

        assert_eq!(0, idx1.index());
        assert_eq!(0, idx1.generation());
        assert_eq!(1, idx2.index());
        assert_eq!(0, idx2.generation());

        assert_eq!(true, alloc.deallocate(idx1));
        assert_eq!(false, alloc.deallocate(GenerationalIndex{generation: 0, index: 0}));
        assert_eq!(false, alloc.deallocate(GenerationalIndex{generation: 2, index: 1}));

        let idx3 = alloc.allocate();
        assert_eq!(0, idx3.index());
        assert_eq!(1, idx3.generation());

        let idx4 = alloc.allocate();
        assert_eq!(2, idx4.index());
        assert_eq!(0, idx4.generation());
    }
}