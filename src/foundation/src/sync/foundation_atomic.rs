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
use loom::sync::atomic::*;

#[cfg(not(loom))]
use ::core::sync::atomic::*;

/// Behaves like [`sync::atomic::AtomicBool`]
pub type FoundationAtomicBool = AtomicBool;

/// Behaves like [`sync::atomic::AtomicUsize`]
pub type FoundationAtomicUsize = AtomicUsize;

/// Behaves like [`sync::atomic::AtomicIsize`]
pub type FoundationAtomicIsize = AtomicIsize;

/// Behaves like [`sync::atomic::AtomicU8`]
pub type FoundationAtomicU8 = AtomicU8;

/// Behaves like [`sync::atomic::AtomicU16`]
pub type FoundationAtomicU16 = AtomicU16;

/// Behaves like [`sync::atomic::AtomicU32`]
pub type FoundationAtomicU32 = AtomicU32;

/// Behaves like [`sync::atomic::AtomicI8`]
pub type FoundationAtomicI8 = AtomicI8;

/// Behaves like [`sync::atomic::AtomicI16`]
pub type FoundationAtomicI16 = AtomicI16;

/// Behaves like [`sync::atomic::AtomicI32`]
pub type FoundationAtomicI32 = AtomicI32;

/// Behaves like [`sync::atomic::AtomicI64`]
pub type FoundationAtomicI64 = AtomicI64;

/// Behaves like [`sync::atomic::AtomicU64`]
pub type FoundationAtomicU64 = AtomicU64;

/// Behaves like [`sync::atomic::AtomicU64`]
pub type FoundationAtomicPtr<T> = AtomicPtr<T>;

/// Behaves like [`sync::atomic::Ordering`]
pub type FoundationOrdering = Ordering;
