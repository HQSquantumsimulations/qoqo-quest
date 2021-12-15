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
//! Integration test for call_operation for pragma operations

use ndarray::{array, Array1, Array2};
use num_complex::{Complex, Complex64};
use roqoqo::operations::{
    self, PragmaGetDensityMatrix, PragmaGetStateVector, PragmaNoiseOperation,
    PragmaSetDensityMatrix, PragmaSetStateVector,
};
use roqoqo::prelude::{OperatePragmaNoise, RoqoqoBackendError};
use roqoqo::{
    registers::{BitOutputRegister, BitRegister, ComplexRegister, FloatRegister},
    Circuit,
};
use roqoqo_quest::{call_operation, Qureg};
use std::collections::HashMap;
use test_case::test_case;

fn create_empty_registers() -> (
    HashMap<String, BitRegister>,
    HashMap<String, FloatRegister>,
    HashMap<String, ComplexRegister>,
    HashMap<String, BitOutputRegister>,
) {
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
fn test_store_load_state_vector() {
    let c0: Complex64 = Complex::new(0.0, 0.0);
    let c1: Complex64 = Complex::new(1.0, 0.0);
    let basis_states: Vec<Array1<Complex64>> = vec![array![c1, c0], array![c0, c1]];
    for (column, basis) in basis_states.into_iter().enumerate() {
        // Create the readout registers
        let (
            mut bit_registers,
            mut float_registers,
            mut complex_registers,
            mut bit_registers_output,
        ) = create_empty_registers();
        // Register for state_vector readout
        complex_registers.insert("state_vec".to_string(), Vec::new());
        // initialize with basis vector to reconstruct unitary gate
        let mut qureg = Qureg::new(1, false);
        let set_basis_operation: operations::Operation =
            PragmaSetStateVector::new(basis.clone()).into();
        call_operation(
            &set_basis_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        // Extract state vector
        let extract_state_vector_operation: operations::Operation =
            PragmaGetStateVector::new("state_vec".to_string(), None).into();
        call_operation(
            &extract_state_vector_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        for (index, value) in basis.clone().iter().enumerate() {
            // Check if entries are the same
            if !is_close(
                value.clone(),
                complex_registers
                    .get("state_vec")
                    .expect("No state_vec produced by PragmaGetStateVec")[index],
            ) {
                {
                    panic!("Read-out entry does not match input value, index: {}, column: {}, input: {} read-out: {}", 
                    index, column, value, complex_registers
                    .get("state_vec")
                    .expect("No state_vec produced by PragmaGetStateVec")[index])
                }
            }
        }
    }
}

#[test_case(true; "is_density_matrix")]
#[test_case(false; "is_state_vector")]
fn test_store_state_vec_load_density_matrix_qureg(density: bool) {
    let c0: Complex64 = Complex::new(0.0, 0.0);
    let c1: Complex64 = Complex::new(1.0, 0.0);
    let c2: Complex64 = Complex::new(1.0 / (2.0), 0.0);
    let c2c: Complex64 = Complex::new(0.0, 1.0 / (2.0));
    let c2cs: Complex64 = Complex::new(0.0, 1.0 / (2.0_f64.sqrt()));
    let c2s: Complex64 = Complex::new(1.0 / (2.0_f64.sqrt()), 0.0);
    let basis_states: Vec<Array1<Complex64>> =
        vec![array![c1, c0], array![c0, c1], array![c2s, c2cs]];
    let density_matrices: Vec<Array2<Complex64>> = vec![
        array![[c1, c0], [c0, c0]],
        array![[c0, c0], [c0, c1]],
        array![[c2, -c2c], [c2c, c2]],
    ];
    for (test_number, (basis, density_matrix)) in basis_states
        .into_iter()
        .zip(density_matrices.into_iter())
        .enumerate()
    {
        // Create the readout registers
        let (
            mut bit_registers,
            mut float_registers,
            mut complex_registers,
            mut bit_registers_output,
        ) = create_empty_registers();
        // Register for state_vector readout
        complex_registers.insert("density_mattrix".to_string(), Vec::new());
        // initialize with basis vector to reconstruct unitary gate
        let mut qureg = Qureg::new(1, density);
        let set_basis_operation: operations::Operation =
            PragmaSetStateVector::new(basis.clone()).into();
        call_operation(
            &set_basis_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        // Extract state vector
        let extract_state_vector_operation: operations::Operation =
            PragmaGetDensityMatrix::new("density_mattrix".to_string(), None).into();
        call_operation(
            &extract_state_vector_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        for (index, value) in density_matrix.iter().enumerate() {
            if !is_close(
                value.clone(),
                complex_registers
                    .get("density_mattrix")
                    .expect("No density_mattrix produced by PragmaGetStateVec")[index],
            ) {
                {
                    panic!("Read-out entry does not match input value, index: {}, test_number: {},  input: {}, read-out: {}", 
                    index, test_number, value, complex_registers
                    .get("density_mattrix")
                    .expect("No density_mattrix produced by PragmaGetStateVec")[index])
                }
            }
        }
    }
}

#[test]
fn test_store_load_density_matrix_qureg() {
    let c0: Complex64 = Complex::new(0.0, 0.0);
    let c1: Complex64 = Complex::new(1.0, 0.0);
    let c2: Complex64 = Complex::new(1.0 / (2.0), 0.0);
    let c2c: Complex64 = Complex::new(0.0, 1.0 / (2.0));

    let density_matrices: Vec<Array2<Complex64>> = vec![
        array![[c1, c0], [c0, c0]],
        array![[c0, c0], [c0, c1]],
        array![[c2, -c2c], [c2c, c2]],
    ];
    for (test_number, density_matrix) in density_matrices.into_iter().enumerate() {
        // Create the readout registers
        let (
            mut bit_registers,
            mut float_registers,
            mut complex_registers,
            mut bit_registers_output,
        ) = create_empty_registers();
        // Register for state_vector readout
        complex_registers.insert("density_mattrix".to_string(), Vec::new());
        // initialize with basis vector to reconstruct unitary gate
        let mut qureg = Qureg::new(1, true);
        let set_basis_operation: operations::Operation =
            PragmaSetDensityMatrix::new(density_matrix.clone()).into();
        call_operation(
            &set_basis_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        // Extract state vector
        let extract_state_vector_operation: operations::Operation =
            PragmaGetDensityMatrix::new("density_mattrix".to_string(), None).into();
        call_operation(
            &extract_state_vector_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        for (index, value) in density_matrix.iter().enumerate() {
            if !is_close(
                value.clone(),
                complex_registers
                    .get("density_mattrix")
                    .expect("No density_mattrix produced by PragmaGetStateVec")[index],
            ) {
                {
                    panic!("Read-out entry does not match input value, index: {}, test_number: {},  input: {}, read-out: {}", 
                    index, test_number, value, complex_registers
                    .get("density_mattrix")
                    .expect("No density_mattrix produced by PragmaGetStateVec")[index])
                }
            }
        }
    }
}

// #[test_case(operations::PragmaNoiseOperation::from(operations::PragmaGeneralNoise::new(0, 0.01.into(),  array![[0.1, 0.0, 0.0],[0.0, 0.2, 0.0],[0.0, 0.0, 0.3]])); "PragmaGeneralNoise")]
#[test_case(operations::PragmaNoiseOperation::from(operations::PragmaDamping::new(0, 0.01.into(),  2.0.into())); "PragmaDamping001")]
#[test_case(operations::PragmaNoiseOperation::from(operations::PragmaDamping::new(0, 0.1.into(),  2.0.into())); "PragmaDamping01")]
#[test_case(operations::PragmaNoiseOperation::from(operations::PragmaDamping::new(0, 1.0.into(),  2.0.into())); "PragmaDamping1")]
#[test_case(operations::PragmaNoiseOperation::from(operations::PragmaDephasing::new(0, 0.01.into(),  2.0.into())); "PragmaDephasing")]
#[test_case(operations::PragmaNoiseOperation::from(operations::PragmaDepolarising::new(0, 0.01.into(),  2.0.into())); "PragmaDepolarising001")]
#[test_case(operations::PragmaNoiseOperation::from(operations::PragmaDepolarising::new(0, 0.1.into(),  2.0.into())); "PragmaDepolarising01")]
#[test_case(operations::PragmaNoiseOperation::from(operations::PragmaDepolarising::new(0, 1.0.into(),  2.0.into())); "PragmaDepolarising1")]

// #[test_case(operations::PragmaNoiseOperation::from(operations::PragmaRandomNoise::new(0, 0.01.into(),  2.0.into(), 0.0.into())); "PragmaRandomNoise")]
fn test_general_noise(operation: PragmaNoiseOperation) {
    let c0: Complex64 = Complex::new(0.0, 0.0);
    let c1: Complex64 = Complex::new(1.0, 0.0);
    let c2: Complex64 = Complex::new(1.0 / (2.0), 0.0);
    let c2c: Complex64 = Complex::new(0.0, 1.0 / (2.0));
    let c2cs: Complex64 = Complex::new(0.0, 1.0 / (2.0_f64.sqrt()));
    let c2s: Complex64 = Complex::new(1.0 / (2.0_f64.sqrt()), 0.0);
    let basis_states: Vec<Array1<Complex64>> = vec![
        array![c1, c0],
        array![c0, c1],
        array![c2s, c2cs],
        array![c2s, c2s],
    ];
    let density_matrices: Vec<Array1<Complex64>> = vec![
        array![c1, c0, c0, c0],
        array![c0, c0, c0, c1],
        array![c2, -c2c, c2c, c2],
        array![c2, c2, c2, c2],
    ];
    let super_op = operation.superoperator().unwrap();
    let mut complex_superop = Array2::<Complex64>::zeros((4, 4));
    for ((row, column), val) in super_op.indexed_iter() {
        complex_superop[(row, column)] = Complex64::new(*val, 0.0);
    }
    for (test_number, (basis, density_matrix)) in basis_states
        .into_iter()
        .zip(density_matrices.into_iter())
        .enumerate()
    {
        let test_density_matrix = complex_superop.dot(&density_matrix);
        // Create the readout registers
        let (
            mut bit_registers,
            mut float_registers,
            mut complex_registers,
            mut bit_registers_output,
        ) = create_empty_registers();
        // Register for state_vector readout
        complex_registers.insert("state_vec".to_string(), Vec::new());
        // initialize with basis vector to reconstruct unitary gate
        let mut qureg = Qureg::new(1, true);
        let set_basis_operation: operations::Operation = PragmaSetStateVector::new(basis).into();
        call_operation(
            &set_basis_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        // Apply tested operation to output
        call_operation(
            &operation.clone().into(),
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        // Extract state vector
        let extract_state_vector_operation: operations::Operation =
            PragmaGetDensityMatrix::new("density_matrix".to_string(), None).into();
        call_operation(
            &extract_state_vector_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        dbg!(&test_density_matrix);
        dbg!(&complex_registers.get("density_matrix").unwrap());
        for (index, (check_value, calculated_value)) in test_density_matrix
            .into_iter()
            .zip(complex_registers.get("density_matrix").unwrap().into_iter())
            .enumerate()
        {
            // Check if entries are the same
            if !is_close(*calculated_value, check_value) {
                // Check if reconstructed entry and enty of unitary is the same with global phase
                panic!("Reconstructed matrix entry does not match targe matrix, index: {}, test_number: {}, reconstructed: {} target: {} ",
                    index, test_number, calculated_value, check_value)
            }
        }
    }
}

#[test]
fn test_statevec_multiplication_quest() {
    let c0: Complex64 = Complex::new(0.0, 0.0);
    let c1: Complex64 = Complex::new(1.0, 0.0);
    let density_matrices: Vec<Array1<Complex64>> = vec![
        array![c1, c0, c0, c0],
        array![c0, c1, c0, c0],
        array![c0, c0, c1, c0],
        array![c0, c0, c0, c1],
    ];
    let unitary_matrices: Vec<Array2<Complex64>> = vec![
        array![
            [c1, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c1, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c0, c1, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c0, c0, c1],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c1, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c0, c1, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c0, c0, c1, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c0, c0, c0, c1],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c1, c0, c0, c0],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c1, c0, c0],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c1, c0],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c1],
            [c0, c0, c0, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c1, c0, c0, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c1, c0, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c1, c0]
        ],
        array![
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c0],
            [c0, c0, c0, c1]
        ],
    ];
    for (test_number, density_matrix) in density_matrices.into_iter().enumerate() {
        for unitary_matrix in unitary_matrices.clone().into_iter() {
            let qureg = Qureg::new(1, true);
            let mut reals: Vec<f64> = density_matrix.iter().map(|x| x.re).collect();
            let mut imags: Vec<f64> = density_matrix.iter().map(|x| x.im).collect();
            unsafe {
                quest_sys::initStateFromAmps(
                    qureg.quest_qureg,
                    reals.as_mut_ptr(),
                    imags.as_mut_ptr(),
                )
            }

            let complex_matrix = quest_sys::ComplexMatrix4 {
                // Row major version
                real: [
                    [
                        unitary_matrix[(0, 0)].re,
                        unitary_matrix[(0, 1)].re,
                        unitary_matrix[(0, 2)].re,
                        unitary_matrix[(0, 3)].re,
                    ],
                    [
                        unitary_matrix[(1, 0)].re,
                        unitary_matrix[(1, 1)].re,
                        unitary_matrix[(1, 2)].re,
                        unitary_matrix[(1, 3)].re,
                    ],
                    [
                        unitary_matrix[(2, 0)].re,
                        unitary_matrix[(2, 1)].re,
                        unitary_matrix[(2, 2)].re,
                        unitary_matrix[(2, 3)].re,
                    ],
                    [
                        unitary_matrix[(3, 0)].re,
                        unitary_matrix[(3, 1)].re,
                        unitary_matrix[(3, 2)].re,
                        unitary_matrix[(3, 3)].re,
                    ],
                ],
                imag: [
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                ],
            };
            unsafe {
                quest_sys::statevec_twoQubitUnitary(qureg.quest_qureg, 0_i32, 1_i32, complex_matrix)
            }
            let mut comparison_matrix = Array2::<f64>::zeros((2, 2));
            for row in 0..2 {
                for column in 0..2 {
                    // QuEST is column major
                    unsafe {
                        comparison_matrix[(row, column)] =
                            quest_sys::getDensityAmp(qureg.quest_qureg, column as i64, row as i64)
                                .real
                    }
                }
            }
            let test_density_matrix = unitary_matrix.dot(&density_matrix);

            for (index, (value_test, value_comp)) in test_density_matrix
                .into_iter()
                .zip(comparison_matrix.into_iter())
                .enumerate()
            {
                // Check if entries are the same
                if !((value_test.re - value_comp).abs() < 1e-10) {
                    // Check if reconstructed entry and enty of unitary is the same with global phase
                    panic!("Reconstructed matrix entry does not match targe matrix, index: {}, test_number: {}, value_from_multiplication: {} value_from_quest: {} ",
                           index, test_number, value_test.re, value_comp)
                }
            }
        }
    }
}

#[test_case(operations::Operation::from(operations::PragmaSetNumberOfMeasurements::new(3, "ro".into())); "PragmaSetNumberOfMeasurements")]
#[test_case(operations::Operation::from(operations::PragmaRepeatGate::new(3)); "PragmaRepeatGate")]
// #[test_case(operations::Operation::from(operations::PragmaOverrotation::new("PauliX".into(), vec![0, 1], 0.1, 0.2)); "PragmaOverrotation")]
#[test_case(operations::Operation::from(operations::PragmaBoostNoise::new(0.5.into())); "PragmaBoostNoise")]
#[test_case(operations::Operation::from(operations::PragmaStopParallelBlock::new(vec![0, 1], 0.5.into())); "PragmaStopParallelBlock")]
#[test_case(operations::Operation::from(operations::PragmaGlobalPhase::new(0.5.into())); "PragmaGlobalPhase")]
#[test_case(operations::Operation::from(operations::PragmaStartDecompositionBlock::new(vec![0, 1], HashMap::new())); "PragmaStartDecompositionBlock")]
#[test_case(operations::Operation::from(operations::PragmaStopDecompositionBlock::new(vec![0, 1])); "PragmaStopDecompositionBlock")]
#[test_case(operations::Operation::from(operations::DefinitionUsize::new("ro".into(), 2, false)); "DefinitionUsize")]
#[test_case(operations::Operation::from(operations::InputSymbolic::new("ro".into(), 2.0)); "InputSymbolic")]
fn test_skipped_operations(pragma: operations::Operation) {
    let c0: Complex64 = Complex::new(0.0, 0.0);
    let c1: Complex64 = Complex::new(1.0, 0.0);
    let basis_states: Vec<Array1<Complex64>> = vec![array![c1, c0], array![c0, c1]];
    for (column, basis) in basis_states.clone().into_iter().enumerate() {
        // Create the readout registers
        let (
            mut bit_registers,
            mut float_registers,
            mut complex_registers,
            mut bit_registers_output,
        ) = create_empty_registers();
        // Register for state_vector readout
        complex_registers.insert("state_vec".to_string(), Vec::new());
        // initialize with basis vector to reconstruct unitary gate
        let mut qureg = Qureg::new(1, false);
        let set_basis_operation: operations::Operation =
            PragmaSetStateVector::new(basis.clone()).into();
        call_operation(
            &set_basis_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        // Apply tested operation to output
        call_operation(
            &pragma.clone().into(),
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        // Extract state vector
        let extract_state_vector_operation: operations::Operation =
            PragmaGetStateVector::new("state_vec".to_string(), None).into();
        call_operation(
            &extract_state_vector_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        for (row, check_value) in basis.clone().into_iter().enumerate() {
            let value = complex_registers.get("state_vec").unwrap()[row];
            // Check if entries are the same
            if !is_close(value, check_value) {
                // Check if reconstructed entry and enty of unitary is the same with global phase
                panic!("Reconstructed matrix entry does not match target matrix, row: {}, column: {}, reconstructed: {} target: {}", 
                       row, column, value, check_value)
            }
        }
    }
}

#[test]
fn test_active_reset() {
    let c0: Complex64 = Complex::new(0.0, 0.0);
    let c1: Complex64 = Complex::new(1.0, 0.0);
    let basis_states: Vec<Array1<Complex64>> = vec![array![c1, c0, c0, c0], array![c0, c0, c1, c0]];
    for (column, basis) in basis_states.clone().into_iter().enumerate() {
        // Create the readout registers
        let (
            mut bit_registers,
            mut float_registers,
            mut complex_registers,
            mut bit_registers_output,
        ) = create_empty_registers();
        // Register for state_vector readout
        complex_registers.insert("state_vec".to_string(), Vec::new());
        // initialize with basis vector to reconstruct unitary gate
        let mut qureg = Qureg::new(2, false);
        let set_basis_operation: operations::Operation =
            PragmaSetStateVector::new(basis.clone()).into();
        call_operation(
            &set_basis_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        // Apply tested operation to output
        let pragma: operations::Operation = operations::PragmaActiveReset::new(1).into();
        call_operation(
            &pragma.clone().into(),
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        // Extract state vector
        let extract_state_vector_operation: operations::Operation =
            PragmaGetStateVector::new("state_vec".to_string(), None).into();
        call_operation(
            &extract_state_vector_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        for (row, check_value) in basis_states[0].clone().into_iter().enumerate() {
            let value = complex_registers.get("state_vec").unwrap()[row];
            // Check if entries are the same
            if !is_close(value, check_value) {
                // Check if reconstructed entry and enty of unitary is the same with global phase
                panic!("Reconstructed matrix entry does not match target matrix, row: {}, column: {}, reconstructed: {} target: {}", 
                       row, column, value, check_value)
            }
        }
    }
}

#[test]
fn test_conditional() {
    let c0: Complex64 = Complex::new(0.0, 0.0);
    let c1: Complex64 = Complex::new(1.0, 0.0);
    let basis_states: Vec<Array1<Complex64>> = vec![array![c1, c0, c0, c0], array![c0, c0, c1, c0]];
    let comparison_states: Vec<Array1<Complex64>> =
        vec![array![c0, c1, c0, c0], array![c0, c0, c0, c1]];
    for (column, basis) in basis_states.clone().into_iter().enumerate() {
        // Create the readout registers
        let (
            mut bit_registers,
            mut float_registers,
            mut complex_registers,
            mut bit_registers_output,
        ) = create_empty_registers();
        // Register for state_vector readout
        bit_registers.insert("conditional".to_string(), vec![false, true]);
        // initialize with basis vector to reconstruct unitary gate
        let mut qureg = Qureg::new(2, false);
        let set_basis_operation: operations::Operation =
            PragmaSetStateVector::new(basis.clone()).into();
        call_operation(
            &set_basis_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        // Apply tested operation to output
        let mut circuit = Circuit::new();
        circuit += operations::PauliX::new(0);
        let pragma: operations::Operation =
            operations::PragmaConditional::new("conditional".into(), 1, circuit).into();
        call_operation(
            &pragma.clone().into(),
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        // Extract state vector
        let extract_state_vector_operation: operations::Operation =
            PragmaGetStateVector::new("state_vec".to_string(), None).into();
        call_operation(
            &extract_state_vector_operation,
            &mut qureg,
            &mut bit_registers,
            &mut float_registers,
            &mut complex_registers,
            &mut bit_registers_output,
        )
        .unwrap();
        for (row, check_value) in comparison_states[column].clone().into_iter().enumerate() {
            let value = complex_registers.get("state_vec").unwrap()[row];
            // Check if entries are the same
            if !is_close(value, check_value) {
                // Check if reconstructed entry and enty of unitary is the same with global phase
                panic!("Reconstructed matrix entry does not match target matrix, row: {}, column: {}, reconstructed: {} target: {}", 
                       row, column, value, check_value)
            }
        }
    }
}

#[test]
fn test_sleep_pragma() {
    let pragma = operations::PragmaSleep::new(vec![0, 1], 0.5.into());
    let mut qureg = Qureg::new(1, false);
    // Create the readout registers
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    // Apply tested operation to output
    let error = call_operation(
        &pragma.clone().into(),
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    assert!(error.is_err());
    assert_eq!(
        error,
        Err(RoqoqoBackendError::OperationNotInBackend {
            backend: "QuEST",
            hqslang: "PragmaSleep",
        })
    );
}

#[test]
fn test_set_density_matrix_error_1() {
    let pragma = operations::PragmaSetDensityMatrix::new(array![[Complex64::new(1.0, 0.0)]]);
    let mut qureg = Qureg::new(1, false);
    // Create the readout registers
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    // Apply tested operation to output
    let error = call_operation(
        &pragma.clone().into(),
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    assert!(error.is_err());
    assert_eq!(
        error,
        Err(RoqoqoBackendError::GenericError {
            msg: "Can not set state vector number of qubits of statevector 1 differs from number of qubits in qubit register 1".to_string()
        })
    );
}

#[test]
fn test_set_density_matrix_error_2() {
    let pragma = operations::PragmaSetDensityMatrix::new(array![
        [Complex64::new(1.0, 0.0), Complex64::new(1.0, 0.0)],
        [Complex64::new(1.0, 0.0), Complex64::new(1.0, 0.0)]
    ]);
    let mut qureg = Qureg::new(1, false);
    // Create the readout registers
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    // Apply tested operation to output
    let error = call_operation(
        &pragma.clone().into(),
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    assert!(error.is_err());
    assert_eq!(
        error,
        Err(RoqoqoBackendError::GenericError {
            msg: "Density matrix can not be set on state vector quantum register".to_string()
        })
    );
}

#[test]
fn test_set_state_vector_error() {
    let pragma = operations::PragmaSetStateVector::new(array![Complex64::new(1.0, 0.0)]);
    let mut qureg = Qureg::new(1, false);
    // Create the readout registers
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    // Apply tested operation to output
    let error = call_operation(
        &pragma.clone().into(),
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    assert!(error.is_err());
    assert_eq!(
        error,
        Err(RoqoqoBackendError::GenericError {
            msg: "Can not set state vector number of qubits of statevector 1 differs from number of qubits in qubit register 1".to_string()
        })
    );
}

#[test]
fn test_get_state_vector_error() {
    let pragma = operations::PragmaGetStateVector::new("ro".into(), None);
    let mut qureg = Qureg::new(1, true);
    // Create the readout registers
    let (mut bit_registers, mut float_registers, mut complex_registers, mut bit_registers_output) =
        create_empty_registers();
    // Apply tested operation to output
    let error = call_operation(
        &pragma.clone().into(),
        &mut qureg,
        &mut bit_registers,
        &mut float_registers,
        &mut complex_registers,
        &mut bit_registers_output,
    );
    assert!(error.is_err());
    assert_eq!(
        error,
        Err(RoqoqoBackendError::GenericError {
            msg: "Trying to obtain state vector from density matrix quantum register".to_string()
        })
    );
}

fn is_close(a: Complex64, b: Complex64) -> bool {
    (a - b).norm() < 1e-10
}
