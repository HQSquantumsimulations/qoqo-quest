// Copyright © 2021 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

//! # quest-sys
//!
//! Rust bindings for the QuEST quantum computer simulator library.
//!
//! Conforming with the sys crate naming convention this package only provides very thin bindings.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![cfg_attr(test, allow(deref_nullptr))]

#[cfg(feature = "rebuild")]
use std::env;
use std::include;
#[cfg(feature = "openmp")]
extern crate openmp_sys;

#[cfg(feature = "rebuild")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(not(feature = "rebuild"))]
#[allow(clippy::doc_overindented_list_items)]
include!("bindings.rs");
