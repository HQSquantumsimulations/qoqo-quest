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
#![deny(missing_crate_level_docs)]
#![deny(missing_debug_implementations)]
#![allow(clippy::borrow_deref_ref)]
//! Qoqo quantum computing toolkit
//!
//! Quantum Operation Quantum Operation
//! Yes we use [reduplication](https://en.wikipedia.org/wiki/Reduplication)

use pyo3::prelude::*;
mod backend;
pub use backend::{convert_into_backend, BackendWrapper};

/// QuEST Simulator backend to the qoqo quantum computing toolkit.
///
///
/// qoqo is the HQS python package to represent quantum circuits.
///
/// .. autosummary::
///     :toctree: generated/
///
///     Backend
///
#[pymodule]
fn qoqo_quest(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<BackendWrapper>()?;
    // Adding nice imports corresponding to maturin example
    Ok(())
}
