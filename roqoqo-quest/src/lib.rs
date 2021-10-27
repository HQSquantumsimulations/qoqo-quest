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

#![deny(missing_docs)]
#![warn(private_intra_doc_links)]
#![warn(missing_crate_level_docs)]
#![warn(missing_doc_code_examples)]
#![warn(private_doc_tests)]
#![deny(missing_debug_implementations)]

//! # roqoqo-quest
//!
//! [QuEST](https://github.com/QuEST-Kit/QuEST) simulator backend for the roqoqo quantum computing toolkit.
//!
//! roqoqo-quest provides a backend to simulate roqoqo quantum circuits with the QuEST simulator

mod interface;
pub use interface::{call_circuit, call_operation};
mod backend;
pub use backend::Backend;
mod quest_bindings;
pub use quest_bindings::*;
