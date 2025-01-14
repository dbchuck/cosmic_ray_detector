use std::ptr::{read_volatile, write_volatile};

use rayon::prelude::*;

/// In order to prevent the optimizer from removing the reads of the memory that make up the detector
/// this struct will only use volatile reads and writes to its memory.
pub struct Detector {
    default: u8,
    capacity: usize,
    detector_mass: Vec<u8>,
}

impl Detector {
    pub fn new(default: u8, initial_capacity: usize) -> Self {
        Detector {
            default,
            capacity: initial_capacity,
            detector_mass: vec![default; initial_capacity],
        }
    }

    /// Checks if every element of the detector memory is equal to the default value.
    pub fn is_intact(&self) -> bool {
        !self.find_index_of_changed_element().is_some()
    }

    /// Writes the given value to every element of the detector memory.
    pub fn write(&mut self, value: u8) {
        self.detector_mass
            .par_iter_mut()
            .for_each(|n| unsafe { write_volatile(n, value) });
    }

    /// If an element in the detector does not match its default value, return it's index.
    pub fn find_index_of_changed_element(&self) -> Option<usize> {
        self.detector_mass
            .par_iter()
            .position_any(|r| unsafe { read_volatile(r) != self.default })
    }

    /// Resets the detector to its default value.
    pub fn reset(&mut self) {
        self.write(self.default);
    }

    /// Returns the value of the element at the given index, if it exists.
    pub fn get(&self, index: usize) -> Option<u8> {
        if index < self.detector_mass.len() {
            Some(unsafe { read_volatile(&self.detector_mass[index]) })
        } else {
            None
        }
    }
}
