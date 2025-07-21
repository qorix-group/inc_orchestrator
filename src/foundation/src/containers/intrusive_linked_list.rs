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

use core::marker::PhantomPinned;
use core::ptr;

/// Every [`List`] item needs to have this as one of its fields.
pub struct Link {
    prev: *mut Link,
    next: *mut Link,
    _pin: PhantomPinned,
}

impl Link {
    /// Create an unlinked link.
    pub fn new() -> Self {
        Self {
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
            _pin: PhantomPinned,
        }
    }
}

unsafe impl Send for Link {}

impl Default for Link {
    fn default() -> Self {
        Self::new()
    }
}

/// This trait signifies that an object implementing it is safe to insert into a [`List`].
///
/// # Safety
///
/// The user needs to guarantee that an object implementing the trait outlives the list it's inserted into.
pub unsafe trait Item {
    /// The offset of the [`Link`] field on the implementing object.
    fn link_field_offset() -> usize;
}

/// An intrusive linked list.
///
/// The list doesn't manage memory. It doesn't do anything on drop.
/// The [`Link`]s used by the list need to be stored in a field on the inserted objects.
/// It is the user's responsibility to ensure that the objects outlive the list.
pub struct List<I: Item> {
    // Can't have a sentinel node because the list can't manage memory.
    head: ptr::NonNull<Link>,
    len: usize,
    _phantom_data: core::marker::PhantomData<I>,
}

impl<I: Item> List<I> {
    /// Insert an item at the back of the list.
    pub fn push_back(&mut self, item: ptr::NonNull<I>) {
        unsafe {
            let mut link = self.item_to_link(item);

            if self.len == 0 {
                // If this is the first item being inserted, set it up as a sentinel.
                debug_assert_eq!(link.as_ref().prev, ptr::null_mut());
                debug_assert_eq!(link.as_ref().next, ptr::null_mut());
                link.as_mut().prev = link.as_ptr();
                link.as_mut().next = link.as_ptr();
                self.head = link;
            } else {
                Self::insert_link_before(link, self.head);
            }
        }

        self.len += 1;
    }

    /// Remove an item from the front of the list. Return the item if the list wasn't empty.
    pub fn pop_front(&mut self) -> Option<ptr::NonNull<I>> {
        if self.len > 0 {
            self.len -= 1;

            unsafe {
                let old_head = self.head;
                // Since the head is set up as a sentinel, thus points to itself,
                // these operations are safe, even if the head is the last element being removed.
                self.head = ptr::NonNull::new_unchecked(old_head.as_ref().next);
                Self::remove_link(old_head);

                Some(self.link_to_item(old_head))
            }
        } else {
            None
        }
    }

    pub fn pop_if<F>(&mut self, mut f: F)
    where
        F: FnMut(&I) -> bool,
    {
        if self.len == 0 {
            return;
        }

        let mut curr = self.head.as_ptr();
        let mut num_of_iteration = self.len;

        unsafe {
            while num_of_iteration > 0 {
                let current_link = ptr::NonNull::new_unchecked(curr);
                let next = (*curr).next;
                curr = next;

                if f(self.link_to_item(current_link).as_ref()) {
                    self.remove_internal(current_link);
                }
                num_of_iteration -= 1; // whether removed or not, we just processed this elem
            }
        }
    }

    pub fn remove(&mut self, item: ptr::NonNull<I>) -> bool {
        match self.len {
            0 => false, // Nothing to remove.
            1 => {
                let link = unsafe { self.item_to_link(item) };

                if link == self.head {
                    // If this is the last item, we can just reset the list.
                    self.head = ptr::NonNull::dangling();
                    self.len = 0;
                    true
                } else {
                    false
                }
            }
            _ => {
                // If this is not the last item, we need to remove it from the list.
                let mut curr = self.head.as_ptr();

                unsafe {
                    while !curr.is_null() {
                        let link = ptr::NonNull::new_unchecked(curr);

                        if item == self.link_to_item(link) {
                            self.remove_internal(link);
                            return true;
                        }

                        curr = (*curr).next;
                    }
                }

                false
            }
        }
    }

    unsafe fn remove_internal(&mut self, link: ptr::NonNull<Link>) {
        if link == self.head {
            if self.len == 1 {
                // If this is the last item, we can just reset the list.
                self.head = ptr::NonNull::dangling();
            } else {
                self.head = unsafe { ptr::NonNull::new_unchecked(link.as_ref().next) };
                Self::remove_link(link);
            }
        } else {
            Self::remove_link(link);
        }

        self.len -= 1;
    }

    /// Number of linked items.
    pub fn len(&self) -> usize {
        self.len
    }

    /// True if the list is empty, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    unsafe fn link_to_item(&self, link: ptr::NonNull<Link>) -> ptr::NonNull<I> {
        link.byte_sub(I::link_field_offset()).cast::<I>()
    }

    unsafe fn item_to_link(&self, item: ptr::NonNull<I>) -> ptr::NonNull<Link> {
        item.byte_add(I::link_field_offset()).cast::<Link>()
    }

    unsafe fn insert_link_before(mut link: ptr::NonNull<Link>, mut before: ptr::NonNull<Link>) {
        debug_assert_eq!(link.as_ref().prev, ptr::null_mut());
        debug_assert_eq!(link.as_ref().next, ptr::null_mut());

        link.as_mut().prev = before.as_ref().prev;
        link.as_mut().next = before.as_ptr();
        (*before.as_mut().prev).next = link.as_ptr();
        before.as_mut().prev = link.as_ptr();
    }

    unsafe fn remove_link(mut link: ptr::NonNull<Link>) {
        (*link.as_mut().prev).next = link.as_ref().next;
        (*link.as_mut().next).prev = link.as_ref().prev;

        link.as_mut().prev = ptr::null_mut();
        link.as_mut().next = ptr::null_mut();
    }
}

impl<I: Item> Default for List<I> {
    fn default() -> Self {
        Self {
            head: ptr::NonNull::dangling(),
            len: 0,
            _phantom_data: Default::default(),
        }
    }
}

unsafe impl<I: Item + Send> Send for List<I> {}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn test_push_pop() {
        // Using repr(C) to force the offset_of_link_in_item to not be 0.
        #[repr(C)]
        struct TestItem {
            value1: u64,
            link: Link,
            value2: f32,
        }

        impl Drop for TestItem {
            fn drop(&mut self) {
                println!(
                    "drop value1 {} value2 {} link.prev {:?} link.next {:?}",
                    self.value1, self.value2, self.link.prev, self.link.next
                );
            }
        }

        unsafe impl Item for TestItem {
            fn link_field_offset() -> usize {
                std::mem::offset_of!(TestItem, link)
            }
        }

        let mut item1 = TestItem {
            value1: 1,
            link: Link::new(),
            value2: 2.0,
        };

        let mut item2 = TestItem {
            value1: 5,
            link: Link::new(),
            value2: 6.0,
        };

        let mut list: List<TestItem> = Default::default();

        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
        list.push_back(ptr::NonNull::new(&mut item1 as *mut TestItem).unwrap());
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());
        list.push_back(ptr::NonNull::new(&mut item2 as *mut TestItem).unwrap());
        assert_eq!(list.len(), 2);
        assert!(!list.is_empty());

        let item1 = list.pop_front().expect("Failed to pop item1");
        assert_eq!(list.len(), 1);
        let item2 = list.pop_front().expect("Failed to pop item2");
        assert_eq!(list.len(), 0);

        unsafe {
            assert_eq!(item1.as_ref().value1, 1);
            assert_eq!(item1.as_ref().value2, 2.0);
            assert_eq!(item2.as_ref().value1, 5);
            assert_eq!(item2.as_ref().value2, 6.0);
        }

        assert!(list.pop_front().is_none());
    }

    #[test]
    fn test_drop_list() {
        let drop_counter = Rc::new(RefCell::new(0_u32));

        struct TestItem {
            link: Link,
            drop_counter: Rc<RefCell<u32>>,
        }

        unsafe impl Item for TestItem {
            fn link_field_offset() -> usize {
                std::mem::offset_of!(TestItem, link)
            }
        }

        impl Drop for TestItem {
            fn drop(&mut self) {
                *self.drop_counter.borrow_mut() += 1_u32;
            }
        }

        {
            let mut item1 = TestItem {
                drop_counter: Rc::clone(&drop_counter),
                link: Link::new(),
            };

            let mut item2 = TestItem {
                drop_counter: Rc::clone(&drop_counter),
                link: Link::new(),
            };

            let mut item3 = TestItem {
                drop_counter: Rc::clone(&drop_counter),
                link: Link::new(),
            };

            let mut list: List<TestItem> = Default::default();
            list.push_back(ptr::NonNull::new(&mut item1 as *mut TestItem).unwrap());
            list.push_back(ptr::NonNull::new(&mut item2 as *mut TestItem).unwrap());
            list.push_back(ptr::NonNull::new(&mut item3 as *mut TestItem).unwrap());
            assert_eq!(list.len(), 3);
            let _ = list.pop_front();
            assert_eq!(list.len(), 2);
            assert_eq!(*drop_counter.borrow(), 0);
        }

        assert_eq!(*drop_counter.borrow(), 3);
    }

    #[test]
    fn test_pop_if_removes_matching_items() {
        #[repr(C)]
        struct TestItem {
            value: u32,
            link: Link,
        }
        unsafe impl Item for TestItem {
            fn link_field_offset() -> usize {
                std::mem::offset_of!(TestItem, link)
            }
        }

        {
            let mut item1 = TestItem { value: 1, link: Link::new() };
            let mut item2 = TestItem { value: 2, link: Link::new() };
            let mut item3 = TestItem { value: 3, link: Link::new() };
            let mut list: List<TestItem> = Default::default();
            list.push_back(ptr::NonNull::new(&mut item1 as *mut TestItem).unwrap());
            list.push_back(ptr::NonNull::new(&mut item2 as *mut TestItem).unwrap());
            list.push_back(ptr::NonNull::new(&mut item3 as *mut TestItem).unwrap());
            // Remove all even values
            list.pop_if(|item| item.value % 2 == 0);

            assert_eq!(list.len(), 2);
            let v1 = unsafe { list.pop_front().unwrap().as_ref().value };
            let v2 = unsafe { list.pop_front().unwrap().as_ref().value };
            assert_eq!(vec![v1, v2], vec![1, 3]);
        }

        {
            let mut item1 = TestItem { value: 1, link: Link::new() };
            let mut list: List<TestItem> = Default::default();
            list.push_back(ptr::NonNull::new(&mut item1 as *mut TestItem).unwrap());

            // Remove all even values
            list.pop_if(|item| item.value % 1 == 0);

            assert_eq!(list.len(), 0);
        }
    }

    #[test]
    fn test_remove_specific_item() {
        #[repr(C)]
        struct TestItem {
            value: u32,
            link: Link,
        }
        unsafe impl Item for TestItem {
            fn link_field_offset() -> usize {
                std::mem::offset_of!(TestItem, link)
            }
        }
        let mut item1 = TestItem {
            value: 10,
            link: Link::new(),
        };
        let mut item2 = TestItem {
            value: 20,
            link: Link::new(),
        };
        let mut item3 = TestItem {
            value: 30,
            link: Link::new(),
        };
        let mut list: List<TestItem> = Default::default();
        let p1 = ptr::NonNull::new(&mut item1 as *mut TestItem).unwrap();
        let p2 = ptr::NonNull::new(&mut item2 as *mut TestItem).unwrap();
        let p3 = ptr::NonNull::new(&mut item3 as *mut TestItem).unwrap();
        list.push_back(p1);
        list.push_back(p2);
        list.push_back(p3);
        // Remove middle item
        assert!(list.remove(p2));
        assert_eq!(list.len(), 2);
        let vals = [unsafe { list.pop_front().unwrap().as_ref().value }, unsafe {
            list.pop_front().unwrap().as_ref().value
        }];
        assert_eq!(vals.contains(&10), true);
        assert_eq!(vals.contains(&30), true);
        // Remove non-existent item
        let mut item4 = TestItem {
            value: 40,
            link: Link::new(),
        };
        let p4 = ptr::NonNull::new(&mut item4 as *mut TestItem).unwrap();
        assert!(!list.remove(p4));
    }
}
