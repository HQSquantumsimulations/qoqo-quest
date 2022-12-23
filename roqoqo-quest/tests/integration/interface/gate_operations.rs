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
//! Integration test for call_operation for gate operations

use ndarray::{array, Array1};
use num_complex::{Complex, Complex64};
use roqoqo::operations::OperateMultiQubit;
use roqoqo::operations::{self, PragmaGetStateVector, PragmaSetStateVector};
use roqoqo::prelude::{OperateGate, OperateSingleQubitGate};
use roqoqo::registers::{BitOutputRegister, BitRegister, ComplexRegister, FloatRegister};
use roqoqo_quest::{call_operation, Qureg};
use std::collections::HashMap;
use std::convert::TryInto;
use test_case::test_case;

type AllRegisters = (
    HashMap<String, BitRegister>,
    HashMap<String, FloatRegister>,
    HashMap<String, ComplexRegister>,
    HashMap<String, BitOutputRegister>,
);

fn create_empty_registers() -> AllRegisters {
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

#[test_case(operations::SingleQubitGateOperation::from(operations::Hadamard::new(0)); "Hadamard")]
#[test_case(operations::SingleQubitGateOperation::from(operations::PauliX::new(0));"PauliX")]
#[test_case(operations::SingleQubitGateOperation::from(operations::PauliY::new(0));"PauliY")]
#[test_case(operations::SingleQubitGateOperation::from(operations::PauliZ::new(0));"PauliZ")]
#[test_case(operations::SingleQubitGateOperation::from(operations::RotateX::new(0, 0.0.into()));"RotateX")]
#[test_case(operations::SingleQubitGateOperation::from(operations::RotateY::new(0, 1.0.into()));"RotateY")]
#[test_case(operations::SingleQubitGateOperation::from(operations::RotateZ::new(0, 2.0.into()));"RotateZ")]
#[test_case(operations::SingleQubitGateOperation::from(operations::PhaseShiftState0::new(0, 3.0.into()));"PhaseShiftState0")]
#[test_case(operations::SingleQubitGateOperation::from(operations::PhaseShiftState1::new(0, 4.0.into()));"PhaseShiftState1")]
#[test_case(operations::SingleQubitGateOperation::from(operations::SGate::new(0)); "SGate")]
#[test_case(operations::SingleQubitGateOperation::from(operations::TGate::new(0)); "TGate")]
#[test_case(operations::SingleQubitGateOperation::from(operations::SqrtPauliX::new(0)); "SqrtPauliX")]
#[test_case(operations::SingleQubitGateOperation::from(operations::InvSqrtPauliX::new(0)); "InvSqrtPauliX")]
#[test_case(operations::SingleQubitGateOperation::from(operations::RotateAroundSphericalAxis::new(0, 1.0.into(), 0.5.into(), 1.0.into())); "RotateAroundSphericalAxis")]
#[test_case(operations::SingleQubitGateOperation::from(operations::SingleQubitGate::new(0,0.5.into(),  0.5.into(), 0.5.into(), 0.5.into(), 0.5.into()));"SingleQubitGate")]
fn test_single_qubit_gate(operation: operations::SingleQubitGateOperation) {
    let c0: Complex64 = Complex::new(0.0, 0.0);
    let c1: Complex64 = Complex::new(1.0, 0.0);
    let basis_states: Vec<Array1<Complex64>> = vec![array![c1, c0], array![c0, c1]];
    let unitary_matrix = operation.unitary_matrix().unwrap();
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
        for (row, check_value) in unitary_matrix.column(column).into_iter().enumerate()
        // complex_registers
        // .get("state_vec")
        // .unwrap()
        // .into_iter()
        // .enumerate()
        {
            let value = complex_registers.get("state_vec").unwrap()[row];
            // Check if entries are the same
            if !is_close(value, *check_value) {
                // Check if reconstructed entry and enty of unitary is the same with global phase
                if !is_close_phased(
                    value,
                    *check_value,
                    operation.global_phase().try_into().unwrap(),
                ) {
                    panic!("Reconstructed matrix entry does not match targe matrix, row: {}, column: {}, reconstructed: {} target: {} global_phase: {}", 
                    row, column, value, check_value,operation.global_phase())
                }
            }
        }
    }
}

#[test_case(operations::TwoQubitGateOperation::from(operations::CNOT::new(1,0)); "CNOT")]
#[test_case(operations::TwoQubitGateOperation::from(operations::SWAP::new(1,0)); "SWAP")]
#[test_case(operations::TwoQubitGateOperation::from(operations::FSwap::new(1,0)); "FSwap")]
#[test_case(operations::TwoQubitGateOperation::from(operations::ISwap::new(1,0)); "ISwap")]
#[test_case(operations::TwoQubitGateOperation::from(operations::SqrtISwap::new(1,0)); "SqrtISwap")]
#[test_case(operations::TwoQubitGateOperation::from(operations::InvSqrtISwap::new(1,0)); "InvSqrtISwap")]
#[test_case(operations::TwoQubitGateOperation::from(operations::XY::new(1,0, 1.0.into())); "XY")]
#[test_case(operations::TwoQubitGateOperation::from(operations::ControlledPauliY::new(1,0)); "ControlledPauliY")]
#[test_case(operations::TwoQubitGateOperation::from(operations::ControlledPauliZ::new(1,0)); "ControlledPauliZ")]
#[test_case(operations::TwoQubitGateOperation::from(operations::ControlledPhaseShift::new(1,0, 1.0.into())); "ControlledPhaseShift")]
#[test_case(operations::TwoQubitGateOperation::from(operations::PMInteraction::new(1,0, 1.0.into())); "PMInteraction")]
#[test_case(operations::TwoQubitGateOperation::from(operations::ComplexPMInteraction::new(1,0, 1.0.into(), 2.0.into())); "ComplexPMInteraction")]
#[test_case(operations::TwoQubitGateOperation::from(operations::MolmerSorensenXX::new(1,0,)); "MolmerSorensenXX")]
#[test_case(operations::TwoQubitGateOperation::from(operations::VariableMSXX::new(1,0, 1.0.into())); "VariableMSXX")]
#[test_case(operations::TwoQubitGateOperation::from(operations::GivensRotation::new(1,0, 1.0.into(), 2.0.into())); "GivensRotation")]
#[test_case(operations::TwoQubitGateOperation::from(operations::GivensRotationLittleEndian::new(1,0, 1.0.into(), 2.0.into())); "GivensRotationLittleEndian")]
#[test_case(operations::TwoQubitGateOperation::from(operations::Qsim::new(1,0, 0.5.into(), 1.0.into(), 0.5.into())); "Qsim")]
#[test_case(operations::TwoQubitGateOperation::from(operations::Fsim::new(1,0, 0.5.into(), 1.0.into(), 0.5.into())); "Fsim")]
#[test_case(operations::TwoQubitGateOperation::from(operations::SpinInteraction::new(1,0, 1.0.into(), 2.0.into(), 3.0.into())); "SpinInteraction")]
#[test_case(operations::TwoQubitGateOperation::from(operations::Bogoliubov::new(1,0, 1.0.into(), 2.0.into())); "Bogoliubov")]
#[test_case(operations::TwoQubitGateOperation::from(operations::PhaseShiftedControlledZ::new(1,0, 3.0.into())); "PhaseShifterControlledZ")]
fn test_two_qubit_gate(operation: operations::TwoQubitGateOperation) {
    let c0: Complex64 = Complex::new(0.0, 0.0);
    let c1: Complex64 = Complex::new(1.0, 0.0);
    let basis_states: Vec<Array1<Complex64>> = vec![
        array![c1, c0, c0, c0],
        array![c0, c1, c0, c0],
        array![c0, c0, c1, c0],
        array![c0, c0, c0, c1],
    ];
    let unitary_matrix = operation.unitary_matrix().unwrap();
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
        let mut qureg = Qureg::new(2, false);
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
        for (row, check_value) in unitary_matrix.column(column).into_iter().enumerate() {
            let value = complex_registers.get("state_vec").unwrap()[row];
            // Check if entries are the same
            if !is_close(value, *check_value) {
                // Check if reconstructed entry and enty of unitary is the same with global phase
                panic!("Reconstructed matrix entry does not match targe matrix, row: {}, column: {}, reconstructed: {} target: {} ", 
                    row, column, value, check_value)
            }
        }
    }
}

#[test_case(operations::MultiQubitGateOperation::from(operations::MultiQubitMS::new(vec![0,1,2,3], 1.0.into())); "MultiQubitMS")]
fn test_multi_qubit_gate(operation: operations::MultiQubitGateOperation) {
    let c1: Complex64 = Complex::new(1.0, 0.0);
    let mut basis_states: Vec<Array1<Complex64>> = Vec::new();
    let number_qubits = operation.qubits().iter().max().unwrap() + 1;
    let dimension = 2_usize.pow(operation.qubits().len() as u32);
    for i in 0..dimension {
        let mut tmp_array = Array1::<Complex64>::zeros(dimension);
        tmp_array[i] = c1;
        basis_states.push(tmp_array);
    }
    let unitary_matrix = operation.unitary_matrix().unwrap();
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
        let mut qureg = Qureg::new(number_qubits as u32, false);
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
        for (row, check_value) in unitary_matrix.column(column).into_iter().enumerate() {
            let value = complex_registers.get("state_vec").unwrap()[row];
            // Check if entries are the same
            if !is_close(value, *check_value) {
                // Check if reconstructed entry and enty of unitary is the same with global phase
                panic!("Reconstructed matrix entry does not match targe matrix, row: {}, column: {}, reconstructed: {} target: {} ", 
                    row, column, value, check_value)
            }
        }
    }
}

fn is_close(a: Complex64, b: Complex64) -> bool {
    (a - b).norm() < 1e-10
}

fn is_close_phased(a: Complex64, b: Complex64, global_phase: f64) -> bool {
    let phase = Complex64::new(global_phase.cos(), global_phase.sin());
    (phase * a - b).norm() < 1e-10
}
