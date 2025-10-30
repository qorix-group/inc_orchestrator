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

pub(crate) type AsyncSelector = crate::mio::selector::unix::poll::Selector;
pub(crate) mod driver;
pub(crate) mod utils;

mod read_buf;
pub use read_buf::ReadBuf;

// Exported building blocks for async IO
pub mod async_registration;
pub mod bridgedfd;

mod traits;

pub use traits::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
