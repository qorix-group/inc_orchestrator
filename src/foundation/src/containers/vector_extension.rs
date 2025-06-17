//
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
//

use crate::{not_recoverable_error, prelude::*};
use iceoryx2_bb_container::vec::Vec;

/// This module provides an extension trait for iceoryx2 `Vec<T>` to add `swap_remove` and `remove` methods.
pub trait VectorExtension<T> {
    fn swap_remove(&mut self, index: usize) -> T;
    fn remove(&mut self, index: usize) -> T;
}

#[allow(dead_code)]
impl<T> VectorExtension<T> for Vec<T> {
    /// Removes the element at the specified index by swapping it with the last element
    /// and then popping the last element. If the index is out of bounds, it panics.
    fn swap_remove(&mut self, index: usize) -> T {
        let length = self.len();

        if index >= length {
            not_recoverable_error!("Index out of bounds");
        } else if index < length - 1 {
            self.swap(index, length - 1);
        }

        self.pop().unwrap()
    }

    /// Removes the element at the specified index by shifting elements to the left.
    /// If the index is out of bounds, it panics.
    fn remove(&mut self, index: usize) -> T {
        let length = self.len();
        if index >= length {
            not_recoverable_error!("Index out of bounds");
        }
        // SAFETY: We are manually handling the memory and ensuring that we do not
        // read or write out of bounds.
        unsafe {
            // Get a raw pointer to the vector's buffer
            let ptr = self.as_mut_ptr();

            // Read the element to be removed
            let removed = std::ptr::read(ptr.add(index));

            // Shift elements left
            for i in index..length - 1 {
                std::ptr::write(ptr.add(i), std::ptr::read(ptr.add(i + 1)));
            }

            // Remove the unwanted last element by popping it
            self.pop();

            removed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iceoryx2_bb_container::vec::Vec;

    #[test]
    fn swap_remove_mid_element() {
        const VEC_SIZE: usize = 10;
        let mut vec = Vec::new(VEC_SIZE);
        for i in 0..VEC_SIZE {
            vec.push(i);
        }

        // only the last element is swapped with the element at index 5
        assert_eq!(vec.swap_remove(5), 5);
        assert_eq!(vec[4], 4); // not affected
        assert_eq!(vec[5], 9); // swapped with 5
        assert_eq!(vec[6], 6); // not affected
        assert_eq!(vec.len(), VEC_SIZE - 1); // length is reduced by 1
    }

    #[test]
    fn swap_remove_first_element() {
        const VEC_SIZE: usize = 10;
        let mut vec = Vec::new(VEC_SIZE);
        for i in 0..VEC_SIZE {
            vec.push(i);
        }
        assert_eq!(vec.swap_remove(0), 0);
        assert_eq!(vec[0], 9);
        assert_eq!(vec[1], 1);
        assert_eq!(vec.len(), VEC_SIZE - 1);
    }

    #[test]
    fn swap_remove_last_element() {
        const VEC_SIZE: usize = 10;
        let mut vec = Vec::new(VEC_SIZE);
        for i in 0..VEC_SIZE {
            vec.push(i);
        }
        assert_eq!(vec.swap_remove(VEC_SIZE - 1), 9);
        assert_eq!(vec[0], 0);
        assert_eq!(vec[8], 8);
        assert_eq!(vec.len(), VEC_SIZE - 1);
    }

    #[test]
    fn swap_remove_vec_with_one_element() {
        let mut vec = Vec::new(1);
        vec.push(100);
        assert_eq!(vec.swap_remove(0), 100);
        assert_eq!(vec.len(), 0);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn swap_remove_out_of_bounds() {
        const VEC_SIZE: usize = 10;
        let mut vec = Vec::new(VEC_SIZE);
        for i in 0..VEC_SIZE {
            vec.push(i);
        }
        vec.swap_remove(VEC_SIZE); // This should panic
    }

    #[test]
    fn remove_mid_element() {
        const VEC_SIZE: usize = 10;
        let mut vec = Vec::new(VEC_SIZE);
        for i in 0..VEC_SIZE {
            vec.push(i);
        }

        // removes the element at index 5 and shifts elements to the left
        assert_eq!(vec.remove(5), 5);
        assert_eq!(vec[4], 4); // not affected
        assert_eq!(vec[5], 6); // shifted left
        assert_eq!(vec[8], 9);
        assert_eq!(vec.len(), VEC_SIZE - 1); // length is reduced by 1
    }

    #[test]
    fn remove_first_element() {
        const VEC_SIZE: usize = 10;
        let mut vec = Vec::new(VEC_SIZE);
        for i in 0..VEC_SIZE {
            vec.push(i);
        }
        assert_eq!(vec.remove(0), 0);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[8], 9);
        assert_eq!(vec.len(), VEC_SIZE - 1);
    }

    #[test]
    fn remove_last_element() {
        const VEC_SIZE: usize = 10;
        let mut vec = Vec::new(VEC_SIZE);
        for i in 0..VEC_SIZE {
            vec.push(i);
        }
        assert_eq!(vec.remove(VEC_SIZE - 1), 9);
        assert_eq!(vec[0], 0);
        assert_eq!(vec[8], 8);
        assert_eq!(vec.len(), VEC_SIZE - 1);
    }

    #[test]
    fn remove_vec_with_one_element() {
        let mut vec = Vec::new(1);
        vec.push(100);
        assert_eq!(vec.remove(0), 100);
        assert_eq!(vec.len(), 0);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn remove_out_of_bounds() {
        const VEC_SIZE: usize = 10;
        let mut vec = Vec::new(VEC_SIZE);
        for i in 0..VEC_SIZE {
            vec.push(i);
        }
        vec.remove(VEC_SIZE); // This should panic
    }
}
