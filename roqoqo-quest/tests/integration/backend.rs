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

use num_complex::Complex64;
use roqoqo::backends::EvaluatingBackend;
use roqoqo::devices::AllToAllDevice;
use roqoqo::devices::Device;
use roqoqo::measurements::ClassicalRegister;
use roqoqo::operations;
use roqoqo::operations::DefinitionBit;
use roqoqo::operations::DefinitionComplex;
use roqoqo::operations::MeasureQubit;
use roqoqo::operations::MultiQubitZZ;
use roqoqo::operations::PragmaGetStateVector;
use roqoqo::operations::PragmaGlobalPhase;
use roqoqo::operations::PragmaRandomNoise;
use roqoqo::operations::PragmaRepeatedMeasurement;
use roqoqo::operations::PragmaSetNumberOfMeasurements;
use roqoqo::operations::RotateX;
use roqoqo::operations::RotateZ;
use roqoqo::operations::Toffoli;
use roqoqo::operations::CNOT;
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

#[test]
fn test_failing_two_set_measurments() {
    let mut circuit = Circuit::new();
    circuit += DefinitionBit::new("ro".to_string(), 1, true);
    circuit += MeasureQubit::new(0, "ro".to_string(), 0);
    circuit += PragmaSetNumberOfMeasurements::new(10, "ro".to_string());
    circuit += PragmaSetNumberOfMeasurements::new(20, "ro".to_string());
    let backend = Backend::new(1);
    let res = backend.run_circuit(&circuit);
    assert!(res.is_err());
}

#[test]
fn test_set_repetitions() {
    let backend = Backend::new(1);
    assert_eq!(backend.repetitions, 1);
    let backend = backend.set_repetitions(10);
    assert_eq!(backend.repetitions, 10);
    let mut circuit = Circuit::new();
    circuit += DefinitionComplex::new("ro".to_string(), 2, true);
    circuit += PragmaRandomNoise::new(0, 1.0.into(), 1.0.into(), 0.0.into());
    circuit += PragmaGetStateVector::new("ro".to_string(), None);
    let (_, _, complex_res) = backend.run_circuit(&circuit).unwrap();
    assert_eq!(complex_res.get("ro").unwrap().len(), 10);
}

#[test]
fn test_running_with_device() {
    let backend = Backend::new(3);
    let device = AllToAllDevice::new(3, &["RotateZ".to_string()], &["CNOT".to_string()], 1.0);
    let mut circuit = Circuit::new();
    circuit += RotateX::new(0, 0.1.into());
    circuit += CNOT::new(0, 1);
    circuit += Toffoli::new(0, 1, 2);
    let mut dev: Option<Box<dyn Device>> = Some(Box::new(device.clone()));
    let res = backend.run_circuit_iterator_with_device(circuit.iter(), &mut dev);
    assert!(res.is_err());
    let mut circuit = Circuit::new();
    circuit += RotateZ::new(0, 0.1.into());
    circuit += CNOT::new(0, 10);
    circuit += Toffoli::new(0, 1, 2);
    let mut dev: Option<Box<dyn Device>> = Some(Box::new(device.clone()));
    let res = backend.run_circuit_iterator_with_device(circuit.iter(), &mut dev);
    assert!(res.is_err());
    let mut circuit = Circuit::new();
    circuit += CNOT::new(0, 1);
    circuit += Toffoli::new(0, 1, 2);
    let mut dev: Option<Box<dyn Device>> = Some(Box::new(device.clone()));
    let res = backend.run_circuit_iterator_with_device(circuit.iter(), &mut dev);
    assert!(res.is_err());
    let mut circuit = Circuit::new();
    circuit += MultiQubitZZ::new(vec![0, 1, 2], 0.1.into());
    let mut dev: Option<Box<dyn Device>> = Some(Box::new(device.clone()));
    let res = backend.run_circuit_iterator_with_device(circuit.iter(), &mut dev);
    assert!(res.is_err());
}

#[test]
fn test_global_phase() {
    let mut circuit = Circuit::new();
    circuit += DefinitionComplex::new("ro".to_string(), 2, true);
    circuit += PragmaGlobalPhase::new(0.1.into());
    circuit += PragmaGetStateVector::new("ro".to_string(), None);
    let backend = Backend::new(1);
    let (_, _, complex_res) = backend.run_circuit(&circuit).unwrap();
    assert_eq!(
        complex_res.get("ro").unwrap()[0][0],
        Complex64::new(1.0, 0.0)
    );
}

#[test]
fn test_failing_two_repeated_measurments() {
    let mut circuit = Circuit::new();
    circuit += DefinitionBit::new("ro".to_string(), 1, true);
    circuit += MeasureQubit::new(0, "ro".to_string(), 0);
    circuit += PragmaSetNumberOfMeasurements::new(10, "ro".to_string());
    circuit += PragmaRepeatedMeasurement::new("ro".to_string(), 20, None);
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
