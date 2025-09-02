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
    pin::Pin,
    task::{Context, Poll},
};

use crate::io::async_registration::ReadinessState;

use crate::{
    io::{
        bridgedfd::BridgedFd,
        AsyncSelector, ReadBuf, {AsyncRead, AsyncWrite},
    },
    mio::{net::tcp_stream::TcpStreamBridge, types::IoEventInterest},
    net::{
        utils::{resolve_as_single_address, ToSocketAddrs},
        NetResult,
    },
};

/// Represents a TCP stream between a local and a remote socket. The `TcpStream` can be used to read and write data to the stream
/// using the `AsyncRead` and `AsyncWrite` traits. However for `async` API there are convenience methods available in
/// trait extension `AsyncReadExt` and `AsyncWriteExt` which shall be used instead.
pub struct TcpStream {
    pub(super) stream: BridgedFd<TcpStreamBridge<AsyncSelector>>,
}

impl TcpStream {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> NetResult<Self> {
        let sock_addr = resolve_as_single_address(addr).await?;
        let stream = TcpStreamBridge::<AsyncSelector>::connect(sock_addr)?;

        Ok(TcpStream {
            stream: BridgedFd::new(stream)?,
        })
    }

    /// Waits for the stream to become ready based on provided `interest`.
    pub async fn ready(&self, interest: IoEventInterest) -> NetResult<ReadinessState> {
        self.stream.ready(interest).await
    }

    /// Returns the local address that this stream is bound to.
    pub fn local_addr(&self) -> NetResult<core::net::SocketAddr> {
        self.stream.local_addr()
    }

    /// Returns the remote address that this stream is connected to.
    pub fn peer_addr(&self) -> NetResult<core::net::SocketAddr> {
        self.stream.peer_addr()
    }

    /// Gets the value of the IP_TTL option for this socket.
    pub fn ttl(&self) -> NetResult<u32> {
        self.stream.ttl()
    }

    /// Sets the value for the IP_TTL option on this socket.
    /// This value sets the time-to-live field that is used in every packet sent from this socket.
    pub fn set_ttl(&self, ttl: u32) -> NetResult<()> {
        self.stream.set_ttl(ttl)
    }

    // Does write end shutdown
    fn shutdown(&self) -> NetResult<()> {
        match (*self.stream).shutdown(std::net::Shutdown::Write) {
            Err(err) if err.kind() == std::io::ErrorKind::NotConnected => Ok(()), // racing between this call and OS, wisdom taken from Tokio impl
            result => result,
        }
    }
}

impl AsyncRead for TcpStream {
    /// # ATTENTION
    /// This will read data into provided `buf` only into INITIALIZED UNFILLED part of the buffer. User is responsible for growing initialized part if it uses MaybeUninit storage for `ReadBuf`!
    fn poll_read(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>, buf: &mut ReadBuf) -> core::task::Poll<NetResult<()>> {
        self.as_mut().stream.poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpStream {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<NetResult<usize>> {
        self.as_mut().stream.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<NetResult<()>> {
        Poll::Ready(Ok(())) // There is no flush on tcp socket
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<NetResult<()>> {
        self.shutdown()?;
        Poll::Ready(Ok(()))
    }
}
