
use std::collections::btree_map::IterMut;
use std::collections::btree_map::Iter;
use std::collections::BTreeMap;
use std::cmp::{max, Ord};
use std::slice;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SelectionStorage<T: Clone> {
    storage: Vec<T>,
    current_selection: usize
}

impl<T: Clone> SelectionStorage<T> {
    pub fn new() -> SelectionStorage<T> {
        SelectionStorage {
            storage: Vec::new(),
            current_selection: 0
        }
    }
    
    pub fn new_from(storage: &Vec<T>) -> SelectionStorage<T> {
        SelectionStorage {
            storage: storage.clone(),
            current_selection: 0
        }
    }

    pub fn prev(&mut self) -> Option<&T> {
        if self.current_selection > 0 {
            self.current_selection -= 1;
        } else if self.storage.len() != 0 {
            self.current_selection = max(0, self.storage.len() - 1);
        }
        self.current()
    }

    pub fn next(&mut self) -> Option<&T> {
        if self.current_selection + 1 < self.storage.len() {
            self.current_selection += 1;
        } else {
            self.current_selection = 0;
        }
        self.current()
    }

    pub fn current(&self) -> Option<&T> {
        if self.storage.len() != 0 {
            self.storage.get(self.current_selection)
        } else {
            None
        }
    }

    pub fn current_mut(&mut self) -> Option<&mut T> {
        if self.storage.len() != 0 {
            self.storage.get_mut(self.current_selection)
        } else {
            None
        }
    }

    pub fn last(&mut self) -> Option<&T> {
        if self.storage.len() != 0 {
            self.current_selection = self.storage.len() - 1;
            self.storage.get(self.current_selection)
        } else {
            None
        }
    }

    pub fn extract_current(&mut self) -> Option<T> {
        if self.storage.len() != 0 {
            let item = self.storage.get(self.current_selection).unwrap().clone();
            self.storage.remove(self.current_selection);

            if self.storage.len() <= self.current_selection && self.current_selection > 0 {
                self.current_selection -= 1;
            }

            Some(item)
        } else {
            None
        }
    }

    pub fn at(&mut self, index: usize) -> Option<&T> {
    	if self.storage.len() > index {
            self.current_selection = index;
    		Some(&self.storage[index])
    	} else {
    		None
    	}
    }

    pub fn current_index(&self) -> usize {
        self.current_selection.clone()
    }

    pub fn insert(&mut self, item: T) {
        self.storage.push(item);
    }

    pub fn iter(&self) -> slice::Iter<T> {
        self.storage.iter()
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<T> {
        self.storage.iter_mut()
    }

    pub fn storage(&self) -> &Vec<T> {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut Vec<T> {
        &mut self.storage
    }

    pub fn clear(&mut self) {
        self.storage.clear()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SelectionHashMap<T: Clone + Ord> {
    storage: BTreeMap<usize, T>,
    current_selection: usize
}

impl<T: Clone + Ord> SelectionHashMap<T> {
    pub fn new() -> SelectionHashMap<T> {
        SelectionHashMap {
            storage: BTreeMap::new(),
            current_selection: 0
        }
    }

    pub fn prev(&mut self) -> Option<&T> {
        if self.current_selection > 0 {
            self.current_selection -= 1;
        } else if self.storage.len() != 0 {
            self.current_selection = max(0, self.storage.len() - 1);
        }
        self.current()
    }

    pub fn next(&mut self) -> Option<&T> {
        if self.current_selection + 1 < self.storage.len() {
            self.current_selection += 1;
        } else {
            self.current_selection = 0;
        }
        self.current()
    }

    pub fn current(&self) -> Option<&T> {
        match self.storage.iter().nth(self.current_selection) {
            Some((_, value)) => Some(value),
            None => None,
        }
    }

    pub fn current_mut(&mut self) -> Option<&mut T> {
        match self.storage.iter_mut().nth(self.current_selection) {
            Some((_, value)) => Some(value),
            None => None,
        }
    }

    pub fn last(&mut self) -> Option<&T> {
        match self.storage.iter().last() {
            Some((_, value)) => Some(value),
            None => None,
        }
    }

    pub fn at(&mut self, id: usize) {
        for (index, (key, _)) in self.storage.iter().enumerate() {
            if id == *key {
                self.current_selection = index;
            }
        }
    }

    pub fn current_index(&self) -> usize {
        self.current_selection.clone()
    }

    pub fn insert(&mut self, key: usize, item: T) {
        self.storage.insert(key, item);
    }

    pub fn iter(&self) -> Iter<usize, T> {
        self.storage.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<usize, T> {
        self.storage.iter_mut()
    }

    pub fn storage(&self) -> &BTreeMap<usize, T> {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut BTreeMap<usize, T> {
        &mut self.storage
    }

    pub fn clear(&mut self) {
        self.storage.clear()
    }
}