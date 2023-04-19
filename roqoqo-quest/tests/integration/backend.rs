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

use roqoqo::backends::EvaluatingBackend;
use roqoqo::measurements::ClassicalRegister;
use roqoqo::operations;
use roqoqo::operations::DefinitionBit;
use roqoqo::operations::MeasureQubit;
use roqoqo::operations::PragmaSetNumberOfMeasurements;
use roqoqo::Circuit;
use roqoqo_quest::Backend;

#[cfg(feature = "async")]
use futures::executor::block_on;
#[cfg(feature = "async")]
use roqoqo::backends::AsyncEvaluatingBackend;

#[test]
fn test_measurement_with_repeated_measurement() {
    let mut constant_circuit = Circuit::new();
    constant_circuit += operations::PauliX::new(1);
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 4, true);
    circuit += operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);
    let measurement = ClassicalRegister {
        constant_circuit: Some(constant_circuit),
        circuits: vec![circuit],
    };
    let backend = Backend::new(4);
    let (bit_result, float_result, complex_result) =
        backend.run_measurement_registers(&measurement).unwrap();
    assert!(float_result.is_empty());
    assert!(complex_result.is_empty());
    assert!(bit_result.contains_key("ro"));
    let nested_vec = bit_result.get("ro").unwrap();
    assert!(nested_vec.len() == 10);
    for repetition in nested_vec {
        assert!(repetition.len() == 4);
        assert!(!repetition[0]);
        assert!(repetition[1]);
        assert!(!repetition[2]);
    }
}

#[test]
fn test_failing_set_number_of_measurments() {
    let mut circuit = Circuit::new();
    circuit += DefinitionBit::new("ro".to_string(), 1, true);
    circuit += MeasureQubit::new(0, "ro".to_string(), 0);
    circuit += PragmaSetNumberOfMeasurements::new(10, "ro_misspelled".to_string());
    let backend = Backend::new(1);
    let res = backend.run_circuit(&circuit);
    assert!(res.is_err());
}

#[cfg(feature = "async")]
#[test]
fn test_measurement_with_repeated_measurement_async() {
    let mut constant_circuit = Circuit::new();
    constant_circuit += operations::PauliX::new(1);
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 4, true);
    circuit += operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);
    let measurement = ClassicalRegister {
        constant_circuit: Some(constant_circuit),
        circuits: vec![circuit],
    };
    let backend = Backend::new(4);
    let (bit_result, float_result, complex_result) =
        block_on(backend.async_run_measurement_registers(&measurement)).unwrap();
    assert!(float_result.is_empty());
    assert!(complex_result.is_empty());
    assert!(bit_result.contains_key("ro"));
    let nested_vec = bit_result.get("ro").unwrap();
    assert!(nested_vec.len() == 10);
    for repetition in nested_vec {
        assert!(repetition.len() == 4);
        assert!(!repetition[0]);
        assert!(repetition[1]);
        assert!(!repetition[2]);
    }
}

#[test]
fn test_circuit_with_repeated_measurement() {
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 4, true);
    circuit += operations::PauliX::new(1);
    circuit += operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);
    let backend = Backend::new(4);
    let (bit_result, float_result, complex_result) =
        backend.run_circuit_iterator(circuit.iter()).unwrap();
    assert!(float_result.is_empty());
    assert!(complex_result.is_empty());
    assert!(bit_result.contains_key("ro"));
    let nested_vec = bit_result.get("ro").unwrap();
    assert!(nested_vec.len() == 10);
    for repetition in nested_vec {
        assert!(repetition.len() == 4);
        assert!(!repetition[0]);
        assert!(repetition[1]);
        assert!(!repetition[2]);
    }
}

#[test]
fn test_circuit_with_repeated_measurement_and_previous_measurement() {
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 2, true);
    circuit += operations::PauliX::new(0);
    circuit += operations::MeasureQubit::new(0, "ro".to_string(), 1);
    circuit += operations::PauliX::new(0);
    circuit += operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);
    let backend = Backend::new(1);
    let (bit_result, float_result, complex_result) =
        backend.run_circuit_iterator(circuit.iter()).unwrap();
    assert!(float_result.is_empty());
    assert!(complex_result.is_empty());
    assert!(bit_result.contains_key("ro"));
    let nested_vec = bit_result.get("ro").unwrap();
    assert!(nested_vec.len() == 10);
    for repetition in nested_vec {
        assert!(repetition.len() == 2);
        assert!(!repetition[0]);
        assert!(repetition[1]);
    }
}

#[test]
fn test_circuit_with_repeated_measurement_and_input() {
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 2, true);
    circuit += operations::InputBit::new("ro".to_string(), 1, true);
    circuit += operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);
    let backend = Backend::new(1);
    let (bit_result, float_result, complex_result) =
        backend.run_circuit_iterator(circuit.iter()).unwrap();
    assert!(float_result.is_empty());
    assert!(complex_result.is_empty());
    assert!(bit_result.contains_key("ro"));
    let nested_vec = bit_result.get("ro").unwrap();
    assert!(nested_vec.len() == 10);
    for repetition in nested_vec {
        assert!(repetition.len() == 2);
        assert!(!repetition[0]);
        assert!(repetition[1]);
    }
}

#[test]
fn test_circuit_with_set_measurement_number() {
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 4, true);
    circuit += operations::PauliX::new(1);
    circuit += operations::PauliX::new(5);
    circuit += operations::MeasureQubit::new(0, "ro".to_string(), 0);
    circuit += operations::MeasureQubit::new(1, "ro".to_string(), 1);
    circuit += operations::MeasureQubit::new(5, "ro".to_string(), 3);
    circuit += operations::PauliX::new(1);
    circuit += operations::PauliX::new(2);
    circuit += operations::MeasureQubit::new(2, "ro".to_string(), 2);
    circuit += operations::PragmaSetNumberOfMeasurements::new(2, "ro".to_string());
    let backend = Backend::new(6);
    let (bit_result, float_result, complex_result) =
        backend.run_circuit_iterator(circuit.iter()).unwrap();
    assert!(float_result.is_empty());
    assert!(complex_result.is_empty());
    assert!(bit_result.contains_key("ro"));
    let nested_vec = bit_result.get("ro").unwrap();
    assert!(nested_vec.len() == 2);
    for repetition in nested_vec {
        assert!(repetition.len() == 4);
        assert!(!repetition[0]);
        assert!(repetition[1]);
        assert!(repetition[2]);
        assert!(repetition[3]);
    }
}

#[cfg(feature = "async")]
#[test]
fn test_circuit_with_repeated_measurement_async() {
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 4, true);
    circuit += operations::PauliX::new(1);
    circuit += operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);
    let backend = Backend::new(4);
    let (bit_result, float_result, complex_result) =
        block_on(backend.async_run_circuit_iterator(circuit.iter())).unwrap();
    assert!(float_result.is_empty());
    assert!(complex_result.is_empty());
    assert!(bit_result.contains_key("ro"));
    let nested_vec = bit_result.get("ro").unwrap();
    assert!(nested_vec.len() == 10);
    for repetition in nested_vec {
        assert!(repetition.len() == 4);
        assert!(!repetition[0]);
        assert!(repetition[1]);
        assert!(!repetition[2]);
    }
}
