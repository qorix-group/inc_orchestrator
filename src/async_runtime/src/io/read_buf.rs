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

// TODO: To be removed once used in IO APIs
#![allow(dead_code)]

use core::mem::MaybeUninit;

///
/// The slice abstraction for reading data into a buffer which may contain uninitialized bytes and also support reading in multiple times.
/// Used in asynchronous I/O operations to manage read buffers.
/// Layout:
///[             capacity              ]
///[ filled |         unfilled         ]
///[    initialized    | uninitialized ]
pub struct ReadBuf<'a> {
    buf: &'a mut [MaybeUninit<u8>],
    filled: usize,
    initialized: usize,
}

impl<'a> ReadBuf<'a> {
    /// Creates a new `ReadBuf` from a fully initialized byte slice.
    /// The buffer is considered fully initialized and empty (filled = 0).
    ///
    /// # Arguments
    /// * `buf` - A mutable reference to a byte slice to use as the buffer.
    ///
    pub fn new(buf: &'a mut [u8]) -> Self {
        let initialized = buf.len();
        let conv = unsafe { &mut *(buf as *mut [u8] as *mut [MaybeUninit<u8>]) };

        ReadBuf {
            buf: conv,
            filled: 0,
            initialized,
        }
    }

    /// Creates a new `ReadBuf` from a slice of uninitialized memory.
    /// The buffer is considered empty and uninitialized (filled = 0, initialized = 0).
    ///
    /// # Safety
    /// The caller must ensure that the buffer is valid for reads/writes as required.
    ///
    /// # Arguments
    /// * `buf` - A mutable reference to a slice of `MaybeUninit<u8>`.
    pub fn uninit(buf: &'a mut [MaybeUninit<u8>]) -> Self {
        ReadBuf {
            buf,
            filled: 0,
            initialized: 0,
        }
    }

    /// Returns the total capacity of the buffer.
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    /// Returns the number of bytes that can still be filled.
    pub fn remaining(&self) -> usize {
        self.capacity() - self.filled
    }

    /// Clears the buffer, setting filled to 0 but not modifying the data.
    pub fn clear(&mut self) {
        self.filled = 0;
    }

    /// Marks the first `n` bytes as initialized. This will only do anything if `n` is greater than the current initialized count.
    ///
    /// # Safety
    /// The caller must ensure that the first `n` bytes are actually initialized.
    pub unsafe fn assume_init(&mut self, n: usize) {
        if n > self.initialized {
            self.initialized = n
        }
    }

    /// Returns a mutable slice of the unfilled portion of the buffer (may be uninitialized).
    ///
    /// # Safety
    /// The caller must ensure that when writes are performed, they are paired with `assume_init` and `advance_filled`.
    pub fn unfilled_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.buf[self.filled..]
    }

    /// Returns a slice of the filled (read) portion of the buffer.
    pub fn filled(&self) -> &[u8] {
        let filled_slice = &self.buf[..self.filled];
        unsafe { Self::slice_assume_init(filled_slice) }
    }

    /// Returns a mutable slice of the filled portion of the buffer.
    pub fn filled_mut(&mut self) -> &mut [u8] {
        let filled_slice = &mut self.buf[..self.filled];
        unsafe { Self::slice_assume_init_mut(filled_slice) }
    }

    /// Returns a slice of the initialized portion of the buffer.
    pub fn initialized(&self) -> &[u8] {
        let initialized_slice = &self.buf[..self.initialized];
        unsafe { Self::slice_assume_init(initialized_slice) }
    }

    /// Returns a mutable slice of the initialized portion of the buffer.
    pub fn initialized_mut(&mut self) -> &mut [u8] {
        let initialized_slice = &mut self.buf[..self.initialized];
        unsafe { Self::slice_assume_init_mut(initialized_slice) }
    }

    /// Sets the number of filled bytes.
    ///
    /// # Panics
    /// Panics if `n` is greater than the number of initialized bytes.
    pub fn set_filled(&mut self, n: usize) {
        assert!(n <= self.initialized, "Cannot set filled more than initialized");
        self.filled = n;
    }

    /// Advances the filled count by `n` bytes.
    ///
    /// # Panics
    /// Panics if this would exceed the number of initialized bytes.
    pub fn advance_filled(&mut self, n: usize) {
        let value = self.filled.checked_add(n).expect("Overflow in advance_filled");
        self.filled = value;
    }

    /// Copies a slice into the buffer at the filled position, advancing filled and initialized as needed.
    ///
    /// # Panics
    /// Panics if the slice would exceed the buffer's capacity.
    pub fn put_slice(&mut self, buf: &[u8]) {
        let len = buf.len();
        assert!(self.remaining() >= len, "Cannot put slice more than capacity");

        // Above assert ensures no overflow
        let filled_slice_uninit = &mut self.buf[self.filled..self.filled + len];

        unsafe { filled_slice_uninit.as_mut_ptr().copy_from_nonoverlapping(buf.as_ptr().cast(), buf.len()) };

        self.filled += len;

        if self.initialized < self.filled {
            self.initialized = self.filled;
        }
    }

    /// Marks the unfilled portion as initialized and returns it as a mutable slice.
    pub fn initialize_unfilled(&mut self) -> &mut [u8] {
        self.init_unfilled(self.remaining());
        let unfilled_slice = &mut self.buf[self.filled..];

        unsafe { Self::slice_assume_init_mut(unfilled_slice) }
    }

    /// Returns a slice of the initialized but unfilled portion of the buffer.
    pub fn initialized_unfilled(&self) -> &[u8] {
        let unfilled_slice = &self.buf[self.filled..self.initialized];
        unsafe { Self::slice_assume_init(unfilled_slice) }
    }

    /// Returns a slice of the initialized but unfilled portion of the buffer.
    pub fn initialized_unfilled_mut(&mut self) -> &mut [u8] {
        let unfilled_slice = &mut self.buf[self.filled..self.initialized];
        unsafe { Self::slice_assume_init_mut(unfilled_slice) }
    }

    // Private helpers
    unsafe fn slice_assume_init(slice: &[MaybeUninit<u8>]) -> &[u8] {
        core::slice::from_raw_parts(slice.as_ptr().cast::<u8>(), slice.len())
    }

    unsafe fn slice_assume_init_mut(slice: &mut [MaybeUninit<u8>]) -> &mut [u8] {
        core::slice::from_raw_parts_mut(slice.as_mut_ptr().cast::<u8>(), slice.len())
    }

    ///
    /// # Panics
    ///  Panics if `size` is greater than the remaining capacity.
    fn init_unfilled(&mut self, size: usize) {
        assert!(self.remaining() >= size, "Cannot fill unfilled more than capacity");

        // Above assert ensures no overflow
        let end = self.filled + size;

        let unfilled_slice = &mut self.buf[self.filled..end];

        unsafe { unfilled_slice.as_mut_ptr().write_bytes(0, size) };

        if end > self.initialized {
            self.initialized = end;
        }
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use std::mem::MaybeUninit;

    #[test]
    fn new_and_capacity() {
        let mut buf = [0u8; 8];
        let read_buf = ReadBuf::new(&mut buf);
        assert_eq!(read_buf.capacity(), 8);
        assert_eq!(read_buf.filled().len(), 0);
        assert_eq!(read_buf.initialized().len(), 8);
    }

    #[test]
    fn uninit_and_capacity() {
        let mut buf: [MaybeUninit<u8>; 8] = unsafe { MaybeUninit::uninit().assume_init() };
        let read_buf = ReadBuf::uninit(&mut buf);
        assert_eq!(read_buf.capacity(), 8);
        assert_eq!(read_buf.filled().len(), 0);
        assert_eq!(read_buf.initialized().len(), 0);
    }

    #[test]
    fn put_slice_and_set_filled() {
        let mut buf = [0u8; 8];
        let mut read_buf = ReadBuf::new(&mut buf);
        read_buf.put_slice(&[1, 2, 3]);
        assert_eq!(read_buf.filled(), &[1, 2, 3]);
        assert_eq!(read_buf.initialized()[..3], [1, 2, 3]);
        read_buf.set_filled(2);
        assert_eq!(read_buf.filled(), &[1, 2]);
    }

    #[test]
    #[should_panic(expected = "Cannot set filled more than initialized")]
    fn set_filled_overflow() {
        let mut buf = [0u8; 4];
        let mut read_buf = ReadBuf::new(&mut buf);
        read_buf.set_filled(5); // should panic
    }

    #[test]
    #[should_panic(expected = "Cannot put slice more than capacity")]
    fn put_slice_overflow() {
        let mut buf = [0u8; 4];
        let mut read_buf = ReadBuf::new(&mut buf);
        read_buf.put_slice(&[1, 2, 3, 4, 5]); // should panic
    }

    #[test]
    fn initialized_unfilled() {
        let mut buf = [0u8; 8];
        let mut read_buf = ReadBuf::new(&mut buf);
        read_buf.put_slice(&[1, 2, 3]);
        let unfilled = read_buf.initialized_unfilled();
        assert_eq!(unfilled.len(), 5);
    }

    #[test]
    fn filled_and_initialized_mut() {
        let mut buf = [0u8; 4];
        let mut read_buf = ReadBuf::new(&mut buf);
        {
            let filled_mut = read_buf.filled_mut();
            assert_eq!(filled_mut.len(), 0);
        }
        {
            let initialized_mut = read_buf.initialized_mut();
            assert_eq!(initialized_mut.len(), 4);
        }
    }

    #[test]
    fn zero_capacity() {
        let mut buf = [0u8; 0];
        let read_buf = ReadBuf::new(&mut buf);
        assert_eq!(read_buf.capacity(), 0);
        assert_eq!(read_buf.filled().len(), 0);
        assert_eq!(read_buf.initialized().len(), 0);
        // put_slice and set_filled should panic
        let put_result = std::panic::catch_unwind(|| {
            let mut read_buf = ReadBuf::new(&mut [0u8; 0]);
            read_buf.put_slice(&[1]);
        });
        assert!(put_result.is_err());
        let set_result = std::panic::catch_unwind(|| {
            let mut read_buf = ReadBuf::new(&mut [0u8; 0]);
            read_buf.set_filled(1);
        });
        assert!(set_result.is_err());
    }

    #[test]
    fn slice_assume_init_and_mut() {
        // Test the private helpers via public API
        let mut buf = [0u8; 4];
        let mut read_buf = ReadBuf::new(&mut buf);
        read_buf.put_slice(&[10, 20]);
        // filled() uses slice_assume_init
        assert_eq!(read_buf.filled(), &[10, 20]);
        // filled_mut() uses slice_assume_init_mut
        let filled_mut = read_buf.filled_mut();
        filled_mut[0] = 99;
        assert_eq!(read_buf.filled(), &[99, 20]);
        // initialized() uses slice_assume_init
        assert_eq!(read_buf.initialized()[..2], [99, 20]);
        // initialized_mut() uses slice_assume_init_mut
        let initialized_mut = read_buf.initialized_mut();
        initialized_mut[2..].copy_from_slice(&[1, 2]);
        assert_eq!(read_buf.initialized(), &[99, 20, 1, 2]);
    }

    #[test]
    fn initialized_unfilled_empty() {
        let mut buf = [0u8; 4];
        let mut read_buf = ReadBuf::new(&mut buf);
        // No put_slice, so filled == 0, initialized == 4
        let unfilled = read_buf.initialized_unfilled();
        assert_eq!(unfilled.len(), 4);
        // Fill all, then unfilled should be empty
        read_buf.put_slice(&[1, 2, 3, 4]);
        let unfilled = read_buf.initialized_unfilled();
        assert_eq!(unfilled.len(), 0);
    }

    #[test]
    fn put_slice_multiple_times() {
        let mut buf = [0u8; 6];
        let mut read_buf = ReadBuf::new(&mut buf);
        read_buf.put_slice(&[1, 2]);
        read_buf.put_slice(&[3, 4]);
        assert_eq!(read_buf.filled(), &[1, 2, 3, 4]);
        assert_eq!(read_buf.initialized()[..4], [1, 2, 3, 4]);
        read_buf.put_slice(&[5, 6]);
        assert_eq!(read_buf.filled(), &[1, 2, 3, 4, 5, 6]);
        assert_eq!(read_buf.initialized(), &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn initialize_unfilled() {
        let mut buf = [0u8; 6];
        let mut read_buf = ReadBuf::new(&mut buf);
        read_buf.put_slice(&[1, 2, 3]);
        let unfilled = read_buf.initialize_unfilled();
        // All unfilled bytes should now be considered initialized
        assert_eq!(unfilled.len(), 3);
        // Write to the unfilled region
        unfilled.copy_from_slice(&[4, 5, 6]);
        // Now the initialized region should be the whole buffer
        assert_eq!(read_buf.initialized(), &[1, 2, 3, 4, 5, 6]);
    }
}
