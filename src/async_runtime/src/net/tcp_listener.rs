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

use crate::{
    io::{bridgedfd::BridgedFd, AsyncSelector},
    mio::{net::tcp_listener::TcpListenerBridge, types::IoEventInterest},
    net::{
        tcp_stream::TcpStream,
        utils::{resolve_as_single_address, ToSocketAddrs},
        NetResult,
    },
};

/// Represents a TCP socket server, listening for connections.
pub struct TcpListener {
    listener: BridgedFd<TcpListenerBridge<AsyncSelector>>,
}

impl TcpListener {
    ///
    /// Creates a TCP listener bound to the specified address.
    ///
    /// Binding with a port number of 0 will request that the OS assigns a port to this listener. The port allocated can be queried via the local_addr method.
    ///
    /// The address can be any implementor of `ToSocketAddrs` trait.
    ///
    /// # ATTENTION
    /// - Currently, natively supported addresses are: `SocketAddr`, &str & String for Ipv4 and Ipv6 in forms of IP string: `192.168.0.1:80` or `[2001:db8::1]:8080`.
    ///   The resolving will happen only for single address.
    ///
    ///  The above means that current there is NO support for `resolve` via DNS.
    ///
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> NetResult<Self> {
        let sock_addr = resolve_as_single_address(addr).await?;

        let listener = TcpListenerBridge::<AsyncSelector>::bind(sock_addr)?;
        Ok(TcpListener {
            listener: BridgedFd::new(listener)?,
        })
    }

    /// Accepts a new incoming connection to this listener. When established, the corresponding TcpStream and the remote peer’s address will be returned.
    pub async fn accept(&self) -> NetResult<(TcpStream, core::net::SocketAddr)> {
        let res = self.listener.async_call(IoEventInterest::READABLE, |listener| listener.accept()).await?;

        Ok((
            TcpStream {
                stream: BridgedFd::new(res.0)?,
            },
            res.1,
        ))
    }

    /// Returns the local address that this listener is bound to.
    /// This can be useful, for example, when binding to port 0 to figure out which port was actually bound.
    pub fn local_addr(&self) -> NetResult<core::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Gets the value of the IP_TTL option for this socket.
    pub fn ttl(&self) -> NetResult<u32> {
        self.listener.ttl()
    }

    /// Sets the value for the IP_TTL option on this socket.
    /// This value sets the time-to-live field that is used in every packet sent from this socket.
    pub fn set_ttl(&self, ttl: u32) -> NetResult<()> {
        self.listener.set_ttl(ttl)
    }
}
