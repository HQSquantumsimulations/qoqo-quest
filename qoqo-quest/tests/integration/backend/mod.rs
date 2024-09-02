// Copyright © 2021-2024 HQS Quantum Simulations GmbH. All Rights Reserved.
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

//! Integration test for public API of Basis rotation measurement

use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::Python;
use qoqo::measurements::ClassicalRegisterWrapper;
use qoqo::measurements::{PauliZProductInputWrapper, PauliZProductWrapper};
use qoqo::CircuitWrapper;
use qoqo::QuantumProgramWrapper;
use qoqo_quest::BackendWrapper;
use roqoqo::measurements::ClassicalRegister;
use roqoqo::measurements::{PauliZProduct, PauliZProductInput};
use roqoqo::operations;
use roqoqo::Circuit;
#[test]
fn test_creating_backend() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let backend_type = py.get_type_bound::<BackendWrapper>();
        let _backend = backend_type
            .call1((2,))
            .unwrap()
            .downcast::<BackendWrapper>()
            .unwrap();
    });

    Python::with_gil(|py| {
        let backend_type = py.get_type_bound::<BackendWrapper>();
        let _backend = backend_type
            .call1((2,))
            .unwrap()
            .downcast::<BackendWrapper>()
            .unwrap();
    })
}

#[test]
fn test_running_circuit() {
    pyo3::prepare_freethreaded_python();
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("readout".to_string(), 3, true);
    circuit += operations::RotateX::new(0, 0.0.into());
    circuit += operations::RotateY::new(0, 1.0.into());
    circuit += operations::RotateZ::new(0, 2.0.into());
    circuit += operations::MolmerSorensenXX::new(0, 1);
    circuit += operations::PragmaRepeatedMeasurement::new("readout".to_string(), 100, None);
    let circuit_wrapper = CircuitWrapper { internal: circuit };

    Python::with_gil(|py| {
        let backend_type = py.get_type_bound::<BackendWrapper>();
        let backend = backend_type.call1((3,)).unwrap();
        let _ = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_circuit", (circuit_wrapper,))
            .unwrap();
    })
}

#[test]
fn test_running_measurement() {
    pyo3::prepare_freethreaded_python();
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("readout".to_string(), 3, true);
    circuit += operations::RotateX::new(0, 0.0.into());
    circuit += operations::RotateY::new(0, 1.0.into());
    circuit += operations::RotateZ::new(0, 2.0.into());
    circuit += operations::MolmerSorensenXX::new(0, 1);
    circuit += operations::PragmaRepeatedMeasurement::new("readout".to_string(), 100, None);
    let cr_measurement = ClassicalRegister {
        constant_circuit: None,
        circuits: vec![circuit],
    };
    let crm_wrapper = ClassicalRegisterWrapper {
        internal: cr_measurement,
    };
    Python::with_gil(|py| {
        let backend_type = py.get_type_bound::<BackendWrapper>();
        let backend = backend_type.call1((3,)).unwrap();
        let _ = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_measurement_registers", (crm_wrapper,))
            .unwrap();
    })
}

/// Test new and run functions of QuantumProgram with all PauliZProduct measurement input
#[test]
fn test_new_run_br() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let input_type = py.get_type_bound::<PauliZProductInputWrapper>();
        let input_instance = input_type.call1((3, false)).unwrap();
        let _ = input_instance
            .downcast::<PauliZProductInputWrapper>()
            .unwrap()
            .call_method1("add_pauliz_product", ("ro", vec![0]))
            .unwrap();

        let mut circs: Vec<CircuitWrapper> = vec![CircuitWrapper::new()];
        let mut circ1 = CircuitWrapper::new();
        circ1.internal += operations::RotateX::new(0, 0.0.into());
        circ1.internal += operations::DefinitionBit::new("ro".to_string(), 1, true);
        circs.push(circ1.clone());
        let br_type = py.get_type_bound::<PauliZProductWrapper>();
        let input = br_type
            .call1((Some(CircuitWrapper::new()), circs.clone(), input_instance))
            .unwrap();

        let program_type = py.get_type_bound::<QuantumProgramWrapper>();
        let binding = program_type
            .call1((
                input.downcast::<PauliZProductWrapper>().unwrap(),
                vec!["test".to_string()],
            ))
            .unwrap();

        let program = binding.downcast::<QuantumProgramWrapper>().unwrap();
        let _program_wrapper = program.extract::<QuantumProgramWrapper>().unwrap();

        let mut bri = PauliZProductInput::new(3, false);
        let _ = bri.add_pauliz_product("ro".to_string(), vec![0]);
        let _br = PauliZProduct {
            constant_circuit: Some(Circuit::new()),
            circuits: vec![Circuit::new(), circ1.internal],
            input: bri,
        };

        let backend_type = py.get_type_bound::<BackendWrapper>();
        let backend = backend_type.call1((3,)).unwrap();

        let _result: HashMap<String, f64> = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_program", (program, vec![0.0]))
            .unwrap()
            .extract()
            .unwrap();
    })
}
