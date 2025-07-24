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
//
//! Integration test for call_operation for measurement operations

use ndarray::{array, Array1};
use num_complex::Complex64;
use roqoqo::registers::{BitOutputRegister, BitRegister, ComplexRegister, FloatRegister};
use roqoqo::{operations, Circuit};
use roqoqo_quest::{call_operation, Qureg};
use std::collections::HashMap;
use test_case::test_case;

type Registers = (
    HashMap<String, BitRegister>,
    HashMap<String, FloatRegister>,
    HashMap<String, ComplexRegister>,
    HashMap<String, BitOutputRegister>,
);

fn create_empty_registers() -> Registers {
    let bit_registers_output: HashMap<String, BitOutputRegister> = HashMap::new();
    let bit_registers: HashMap<String, BitRegister> = HashMap::new();
    let float_registers: HashMap<String, FloatRegister> = HashMap::new();
    let complex_registers: HashMap<String, ComplexRegister> = HashMap::new();
    (
        bit_registers,
        float_registers,
        complex_registers,
        bit_registers_output,
    )
}

#[test]
fn repeated_measurement() {
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    bit_registers_output.insert("ro".to_string(), Vec::new());
    let operation: operations::Operation =
        operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None).into();
    let mut qureg = Qureg::new(2, false);
    let a = call_operation(
        &operation,
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    assert!(a.is_ok());
    assert!(bit_registers_output.contains_key("ro"));
    let nested_vec = bit_registers_output.get("ro").unwrap();
    assert!(nested_vec.len() == 10);
    for repetition in nested_vec {
        assert!(repetition.len() == 2);
        for j in repetition {
            assert_eq!(j, &false)
        }
    }
    let mut mapping = HashMap::new();
    mapping.insert(0, 0);
    let operation: operations::Operation =
        operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, Some(mapping)).into();
    let a = call_operation(
        &operation,
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    assert!(a.is_ok());
}

#[test]
fn repeated_measurement_with_bitflip() {
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    bit_registers_output.insert("ro".to_string(), Vec::new());
    let operation: operations::Operation = operations::PauliX::new(0).into();

    let mut qureg = Qureg::new(2, false);
    let _ = call_operation(
        &operation,
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    let operation: operations::Operation =
        operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None).into();
    let a = call_operation(
        &operation,
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    assert!(a.is_ok());
    assert!(bit_registers_output.contains_key("ro"));
    let nested_vec = bit_registers_output.get("ro").unwrap();
    assert!(nested_vec.len() == 10);
    for repetition in nested_vec {
        assert!(repetition.len() == 2);
        assert!(repetition[0]);
        assert!(!repetition[1]);
    }
}

#[test]
fn measure_with_bitflip() {
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    bit_registers.insert("ro".to_string(), vec![false, false, false]);
    let operation: operations::Operation = operations::PauliX::new(1).into();

    let mut qureg = Qureg::new(3, false);
    let _ = call_operation(
        &operation,
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    let operation: operations::Operation =
        operations::MeasureQubit::new(0, "ro".to_string(), 1).into();
    let _ = call_operation(
        &operation,
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    let operation: operations::Operation =
        operations::MeasureQubit::new(1, "ro".to_string(), 2).into();
    let _ = call_operation(
        &operation,
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    let operation: operations::Operation =
        operations::MeasureQubit::new(2, "ro".to_string(), 0).into();
    let a = call_operation(
        &operation,
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    assert!(a.is_ok());
    assert!(bit_registers.contains_key("ro"));
    let nested_vec = bit_registers.get("ro").unwrap();
    assert!(nested_vec.len() == 3);
    assert!(!nested_vec[0]);
    assert!(!nested_vec[1]);
    assert!(nested_vec[2]);
}

#[test_case(operations::Definition::from(operations::DefinitionBit::new("ro".into(), 2, false)), false; "not_output")]
#[test_case(operations::Definition::from(operations::DefinitionBit::new("ro".into(), 2, true)), true; "output")]
fn test_definition_bit(pragma: operations::Definition, output: bool) {
    // Create the readout registers
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    // initialize with basis vector to reconstruct unitary gate
    let mut qureg = Qureg::new(1, false);
    // Apply tested operation to output
    call_operation(
        &pragma.into(),
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    )
    .unwrap();
    let mut comparison: HashMap<String, BitRegister> = HashMap::new();
    if output {
        comparison.insert("ro".into(), vec![false, false]);
        assert_eq!(bit_registers, comparison);
    } else {
        assert_eq!(bit_registers, comparison);
    }
}

#[test_case(operations::Definition::from(operations::DefinitionFloat::new("ro".into(), 2, false)), false; "not_output")]
#[test_case(operations::Definition::from(operations::DefinitionFloat::new("ro".into(), 2, true)), true; "output")]
fn test_definition_float(pragma: operations::Definition, output: bool) {
    // Create the readout registers
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    // initialize with basis vector to reconstruct unitary gate
    let mut qureg = Qureg::new(1, false);
    // Apply tested operation to output
    call_operation(
        &pragma.into(),
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    )
    .unwrap();
    let mut comparison: HashMap<String, FloatRegister> = HashMap::new();
    if output {
        comparison.insert("ro".into(), vec![0.0, 0.0]);
        assert_eq!(float_registers, comparison);
    } else {
        assert_eq!(float_registers, comparison);
    }
}

#[test_case(operations::Definition::from(operations::DefinitionComplex::new("ro".into(), 2, false)), false; "not_output")]
#[test_case(operations::Definition::from(operations::DefinitionComplex::new("ro".into(), 2, true)), true; "output")]
fn test_definition_complex(pragma: operations::Definition, output: bool) {
    // Create the readout registers
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    // initialize with basis vector to reconstruct unitary gate
    let mut qureg = Qureg::new(1, false);
    // Apply tested operation to output
    call_operation(
        &pragma.into(),
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    )
    .unwrap();
    let mut comparison: HashMap<String, ComplexRegister> = HashMap::new();
    if output {
        comparison.insert(
            "ro".into(),
            vec![Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0)],
        );
        assert_eq!(complex_registers, comparison);
    } else {
        assert_eq!(complex_registers, comparison);
    }
}

#[test]
fn test_get_pauli_product() {
    let c0: Complex64 = Complex64::new(0.0, 0.0);
    let c1: Complex64 = Complex64::new(1.0, 0.0);
    let basis_states: Vec<Array1<Complex64>> = vec![array![c1, c0, c0, c0]];
    for (column, _basis) in basis_states.into_iter().enumerate() {
        // Create the readout registers
        let (
            mut bit_registers,
            mut float_registers,
            mut complex_registers,
            mut bit_registers_output,
        ) = create_empty_registers();
        // Register for state_vector readout
        // initialize with basis vector to reconstruct unitary gate
        let mut qureg = Qureg::new(2, false);
        // Apply tested operation to output
        let mut qubit_paulis: HashMap<usize, usize> = HashMap::new();
        qubit_paulis.insert(0, 3);
        let mut circuit = Circuit::new();
        circuit += operations::PauliX::new(1);
        let pragma: operations::Operation =
            operations::PragmaGetPauliProduct::new(qubit_paulis, "ro".into(), circuit).into();
        call_operation(
            &pragma,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        let comparison: Vec<f64> = vec![1.0];
        for (row, check_value) in comparison.clone().into_iter().enumerate() {
            let value = float_registers.get("ro").unwrap()[row];
            // Check if entries are the same
            if !is_close(value, check_value) {
                // Check if reconstructed entry and enty of unitary is the same with global phase
                panic!("Reconstructed matrix entry does not match target matrix, row: {row}, column: {column}, reconstructed: {value} target: {check_value}")
            }
        }
    }
}

#[test_case(true; "is_density_matrix")]
#[test_case(false; "is_state_vector")]
fn test_get_occupation_probability(density: bool) {
    // Create the readout registers
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    // initialize with basis vector to reconstruct unitary gate
    let mut qureg = Qureg::new(2, density);
    let mut circuit = Circuit::new();
    circuit += operations::PauliX::new(1);
    // Apply tested operation to output
    let pragma: operations::Operation =
        operations::PragmaGetOccupationProbability::new("ro".into(), Some(circuit)).into();
    call_operation(
        &pragma,
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    )
    .unwrap();
    let mut comparison: Vec<f64> = vec![0.0, 0.0, 1.0, 0.0];
    if density {
        comparison = vec![
            vec![0.0; comparison.len()],
            vec![0.0; comparison.len()],
            comparison.clone(),
            vec![0.0; comparison.len()],
        ]
        .into_iter()
        .flatten()
        .collect();
    }
    for (row, check_value) in comparison.into_iter().enumerate() {
        let value = float_registers.get("ro").unwrap()[row];
        // Check if entries are the same
        if !is_close(value, check_value) {
            // Check if reconstructed entry and enty of unitary is the same with global phase
            panic!("Reconstructed matrix entry does not match target matrix, row: {row}, reconstructed: {value} target: {check_value}")
        }
    }
}

fn is_close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-10
}
