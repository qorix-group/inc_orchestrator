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

#include <type_traits>

// Assert the member function is of signature void CLASS::FUNC()
#define ASSERT_VOID_RETURN(CLASS, FUNC)                                           \
    static_assert(std::is_same<decltype(&CLASS::FUNC), void (CLASS::*)()>::value, \
                  #FUNC " must be of signature void " #CLASS "::" #FUNC "()")

// Single method exposure
#define EXPOSE_METHOD(CLASS, FUNC)         \
    ASSERT_VOID_RETURN(CLASS, FUNC);       \
    void FUNC##_##CLASS(void *ptr)         \
    {                                      \
        static_cast<CLASS *>(ptr)->FUNC(); \
    }

// Method expansion macros for up to 10 functions, can be extended if needed.
// Due to limitations in C++17, we have to manually define macros for required number of methods.
#define EXPOSE_METHODS_1(CLASS, F1) \
    EXPOSE_METHOD(CLASS, F1)

#define EXPOSE_METHODS_2(CLASS, F1, F2) \
    EXPOSE_METHOD(CLASS, F1)            \
    EXPOSE_METHOD(CLASS, F2)

#define EXPOSE_METHODS_3(CLASS, F1, F2, F3) \
    EXPOSE_METHODS_2(CLASS, F1, F2)         \
    EXPOSE_METHOD(CLASS, F3)

#define EXPOSE_METHODS_4(CLASS, F1, F2, F3, F4) \
    EXPOSE_METHODS_3(CLASS, F1, F2, F3)         \
    EXPOSE_METHOD(CLASS, F4)

#define EXPOSE_METHODS_5(CLASS, F1, F2, F3, F4, F5) \
    EXPOSE_METHODS_4(CLASS, F1, F2, F3, F4)         \
    EXPOSE_METHOD(CLASS, F5)

#define EXPOSE_METHODS_6(CLASS, F1, F2, F3, F4, F5, F6) \
    EXPOSE_METHODS_5(CLASS, F1, F2, F3, F4, F5)         \
    EXPOSE_METHOD(CLASS, F6)

#define EXPOSE_METHODS_7(CLASS, F1, F2, F3, F4, F5, F6, F7) \
    EXPOSE_METHODS_6(CLASS, F1, F2, F3, F4, F5, F6)         \
    EXPOSE_METHOD(CLASS, F7)

#define EXPOSE_METHODS_8(CLASS, F1, F2, F3, F4, F5, F6, F7, F8) \
    EXPOSE_METHODS_7(CLASS, F1, F2, F3, F4, F5, F6, F7)         \
    EXPOSE_METHOD(CLASS, F8)

#define EXPOSE_METHODS_9(CLASS, F1, F2, F3, F4, F5, F6, F7, F8, F9) \
    EXPOSE_METHODS_8(CLASS, F1, F2, F3, F4, F5, F6, F7, F8)         \
    EXPOSE_METHOD(CLASS, F9)

#define EXPOSE_METHODS_10(CLASS, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10) \
    EXPOSE_METHODS_9(CLASS, F1, F2, F3, F4, F5, F6, F7, F8, F9)           \
    EXPOSE_METHOD(CLASS, F10)

// Macro selector
#define GET_MACRO(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, NAME, ...) NAME

#define EXPAND_EXPOSE_METHODS(CLASS, ...) \
    GET_MACRO(__VA_ARGS__,                \
              EXPOSE_METHODS_10,          \
              EXPOSE_METHODS_9,           \
              EXPOSE_METHODS_8,           \
              EXPOSE_METHODS_7,           \
              EXPOSE_METHODS_6,           \
              EXPOSE_METHODS_5,           \
              EXPOSE_METHODS_4,           \
              EXPOSE_METHODS_3,           \
              EXPOSE_METHODS_2,           \
              EXPOSE_METHODS_1)(CLASS, __VA_ARGS__)
