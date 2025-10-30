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

use crate::io::{
    read_buf::ReadBuf,
    utils::{read_future::ReadFuture, write_future::WriteFuture},
};
use core::{
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};
use std::io::Error;

pub trait AsyncRead {
    /// Attempts to read into buf.
    /// On success, returns Poll::Ready(Ok(())) and places data in the unfilled portion of buf (**ATTENTION: Read specific Implementer additions to check how does this works**).
    /// If no data was read (buf.filled().len() is unchanged), it implies that EOF has been reached.
    /// If no data is available for reading, the method returns Poll::Pending and arranges for the current task (via cx.waker()) to receive a notification when the object becomes readable or is closed.
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<(), Error>>;
}

/// Extension trait for AsyncRead to provide additional methods that are `async/await` compatible.
/// This is auto implemented for all types that implement AsyncRead.
pub trait AsyncReadExt: AsyncRead {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadFuture<'a, Self>
    where
        Self: Unpin;
}

// pub struct IoSlice<'a> {}

pub trait AsyncWrite {
    /// Attempts to write a buffer into this writer.
    ///
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
    /// Attempts to write a buffer into this writer.
    ///
    /// There's no guarantee that the whole buffer will be written, and such case is not considered an error.
    /// The result of the returned future is the same as for [`AsyncWrite::poll_write`].
    fn write<'a>(&'a mut self, src: &'a [u8]) -> WriteFuture<'a, Self>
    where
        Self: Unpin;
}

// Blanket impls

impl<T: AsyncRead + Unpin + ?Sized> AsyncRead for &mut T {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf) -> Poll<Result<(), Error>> {
        Pin::new(&mut **self).poll_read(cx, buf)
    }
}

impl<T: AsyncRead + ?Sized> AsyncReadExt for T {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadFuture<'a, Self>
    where
        Self: Unpin,
    {
        ReadFuture::new(self, buf)
    }
}

impl<T: AsyncWrite + Unpin + ?Sized> AsyncWrite for &mut T {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        Pin::new(&mut **self).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut **self).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut **self).poll_shutdown(cx)
    }
}

impl<T: AsyncWrite + ?Sized> AsyncWriteExt for T {
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> WriteFuture<'a, Self>
    where
        Self: Unpin,
    {
        WriteFuture::new(self, buf)
    }
}
