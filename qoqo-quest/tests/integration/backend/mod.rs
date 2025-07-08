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

//! Integration test for public API of Basis rotation measurement

use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::Python;
use qoqo::measurements::CheatedWrapper;
use qoqo::measurements::ClassicalRegisterWrapper;
use qoqo::measurements::{PauliZProductInputWrapper, PauliZProductWrapper};
use qoqo::noise_models::ImperfectReadoutModelWrapper;
use qoqo::CircuitWrapper;
use qoqo::QuantumProgramWrapper;
use qoqo_quest::convert_into_backend;
use qoqo_quest::BackendWrapper;
use roqoqo::measurements::Cheated;
use roqoqo::measurements::CheatedInput;
use roqoqo::measurements::ClassicalRegister;
use roqoqo::measurements::{PauliZProduct, PauliZProductInput};
use roqoqo::operations;
use roqoqo::registers::Registers;
use roqoqo::Circuit;
#[test]
fn test_creating_backend() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let backend_type = py.get_type::<BackendWrapper>();
        let _backend = backend_type
            .call1((2,))
            .unwrap()
            .downcast::<BackendWrapper>()
            .unwrap();
    });

    Python::with_gil(|py| {
        let backend_type = py.get_type::<BackendWrapper>();
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
    let mut big_circuit = Circuit::new();
    big_circuit += operations::PauliX::new(100);
    let big_circuit_wrapper = CircuitWrapper {
        internal: big_circuit,
    };

    Python::with_gil(|py| {
        let backend_type = py.get_type::<BackendWrapper>();
        let backend = backend_type.call1((3,)).unwrap();
        let _ = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_circuit", (circuit_wrapper,))
            .unwrap();
        let err = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_circuit", (big_circuit_wrapper,));
        assert!(err.is_err());
        let err = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_circuit", (backend_type,));
        assert!(err.is_err());
    })
}

#[test]
fn test_backend_seed() {
    pyo3::prepare_freethreaded_python();
    let seeds = vec![555, 555, 555, 555, 555, 555];
    Python::with_gil(|py| {
        let backend_type = py.get_type::<BackendWrapper>();
        let backend = backend_type.call1((3,)).unwrap();
        let _ = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("set_random_seed", (seeds.clone(),))
            .unwrap();
        let seeds_type = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method0("get_random_seed")
            .unwrap();
        let python_seed = seeds_type.extract::<Vec<u64>>().unwrap();
        assert_eq!(python_seed, seeds);
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
    let ch_measurement = Cheated {
        constant_circuit: None,
        circuits: vec![circuit.clone()],
        input: CheatedInput {
            measured_operators: HashMap::new(),
            number_qubits: 4,
        },
    };
    let chm_wrapper = CheatedWrapper {
        internal: ch_measurement,
    };

    let cr_measurement = ClassicalRegister {
        constant_circuit: None,
        circuits: vec![circuit],
    };
    let crm_wrapper = ClassicalRegisterWrapper {
        internal: cr_measurement,
    };
    Python::with_gil(|py| {
        let backend_type = py.get_type::<BackendWrapper>();
        let backend = backend_type.call1((3,)).unwrap();
        let _ = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_measurement", (chm_wrapper,))
            .unwrap();
        let err = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_measurement", (crm_wrapper,));
        assert!(err.is_err());
        let err = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_measurement", (backend_type,));
        assert!(err.is_err());
    })
}

#[test]
fn test_running_measurement_registers() {
    pyo3::prepare_freethreaded_python();
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("readout".to_string(), 3, true);
    circuit += operations::RotateX::new(0, 0.0.into());
    circuit += operations::RotateY::new(0, 1.0.into());
    circuit += operations::RotateZ::new(0, 2.0.into());
    circuit += operations::MolmerSorensenXX::new(0, 1);
    circuit += operations::PragmaRepeatedMeasurement::new("readout".to_string(), 100, None);
    let cr_measurement = ClassicalRegister {
        constant_circuit: Some(Circuit::new()),
        circuits: vec![circuit],
    };
    let crm_wrapper = ClassicalRegisterWrapper {
        internal: cr_measurement,
    };

    let mut circuit2 = Circuit::new();
    circuit2 += operations::DefinitionFloat::new("floats".to_owned(), 5, true);
    circuit2 += operations::DefinitionComplex::new("complexs".to_owned(), 4, true);
    circuit2 += operations::RotateX::new(0, 0.0.into());
    circuit2 += operations::RotateY::new(0, 1.0.into());
    circuit2 += operations::RotateZ::new(0, 2.0.into());
    circuit2 += operations::MolmerSorensenXX::new(0, 1);
    circuit2 += operations::PragmaGetStateVector::new("complexs".to_owned(), None);
    circuit2 += operations::PragmaGetStateVector::new("complexs".to_owned(), None);
    circuit2 += operations::PragmaGetOccupationProbability::new("floats".to_owned(), None);
    circuit2 += operations::PragmaGetOccupationProbability::new("floats".to_owned(), None);
    let ch_measurement = Cheated {
        constant_circuit: None,
        circuits: vec![circuit2],
        input: CheatedInput {
            measured_operators: HashMap::new(),
            number_qubits: 4,
        },
    };
    let chm_wrapper = CheatedWrapper {
        internal: ch_measurement,
    };
    Python::with_gil(|py| {
        let backend_type = py.get_type::<BackendWrapper>();
        let small_backend = backend_type.call1((1,)).unwrap();
        let backend = backend_type.call1((6,)).unwrap();
        let _ = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_measurement_registers", (crm_wrapper,))
            .unwrap();
        let _ = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_measurement_registers", (chm_wrapper.clone(),))
            .unwrap();
        let err = small_backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_measurement_registers", (chm_wrapper,));
        assert!(err.is_err());
        let err = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_measurement_registers", (backend_type,));
        assert!(err.is_err());
    })
}

/// Test new and run functions of QuantumProgram with all PauliZProduct measurement input
#[test]
fn test_new_run_br() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let input_type = py.get_type::<PauliZProductInputWrapper>();
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
        let br_type = py.get_type::<PauliZProductWrapper>();
        let input = br_type
            .call1((Some(CircuitWrapper::new()), circs.clone(), input_instance))
            .unwrap();

        let program_type = py.get_type::<QuantumProgramWrapper>();
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

        let backend_type = py.get_type::<BackendWrapper>();
        let backend = backend_type.call1((3,)).unwrap();

        let _result: HashMap<String, f64> = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_program", (program, vec![0.0]))
            .unwrap()
            .extract()
            .unwrap();
        let err = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_program", (backend_type, vec![0, 0]));
        assert!(err.is_err());
    })
}

#[test]
fn test_copy_deepcopy() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let backend_type = py.get_type::<BackendWrapper>();
        let backend = backend_type.call1((3,)).unwrap();
        let _copy_backend = backend.call_method0("__copy__").unwrap();
        let _deepcopy_backend = backend.call_method1("__deepcopy__", ("",)).unwrap();
    })
}

#[test]
fn test_bincode() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let backend_type = py.get_type::<BackendWrapper>();
        let backend = backend_type.call1((3,)).unwrap();
        let bytes = backend.call_method0("to_bincode").unwrap();
        let _ = backend
            .call_method1("from_bincode", (bytes.clone(),))
            .unwrap();
        let _ = backend_type.call_method1("from_bincode", (bytes,)).unwrap();

        let deserialised_error =
            backend.call_method1("from_bincode", (bincode::serialize("fails").unwrap(),));
        assert!(deserialised_error.is_err());

        let deserialised_error =
            backend.call_method1("from_bincode", (bincode::serialize(&vec![0]).unwrap(),));
        assert!(deserialised_error.is_err());

        let deserialised_error = backend.call_method0("from_bincode");
        assert!(deserialised_error.is_err());
    })
}

#[test]
fn test_json() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let backend_type = py.get_type::<BackendWrapper>();
        let backend = backend_type.call1((3,)).unwrap();
        let json = backend.call_method0("to_json").unwrap();
        let _ = backend.call_method1("from_json", (json.clone(),)).unwrap();
        let _ = backend_type.call_method1("from_json", (json,)).unwrap();

        let deserialised_error =
            backend.call_method1("from_json", (serde_json::to_string("fails").unwrap(),));
        assert!(deserialised_error.is_err());

        let deserialised_error =
            backend.call_method1("from_json", (serde_json::to_string(&vec![0]).unwrap(),));
        assert!(deserialised_error.is_err());

        let deserialised_error = backend.call_method0("from_json");
        assert!(deserialised_error.is_err());
    })
}

#[test]
fn test_convert_from_pyany() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let backend_type = py.get_type::<BackendWrapper>();
        let backend = backend_type.call1((3,)).unwrap();
        let _ = convert_into_backend(backend.as_ref()).unwrap();
        let _ = convert_into_backend(backend_type.as_ref()).is_err();
    })
}

#[test]
fn test_imperfect_model() {
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 2, true);
    circuit += operations::PauliX::new(0);
    circuit += operations::PauliX::new(1);
    circuit += operations::MeasureQubit::new(0, "ro".to_string(), 0);
    circuit += operations::MeasureQubit::new(1, "ro".to_string(), 1);
    let circuit_wrapper = CircuitWrapper { internal: circuit };

    let mut noise_model = ImperfectReadoutModelWrapper::new();
    noise_model = noise_model.set_error_probabilites(0, 0.0, 1.0).unwrap();
    noise_model = noise_model.set_error_probabilites(1, 0.0, 1.0).unwrap();

    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let backend_type = py.get_type::<BackendWrapper>();
        let backend = backend_type.call1((2,)).unwrap();

        let res = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_circuit", (circuit_wrapper.clone(),))
            .unwrap();
        assert_eq!(
            res.extract::<Registers>().unwrap().0.get("ro").unwrap(),
            &vec![vec![true, true]]
        );

        backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("set_imperfect_readout_model", (noise_model.clone(),))
            .unwrap();

        let res = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_circuit", (circuit_wrapper.clone(),))
            .unwrap();
        assert_eq!(
            res.extract::<Registers>().unwrap().0.get("ro").unwrap(),
            &vec![vec![false, false]]
        );

        assert_eq!(
            backend
                .downcast::<BackendWrapper>()
                .unwrap()
                .call_method0("get_imperfect_readout_model")
                .unwrap()
                .extract::<Option<ImperfectReadoutModelWrapper>>()
                .unwrap(),
            Some(noise_model)
        );

        backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1(
                "set_imperfect_readout_model",
                (None::<ImperfectReadoutModelWrapper>,),
            )
            .unwrap();

        let res = backend
            .downcast::<BackendWrapper>()
            .unwrap()
            .call_method1("run_circuit", (circuit_wrapper,))
            .unwrap();
        assert_eq!(
            res.extract::<Registers>().unwrap().0.get("ro").unwrap(),
            &vec![vec![true, true]]
        );
    })
}
