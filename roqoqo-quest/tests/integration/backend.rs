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

use std::collections::HashMap;

use num_complex::Complex64;
use roqoqo::backends::EvaluatingBackend;
use roqoqo::devices::AllToAllDevice;
use roqoqo::devices::Device;
use roqoqo::measurements::ClassicalRegister;
use roqoqo::measurements::PauliProductsToExpVal;
use roqoqo::operations;
use rusty_fork::rusty_fork_test;

use roqoqo::operations::*;
use roqoqo::prelude::Measure;
use roqoqo::Circuit;
use roqoqo::RoqoqoBackendError;
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
    let mut circuit1: Circuit = Circuit::new();
    circuit1 += operations::DefinitionBit::new("ro1".to_string(), 4, true);
    circuit1 += operations::PragmaRepeatedMeasurement::new("ro1".to_string(), 10, None);
    let mut circuit2 = Circuit::new();
    circuit2 += operations::DefinitionBit::new("ro2".to_string(), 4, true);
    circuit2 += operations::PragmaRepeatedMeasurement::new("ro2".to_string(), 10, None);

    let measurement = ClassicalRegister {
        constant_circuit: Some(constant_circuit),
        circuits: vec![circuit, circuit1, circuit2],
    };
    let backend = Backend::new(4, None);
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
    let backend = Backend::new(1, None);
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
    let backend = Backend::new(1, None);
    let res = backend.run_circuit(&circuit);
    assert!(res.is_err());
}

#[test]
fn test_set_repetitions() {
    let backend = Backend::new(2, None);
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
fn test_readout_into_partial_register() {
    let backend = Backend::new(6, None);
    let mut circuit = Circuit::new();
    circuit += DefinitionBit::new("ro_0".to_string(), 6, true);
    circuit += Hadamard::new(0);
    circuit += Hadamard::new(1);
    circuit += Hadamard::new(2);
    circuit += MeasureQubit::new(0, "ro_0".to_string(), 3);
    circuit += MeasureQubit::new(1, "ro_0".to_string(), 4);
    circuit += MeasureQubit::new(1, "ro_0".to_string(), 5);
    circuit += PragmaRepeatedMeasurement::new("ro_0".to_string(), 10, None);
    let (bit_res, _, _) = backend.run_circuit(&circuit).unwrap();
    assert!(bit_res.contains_key("ro_0"));
    let bit_vec_of_vecs = bit_res.get("ro_0").unwrap();
    assert_eq!(bit_vec_of_vecs.len(), 10);
    for bit_vec in bit_vec_of_vecs {
        assert_eq!(bit_vec.len(), 6);
    }
}

#[test]
fn test_readout_into_partial_register_set_number_measurements() {
    let backend = Backend::new(6, None);
    let mut circuit = Circuit::new();
    circuit += DefinitionBit::new("ro_0".to_string(), 6, true);
    circuit += PauliX::new(0);
    circuit += PauliX::new(1);
    circuit += PauliX::new(2);
    circuit += MeasureQubit::new(0, "ro_0".to_string(), 3);
    circuit += MeasureQubit::new(1, "ro_0".to_string(), 4);
    circuit += MeasureQubit::new(1, "ro_0".to_string(), 5);
    circuit += PragmaSetNumberOfMeasurements::new(10, "ro_0".to_string());
    let (bit_res, _, _) = backend.run_circuit(&circuit).unwrap();
    assert!(bit_res.contains_key("ro_0"));
    let bit_vec_of_vecs = bit_res.get("ro_0").unwrap();
    assert_eq!(bit_vec_of_vecs.len(), 10);
    for bit_vec in bit_vec_of_vecs {
        assert_eq!(bit_vec.len(), 6);
        assert_eq!(bit_vec, &vec![false, false, false, true, true, true]);
    }
}

#[test]
fn test_failing_set_number_measurements() {
    let backend = Backend::new(6, None);
    let mut circuit = Circuit::new();
    circuit += DefinitionBit::new("ro_0".to_string(), 6, true);
    circuit += Hadamard::new(0);
    circuit += Hadamard::new(1);
    circuit += Hadamard::new(2);
    circuit += MeasureQubit::new(1, "ro_1".to_string(), 4);
    circuit += PragmaSetNumberOfMeasurements::new(10, "ro_0".to_string());
    let res = backend.run_circuit(&circuit);
    assert!(res.is_err());
}

#[test]
fn test_float_registry() {
    let backend = Backend::new(1, None);
    let mut circuit = Circuit::new();
    circuit += DefinitionFloat::new("ro_f".to_string(), 1, true);

    let (_, float_res, _) = backend.run_circuit(&circuit).unwrap();

    assert!(float_res.contains_key("ro_f"));
}

#[test]
fn test_running_with_device() {
    let backend = Backend::new(3, None);
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
    let mut dev: Option<Box<dyn Device>> = Some(Box::new(device));
    let res = backend.run_circuit_iterator_with_device(circuit.iter(), &mut dev);
    assert!(res.is_err());
}

#[test]
fn test_global_phase() {
    let mut circuit = Circuit::new();
    circuit += DefinitionComplex::new("ro".to_string(), 4, true);
    circuit += PragmaGlobalPhase::new(0.1.into());
    circuit += PragmaGetStateVector::new("ro".to_string(), None);
    let backend = Backend::new(2, None);
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
    let backend = Backend::new(1, None);
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
    let backend = Backend::new(4, None);
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
    let backend = Backend::new(4, None);
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
    let backend = Backend::new(2, None);
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
    circuit += operations::DefinitionBit::new("ro1".to_string(), 2, true);
    circuit += operations::InputBit::new("ro1".to_string(), 1, true);
    circuit += operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);
    let backend = Backend::new(2, None);
    let (bit_result, float_result, complex_result) =
        backend.run_circuit_iterator(circuit.iter()).unwrap();
    assert!(float_result.is_empty());
    assert!(complex_result.is_empty());
    assert!(bit_result.contains_key("ro"));
    let nested_vec = bit_result.get("ro").unwrap();
    assert!(nested_vec.len() == 10);
    let nested_vec = bit_result.get("ro1").unwrap();
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
    circuit += operations::DefinitionBit::new("ro2".to_string(), 4, true);
    circuit += operations::MeasureQubit::new(0, "ro2".to_string(), 0);
    circuit += operations::MeasureQubit::new(3, "ro2".to_string(), 3);
    circuit += operations::PauliX::new(1);
    circuit += operations::PauliX::new(5);
    circuit += operations::MeasureQubit::new(0, "ro".to_string(), 0);
    circuit += operations::MeasureQubit::new(1, "ro".to_string(), 1);
    circuit += operations::MeasureQubit::new(5, "ro".to_string(), 3);
    circuit += operations::PauliX::new(1);
    circuit += operations::PauliX::new(2);
    circuit += operations::MeasureQubit::new(2, "ro".to_string(), 2);
    circuit += operations::PragmaSetNumberOfMeasurements::new(2, "ro".to_string());
    let backend = Backend::new(6, None);
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
    let backend = Backend::new(4, None);
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

#[test]
fn test_pragma_stop_parallel_block_slow() {
    // loop circuit of cst circiut
    let mut loop_circ = Circuit::new();
    loop_circ += RotateZ::new(0, 0.03.into());
    loop_circ += RotateZ::new(1, 0.03.into());
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(0, (-std::f64::consts::FRAC_PI_2).into());
    loop_circ += PragmaStopParallelBlock::new(vec![0], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += CNOT::new(0, 1);
    loop_circ += PragmaStopParallelBlock::new(vec![1, 0], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(0, std::f64::consts::FRAC_PI_2.into());
    loop_circ += RotateZ::new(1, 1.5707963267948972.into());
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateX::new(0, 1.6207963267948968.into());
    loop_circ += RotateX::new(1, 1.5707963267948974.into());
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(0, std::f64::consts::FRAC_PI_2.into());
    loop_circ += RotateZ::new(1, 1.5707963267948963.into());
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += CNOT::new(0, 1);
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(0, std::f64::consts::FRAC_PI_2.into());
    loop_circ += RotateZ::new(1, (-4.71238898038469).into());
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateX::new(0, std::f64::consts::FRAC_PI_2.into());
    loop_circ += RotateX::new(1, 1.5707963267948974.into());
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(0, std::f64::consts::FRAC_PI_2.into());
    loop_circ += RotateZ::new(1, std::f64::consts::FRAC_PI_2.into());
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += CNOT::new(0, 1);
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(1, std::f64::consts::FRAC_PI_2.into());
    loop_circ += PragmaStopParallelBlock::new(vec![1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(0, (-std::f64::consts::FRAC_PI_2).into());
    loop_circ += PragmaStopParallelBlock::new(vec![0], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += CNOT::new(0, 1);
    loop_circ += PragmaStopParallelBlock::new(vec![1, 0], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(0, std::f64::consts::FRAC_PI_2.into());
    loop_circ += RotateZ::new(1, 1.5707963267948972.into());
    loop_circ += PragmaStopParallelBlock::new(vec![1, 0], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateX::new(0, 1.6207963267948968.into());
    loop_circ += RotateX::new(1, 1.5707963267948974.into());
    loop_circ += PragmaStopParallelBlock::new(vec![1, 0], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(0, std::f64::consts::FRAC_PI_2.into());
    loop_circ += RotateZ::new(1, 1.5707963267948963.into());
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += CNOT::new(0, 1);
    loop_circ += PragmaStopParallelBlock::new(vec![1, 0], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(0, std::f64::consts::FRAC_PI_2.into());
    loop_circ += RotateZ::new(1, (-4.71238898038469).into());
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateX::new(0, std::f64::consts::FRAC_PI_2.into());
    loop_circ += RotateX::new(1, 1.5707963267948974.into());
    loop_circ += PragmaStopParallelBlock::new(vec![1, 0], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(0, std::f64::consts::FRAC_PI_2.into());
    loop_circ += RotateZ::new(1, std::f64::consts::FRAC_PI_2.into());
    loop_circ += PragmaStopParallelBlock::new(vec![1, 0], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += CNOT::new(0, 1);
    loop_circ += PragmaStopParallelBlock::new(vec![1, 0], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(1, std::f64::consts::FRAC_PI_2.into());
    loop_circ += PragmaStopParallelBlock::new(vec![1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += RotateZ::new(1, 0.03.into());
    loop_circ += RotateZ::new(0, 0.03.into());
    loop_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    loop_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    loop_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    loop_circ += PragmaGlobalPhase::new(std::f64::consts::PI.into());
    loop_circ += PragmaGlobalPhase::new(std::f64::consts::PI.into());
    loop_circ += PragmaGlobalPhase::new(std::f64::consts::FRAC_PI_2.into());
    loop_circ += PragmaGlobalPhase::new(3.9269908169872414.into());
    loop_circ += PragmaGlobalPhase::new((-std::f64::consts::FRAC_PI_4).into());
    loop_circ += PragmaGlobalPhase::new(std::f64::consts::PI.into());
    loop_circ += PragmaGlobalPhase::new(std::f64::consts::PI.into());
    loop_circ += PragmaGlobalPhase::new(std::f64::consts::FRAC_PI_2.into());
    loop_circ += PragmaGlobalPhase::new(3.9269908169872414.into());
    loop_circ += PragmaGlobalPhase::new((-std::f64::consts::FRAC_PI_4).into());

    // rest of cst circuit
    let mut cst_circ = Circuit::new();
    cst_circ += RotateZ::new(0, (-std::f64::consts::FRAC_PI_2).into());
    cst_circ += RotateZ::new(1, (-std::f64::consts::FRAC_PI_2).into());
    cst_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    cst_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    cst_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    cst_circ += RotateX::new(0, std::f64::consts::PI.into());
    cst_circ += RotateX::new(1, std::f64::consts::PI.into());
    cst_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    cst_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    cst_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    cst_circ += RotateZ::new(0, std::f64::consts::FRAC_PI_2.into());
    cst_circ += RotateZ::new(1, std::f64::consts::FRAC_PI_2.into());
    cst_circ += PragmaStopParallelBlock::new(vec![0, 1], 1.0.into());
    cst_circ += PragmaDamping::new(0, 1.0.into(), 0.0004.into());
    cst_circ += PragmaDamping::new(1, 1.0.into(), 0.0004.into());
    cst_circ += PragmaLoop::new("number_trottersteps".into(), loop_circ);

    // circuit in circuits
    let mut circ = Circuit::new();
    circ += DefinitionBit::new("ro_0".into(), 2, true);
    circ += MeasureQubit::new(1, "ro_0".into(), 1);
    circ += MeasureQubit::new(0, "ro_0".into(), 0);
    circ += PragmaStopParallelBlock::new(vec![0, 1], 0.0.into());
    circ += PragmaSetNumberOfMeasurements::new(100000, "ro_0".into());

    let internal_hash: HashMap<usize, Vec<usize>> =
        HashMap::from([(0, vec![0, 1]), (1, vec![1]), (2, vec![0])]);
    let mut input = roqoqo::measurements::PauliZProductInput {
        pauli_product_qubit_masks: HashMap::<String, HashMap<usize, Vec<usize>>>::from([(
            "ro_0".to_string(),
            internal_hash,
        )]),
        number_qubits: 2,
        number_pauli_products: 3,
        measured_exp_vals: HashMap::<String, PauliProductsToExpVal>::new(),
        use_flipped_measurement: false,
    };
    input
        .add_linear_exp_val(
            "operator_2_re".to_string(),
            HashMap::<usize, f64>::from([(0, 1.0), (1, 0.0), (2, 0.0)]),
        )
        .unwrap();
    input
        .add_linear_exp_val(
            "operator_2_im".to_string(),
            HashMap::<usize, f64>::from([(0, 0.0), (1, 0.0), (2, 0.0)]),
        )
        .unwrap();
    input
        .add_linear_exp_val(
            "operator_1_re".to_string(),
            HashMap::<usize, f64>::from([(0, 0.0), (1, 1.0), (2, 0.0)]),
        )
        .unwrap();
    input
        .add_linear_exp_val(
            "operator_1_im".to_string(),
            HashMap::<usize, f64>::from([(0, 0.0), (1, 0.0), (2, 0.0)]),
        )
        .unwrap();
    input
        .add_linear_exp_val(
            "operator_0_re".to_string(),
            HashMap::<usize, f64>::from([(0, 0.0), (1, 0.0), (2, 1.0)]),
        )
        .unwrap();
    input
        .add_linear_exp_val(
            "operator_0_im".to_string(),
            HashMap::<usize, f64>::from([(0, 0.0), (1, 0.0), (2, 0.0)]),
        )
        .unwrap();
    let measurement = roqoqo::measurements::PauliZProduct {
        constant_circuit: Some(cst_circ),
        circuits: vec![circ],
        input,
    };
    let number_trottersteps = 500;
    let backend = roqoqo_quest::Backend::new(2, None);

    for i in 0..number_trottersteps {
        let substituted = measurement
            .clone()
            .substitute_parameters(HashMap::<String, f64>::from([(
                "number_trottersteps".to_string(),
                i as f64,
            )]))
            .unwrap();
        let (bit_registers, float_registers, complex_registers) =
            backend.run_measurement_registers(&substituted).unwrap();
        let res = roqoqo::prelude::MeasureExpectationValues::evaluate(
            &measurement,
            bit_registers,
            float_registers,
            complex_registers,
        );
        assert!(res.is_ok());
        assert!(res.unwrap().is_some());
    }
}

#[test]
fn test_insufficient_qubit_error1() {
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro0".into(), 1, true);
    circuit += operations::DefinitionBit::new("ro3".into(), 4, true);
    circuit += operations::PauliX::new(0);
    circuit += operations::PauliX::new(1);
    circuit += operations::PauliX::new(2);
    circuit += operations::PauliX::new(5);
    circuit += operations::MeasureQubit::new(0, "ro0".to_string(), 0);
    circuit += operations::MeasureQubit::new(3, "ro3".to_string(), 3);
    let backend = Backend::new(4, None);
    let res = backend.run_circuit_iterator(circuit.iter());
    assert!(res.is_err());
    let e = res.unwrap_err();
    assert_eq!(
        e,
        RoqoqoBackendError::GenericError { msg: " Insufficient qubits in backend. Available qubits:`4`. Number of qubits used in circuit:`6`. ".to_string() }
    )
}

#[test]
fn test_insufficient_qubit_error2() {
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 4, true);
    circuit += operations::PauliX::new(1);
    circuit += operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);
    let backend = Backend::new(1, None);
    let res = backend.run_circuit_iterator(circuit.iter());
    assert!(res.is_err());
    let e = res.unwrap_err();
    assert_eq!(
        e,
        RoqoqoBackendError::GenericError { msg: " Insufficient qubits in backend. Available qubits:`1`. Number of qubits used in circuit:`4`. ".to_string() }
    )
}

/// Tests the case that fewer qubits are measured with MeasureQubit than are operated on and PragmaSetNumberOfMeasurements is used.
/// used. This failed in the past due to a replacement PragmaRepeatedMeasurement trying to write all qubit readouts to the array.
#[test]
fn test_replaced_repeated_measurement_fewer_qubits() {
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 2, true);
    circuit += operations::PauliX::new(0);
    circuit += operations::PauliX::new(1);
    circuit += operations::PauliX::new(2);
    circuit += operations::MeasureQubit::new(0, "ro".to_string(), 0);
    circuit += operations::MeasureQubit::new(1, "ro".to_string(), 1);

    circuit += operations::PragmaSetNumberOfMeasurements::new(10, "ro".to_string());
    let backend = Backend::new(3, None);
    let res = backend.run_circuit_iterator(circuit.iter());
    assert!(res.is_ok());
}

rusty_fork_test! {

    #[test]
    fn test_random_seed_measure_qubit() {
        let mut circuit = Circuit::new();
        circuit += operations::DefinitionBit::new("ro".to_string(), 3, true);
        circuit += operations::Hadamard::new(0);
        circuit += operations::Hadamard::new(1);
        circuit += operations::Hadamard::new(2);
        circuit += operations::MeasureQubit::new(0, "ro".to_string(), 0);
        circuit += operations::MeasureQubit::new(1, "ro".to_string(), 1);
        circuit += operations::MeasureQubit::new(2, "ro".to_string(), 2);

        let backend = Backend::new(3, Some(vec![666, 777]));
        let res = backend.run_circuit_iterator(circuit.iter()).unwrap();
        let ro = res.0.get("ro").unwrap();
        assert_eq!(ro, &vec![vec![true, false, false],]);
    }

    #[test]
    fn test_random_seed_set_number_measurement() {
        let mut circuit = Circuit::new();
        circuit += operations::DefinitionBit::new("ro".to_string(), 3, true);
        circuit += operations::Hadamard::new(0);
        circuit += operations::Hadamard::new(1);
        circuit += operations::Hadamard::new(2);
        circuit += operations::MeasureQubit::new(0, "ro".to_string(), 0);
        circuit += operations::MeasureQubit::new(1, "ro".to_string(), 1);
        circuit += operations::MeasureQubit::new(2, "ro".to_string(), 2);

        circuit += operations::PragmaSetNumberOfMeasurements::new(5, "ro".to_string());
        let backend = Backend::new(3, Some(vec![555, 666, 777]));
        let res = backend.run_circuit_iterator(circuit.iter()).unwrap();
        let ro = res.0.get("ro").unwrap();
        assert_eq!(
            ro,
            &vec![
                vec![true, true, false],
                vec![true, true, true],
                vec![true, false, true],
                vec![false, true, true],
                vec![true, false, true]
            ]
        );
    }

    #[test]
    fn test_random_seed_repeated_measurements() {
        let mut circuit = Circuit::new();
        circuit += operations::DefinitionBit::new("ro".to_string(), 3, true);
        circuit += operations::Hadamard::new(0);
        circuit += operations::Hadamard::new(1);
        circuit += operations::Hadamard::new(2);
        circuit += operations::MeasureQubit::new(0, "ro".to_string(), 0);
        circuit += operations::MeasureQubit::new(1, "ro".to_string(), 1);
        circuit += operations::MeasureQubit::new(2, "ro".to_string(), 2);

        circuit += operations::PragmaRepeatedMeasurement::new("ro".to_string(), 5, None);
        let backend = Backend::new(3, Some(vec![5554234234, 666456]));
        let res = backend.run_circuit_iterator(circuit.iter()).unwrap();
        let ro = res.0.get("ro").unwrap();
        assert_eq!(
            ro,
            &vec![
                vec![true, false, false],
                vec![false, false, false],
                vec![false, false, true],
                vec![false, true, false],
                vec![false, true, false]
            ]
        );
    }
}
