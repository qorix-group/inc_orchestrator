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

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use std::io::Error;

use crate::{io::AsyncRead, io::ReadBuf};

pub struct ReadFuture<'a, R: Unpin + ?Sized> {
    pub(crate) reader: &'a mut R,
    pub(crate) buf: ReadBuf<'a>,
}

impl<'a, R: AsyncRead + Unpin + ?Sized> ReadFuture<'a, R> {
    pub fn new(reader: &'a mut R, buf: &'a mut [u8]) -> Self {
        ReadFuture {
            reader,
            buf: ReadBuf::new(buf),
        }
    }
}

impl<R: AsyncRead + Unpin + ?Sized> Future for ReadFuture<'_, R> {
    type Output = Result<usize, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { reader, buf } = &mut *self;

        let before = buf.filled().len();
        Pin::new(reader).poll_read(cx, buf).map(|_| Ok(buf.filled().len() - before))
    }
}
