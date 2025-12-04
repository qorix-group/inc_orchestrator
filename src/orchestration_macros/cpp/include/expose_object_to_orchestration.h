/********************************************************************************
 * Copyright (c) 2025 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/
#pragma once

#include "expose_internal.h"

// Macro to expose a C++ class and selected member functions to the Rust FFI layer.
//
// Usage:
//     EXPOSE_OBJECT_TO_ORCHESTRATION(MyClass, method1, method2, ..., methodN)
//
// Parameters:
// - CLASS:        The name of the C++ class to expose (e.g., MyClass).
// - __VA_ARGS__:  A variadic list of member function names (methods of CLASS)
//                 that must have the following signature:
//                     void CLASS::method();
//                 i.e., they must return `void` and take **no arguments**.
//
// This macro expands to:
// - A factory function:     void* create_<CLASS>()
// - A destructor function:  void  free_<CLASS>(void*)
// - For each method `fn` in __VA_ARGS__:
//       void <fn>_<CLASS>(void*)
//   which internally calls: static_cast<CLASS*>(ptr)->fn();
//
// Restrictions:
// - All exposed methods must return `void` and take no parameters.
// - The macro must be used **outside of any C++ namespace**, because the
//   generated `extern "C"` symbols must have global linkage.
//
// Example:
//     class MyClass {
//     public:
//         void initialize();
//         void step();
//         void shutdown();
//     };
//
//     // This macro exposes MyClass and its methods to Rust via FFI
//     EXPOSE_OBJECT_TO_ORCHESTRATION(MyClass, initialize, step, shutdown)
#define EXPOSE_OBJECT_TO_ORCHESTRATION(CLASS, ...)                          \
    extern "C"                                                              \
    {                                                                       \
        void *create_##CLASS() { return static_cast<void *>(new CLASS()); } \
        void free_##CLASS(void *ptr) { delete static_cast<CLASS *>(ptr); }  \
        EXPAND_EXPOSE_METHODS(CLASS, __VA_ARGS__)                           \
    }
