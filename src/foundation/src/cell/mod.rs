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

#[cfg(loom)]
pub use loom::cell::UnsafeCell;

#[cfg(not(loom))]
#[derive(Debug)]
pub struct UnsafeCell<T>(::core::cell::UnsafeCell<T>);

#[cfg(not(loom))]
impl<T> UnsafeCell<T> {
    pub fn new(data: T) -> UnsafeCell<T> {
        UnsafeCell(::core::cell::UnsafeCell::new(data))
    }

    pub fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
        f(self.0.get())
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
        f(self.0.get())
    }
}

/// Internal bridge to connect normal cell and loom cell
pub trait UnsafeCellExt<T> {
    ///
    ///
    /// # Safety
    ///   This method is unsafe because it allows mutable access to the data without borrow checker
    ///
    ///
    unsafe fn get_access(&self) -> *mut T;
}

#[cfg(not(loom))]
impl<T> UnsafeCellExt<T> for UnsafeCell<T> {
    unsafe fn get_access(&self) -> *mut T {
        &mut *self.0.get()
    }
}

#[cfg(loom)]
impl<T> UnsafeCellExt<T> for UnsafeCell<T> {
    unsafe fn get_access(&self) -> *mut T {
        panic!("UnsafeCellExt::get_mut_access is not implemented for loom::cell::UnsafeCell");
    }
}
