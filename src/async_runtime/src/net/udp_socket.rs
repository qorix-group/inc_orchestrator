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
    mio::net::udp::UdpSocketBridge,
    net::{
        utils::{resolve_as_single_address, ToSocketAddrs},
        NetResult,
    },
};
use core::net::SocketAddr;
use std::io::Error;

pub struct UdpSocket {
    socket: BridgedFd<UdpSocketBridge<AsyncSelector>>,
}

/// Represents a UDP socket.
/// Since UDP is connection-less protocol there are two ways to communicate with remotes:
///  - `send_to` and `recv_from` methods which allow to specify remote address for each packet.
///  - `connect` method which sets a default remote address for the socket. After calling `connect`, the `send` and `recv` methods can be used to communicate with the "bounded" peer.
///
/// Note that `connect` in UDP does not establish a connection like in TCP !
///
/// This class can be put freely into `Arc` and cloned to receive/send data from multiple tasks.
///
impl UdpSocket {
    ///
    /// Creates a UDP socket bound to the specified address.
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
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> NetResult<UdpSocket> {
        let sock_addr = resolve_as_single_address(addr).await?;

        // Just connect to single address as we don't support multiple addresses yet.
        let inner = UdpSocketBridge::<AsyncSelector>::bind(sock_addr)?;

        Ok(Self {
            socket: BridgedFd::new(inner)?,
        })
    }

    /// Connects the UDP socket setting the default destination for send() and limiting packets that are read via recv from the address specified in addr.
    pub async fn connect<A: ToSocketAddrs>(&self, addrs: A) -> NetResult<()> {
        let sock_addr = resolve_as_single_address(addrs).await?;
        self.socket.connect(sock_addr) // This is not blocking call for now
    }

    /// Sends data on the socket to the remote address that the socket is connected to.
    ///
    /// The connect method will connect this socket to a remote address. This method will fail if the socket is not connected.
    ///
    /// # Return
    /// On success, the number of bytes sent is returned, otherwise, the encountered error is returned.
    pub async fn send(&self, buf: &[u8]) -> NetResult<usize> {
        self.socket
            .async_call(crate::mio::types::IoEventInterest::WRITABLE, |socket| socket.send(buf))
            .await
    }

    /// Sends data on the socket to the remote address `addr`.
    ///
    /// # Return
    /// On success, the number of bytes sent is returned, otherwise, the encountered error is returned.
    ///
    pub async fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> NetResult<usize> {
        let sock_addr = resolve_as_single_address(addr).await?;

        self.socket
            .async_call(crate::mio::types::IoEventInterest::WRITABLE, |socket| socket.send_to(buf, sock_addr))
            .await
    }

    /// Receives a single datagram message on the socket from the remote address to which it is connected. On success, returns the number of bytes read.
    ///
    /// The function must be called with valid byte array buf of sufficient size to hold the message bytes. If a message is too long to fit in the supplied buffer, excess bytes may be discarded.
    ///
    /// The connect method will connect this socket to a remote address. This method will fail if the socket is not connected.
    ///
    pub async fn recv(&self, buf: &mut [u8]) -> NetResult<usize> {
        self.socket
            .async_call(crate::mio::types::IoEventInterest::READABLE, |socket| socket.recv(buf))
            .await
    }

    /// Receives a single datagram message on the socket. On success, returns the number of bytes read and the origin.
    ///
    /// The function must be called with valid byte array buf of sufficient size to hold the message bytes. If a message is too long to fit in the supplied buffer, excess bytes may be discarded.
    pub async fn recv_from(&self, buf: &mut [u8]) -> NetResult<(usize, SocketAddr)> {
        self.socket
            .async_call(crate::mio::types::IoEventInterest::READABLE, |socket| socket.recv_from(buf))
            .await
    }

    /// Returns the value of the SO_ERROR option.
    pub fn take_error(&self) -> NetResult<Option<Error>> {
        self.socket.take_error()
    }

    /// Gets the value of the IP_TTL option for this socket.
    pub fn ttl(&self) -> NetResult<u32> {
        self.socket.ttl()
    }

    /// Sets the value for the IP_TTL option on this socket.
    /// This value sets the time-to-live field that is used in every packet sent from this socket.
    pub fn set_ttl(&self, ttl: u32) -> NetResult<()> {
        self.socket.set_ttl(ttl)
    }
}
