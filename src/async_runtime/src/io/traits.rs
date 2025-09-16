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

use core::{
    pin::Pin,
    task::{Context, Poll},
};

use std::io::Error;

use core::marker::Unpin;

use crate::{io::read_buf::ReadBuf, io::utils::read_future::ReadFuture};

pub trait AsyncRead {
    ///
    /// Attempts to read into buf.
    /// On success, returns Poll::Ready(Ok(())) and places data in the unfilled portion of buf (**ATTENTION: Read specific Implementer additions to check how does this works**).
    /// If no data was read (buf.filled().len() is unchanged), it implies that EOF has been reached.
    /// If no data is available for reading, the method returns Poll::Pending and arranges for the current task (via cx.waker()) to receive a notification when the object becomes readable or is closed.
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<(), Error>>;
}

/// Extension trait for AsyncRead to provide additional methods that are `async/await` compatible.
/// This is auto implemented for all types that implement AsyncRead.
pub trait AsyncReadExt: AsyncRead {
    // Provided methods
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadFuture<'a, Self>
    where
        Self: Unpin;

    //TODO: Add more useful calls in next PRs
}

// pub struct IoSlice<'a> {}

pub trait AsyncWrite {
    /// Attempt to write bytes from buf into the object.
    /// On success, returns Poll::Ready(Ok(num_bytes_written)). If successful, then it must be guaranteed that n <= buf.len().
    /// A return value of 0 typically means that the underlying object is no longer able to accept bytes and will likely not be able to do it in the future as well, or that the buffer provided is empty.
    /// If the object is not ready for writing, the method returns Poll::Pending and arranges for the current task (via cx.waker()) to receive a notification when the object becomes writable or is closed.
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>>;

    /// Attempts to flush the object, ensuring that any buffered data reach their destination.
    /// On success, returns Poll::Ready(Ok(())).
    /// If flushing cannot immediately complete, this method returns Poll::Pending and arranges for the current task (via cx.waker()) to receive a notification when the object can make progress towards flushing.
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>>;

    /// Initiates or attempts to shut down this writer, returning success when the I/O connection has completely shut down.
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>>;

    // No vectored write support yet
}

/// Extension trait for AsyncWrite to provide additional methods that are `async/await` compatible.
pub trait AsyncWriteExt: AsyncWrite {
    //TODO: Add more useful calls in next PRs
}

// Blanket impls

impl<R> AsyncReadExt for R
where
    R: AsyncRead + ?Sized,
{
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadFuture<'a, Self>
    where
        Self: Unpin,
    {
        ReadFuture::new(self, buf)
    }
}

// Blanket impls
impl<T> AsyncRead for &mut T
where
    T: AsyncRead + Unpin + ?Sized,
{
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf) -> Poll<Result<(), Error>> {
        Pin::new(&mut **self).poll_read(cx, buf)
    }
}
