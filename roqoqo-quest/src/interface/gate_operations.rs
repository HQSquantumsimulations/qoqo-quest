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

use std::borrow::BorrowMut;

use crate::ComplexMatrixN;
use crate::Qureg;
use roqoqo::operations::*;
use roqoqo::RoqoqoBackendError;

/// Simulate generic single qubit gate operate on quantum register
pub fn execute_generic_single_qubit_operation(
    operation: &SingleQubitGateOperation,
    qureg: &mut Qureg,
) -> Result<(), RoqoqoBackendError> {
    let unitary_matrix = operation.unitary_matrix()?;
    let complex_matrix = quest_sys::ComplexMatrix2 {
        // row major version only used for Complex2/4/N intio
        real: [
            [unitary_matrix[(0, 0)].re, unitary_matrix[(0, 1)].re],
            [unitary_matrix[(1, 0)].re, unitary_matrix[(1, 1)].re],
        ],
        imag: [
            [unitary_matrix[(0, 0)].im, unitary_matrix[(0, 1)].im],
            [unitary_matrix[(1, 0)].im, unitary_matrix[(1, 1)].im],
        ],
        // column major version
        // real: [
        //     [unitary_matrix[(0, 0)].re, unitary_matrix[(1, 0)].re],
        //     [unitary_matrix[(0, 1)].re, unitary_matrix[(1, 1)].re],
        // ],
        // imag: [
        //     [unitary_matrix[(0, 0)].im, unitary_matrix[(1, 0)].im],
        //     [unitary_matrix[(0, 1)].im, unitary_matrix[(1, 1)].im],
        // ],
    };
    unsafe { quest_sys::unitary(qureg.quest_qureg, *operation.qubit() as i32, complex_matrix) };
    Ok(())
}

pub fn execute_generic_three_qubit_operation(
    operation: &ThreeQubitGateOperation,
    qureg: &mut Qureg,
) -> Result<(), RoqoqoBackendError> {
    let unitary_matrix = operation.unitary_matrix()?;
    let mut complex_matrix = ComplexMatrixN::new(3_u32);
    for ((row, column), value) in unitary_matrix.indexed_iter() {
        complex_matrix.set(row, column, *value).map_err(|err| {
            RoqoqoBackendError::GenericError {
                msg: err.to_string(),
            }
        })?;
    }
    unsafe {
        quest_sys::multiQubitUnitary(
            qureg.quest_qureg,
            (*operation.target() as i32).borrow_mut(),
            1,
            complex_matrix.complex_matrix,
        )
    };
    Ok(())
}

pub fn execute_generic_multi_qubit_operation(
    operation: &MultiQubitGateOperation,
    qureg: &mut Qureg,
) -> Result<(), RoqoqoBackendError> {
    let unitary_matrix = operation.unitary_matrix()?;
    let number_qubits = operation.qubits().len() as i32;
    let mut complex_matrix = ComplexMatrixN::new(number_qubits as u32);
    for ((row, column), value) in unitary_matrix.indexed_iter() {
        complex_matrix.set(row, column, *value).map_err(|err| {
            RoqoqoBackendError::GenericError {
                msg: err.to_string(),
            }
        })?;
    }
    let mut targets: Vec<i32> = operation
        .qubits()
        .iter()
        .cloned()
        .map(|x| x as i32)
        .collect();
    unsafe {
        quest_sys::multiQubitUnitary(
            qureg.quest_qureg,
            targets.as_mut_ptr(),
            number_qubits,
            complex_matrix.complex_matrix,
        )
    };
    Ok(())
}

pub fn execute_generic_two_qubit_operation(
    operation: &TwoQubitGateOperation,
    qureg: &mut Qureg,
) -> Result<(), RoqoqoBackendError> {
    let unitary_matrix = operation.unitary_matrix()?;
    let complex_matrix = quest_sys::ComplexMatrix4 {
        // row major version only used for Complex2/4/N intio
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
            [
                unitary_matrix[(0, 0)].im,
                unitary_matrix[(0, 1)].im,
                unitary_matrix[(0, 2)].im,
                unitary_matrix[(0, 3)].im,
            ],
            [
                unitary_matrix[(1, 0)].im,
                unitary_matrix[(1, 1)].im,
                unitary_matrix[(1, 2)].im,
                unitary_matrix[(1, 3)].im,
            ],
            [
                unitary_matrix[(2, 0)].im,
                unitary_matrix[(2, 1)].im,
                unitary_matrix[(2, 2)].im,
                unitary_matrix[(2, 3)].im,
            ],
            [
                unitary_matrix[(3, 0)].im,
                unitary_matrix[(3, 1)].im,
                unitary_matrix[(3, 2)].im,
                unitary_matrix[(3, 3)].im,
            ],
        ],
        // column major version
        // real: [
        //     [
        //         unitary_matrix[(0, 0)].re,
        //         unitary_matrix[(1, 0)].re,
        //         unitary_matrix[(2, 0)].re,
        //         unitary_matrix[(3, 0)].re,
        //     ],
        //     [
        //         unitary_matrix[(0, 1)].re,
        //         unitary_matrix[(1, 1)].re,
        //         unitary_matrix[(2, 1)].re,
        //         unitary_matrix[(3, 1)].re,
        //     ],
        //     [
        //         unitary_matrix[(0, 2)].re,
        //         unitary_matrix[(1, 2)].re,
        //         unitary_matrix[(2, 2)].re,
        //         unitary_matrix[(3, 2)].re,
        //     ],
        //     [
        //         unitary_matrix[(0, 3)].re,
        //         unitary_matrix[(1, 3)].re,
        //         unitary_matrix[(2, 3)].re,
        //         unitary_matrix[(3, 3)].re,
        //     ],
        // ],
        // imag: [
        //     [
        //         unitary_matrix[(0, 0)].im,
        //         unitary_matrix[(1, 0)].im,
        //         unitary_matrix[(2, 0)].im,
        //         unitary_matrix[(3, 0)].im,
        //     ],
        //     [
        //         unitary_matrix[(0, 1)].im,
        //         unitary_matrix[(1, 1)].im,
        //         unitary_matrix[(2, 1)].im,
        //         unitary_matrix[(3, 1)].im,
        //     ],
        //     [
        //         unitary_matrix[(0, 2)].im,
        //         unitary_matrix[(1, 2)].im,
        //         unitary_matrix[(2, 2)].im,
        //         unitary_matrix[(3, 2)].im,
        //     ],
        //     [
        //         unitary_matrix[(0, 3)].im,
        //         unitary_matrix[(1, 3)].im,
        //         unitary_matrix[(2, 3)].im,
        //         unitary_matrix[(3, 3)].im,
        //     ],
        // ],
    };
    unsafe {
        quest_sys::twoQubitUnitary(
            qureg.quest_qureg,
            *operation.target() as i32,
            *operation.control() as i32,
            complex_matrix,
        )
    }
    Ok(())
}

pub fn execute_generic_single_qubit_noise(
    operation: &PragmaGeneralNoise,
    qureg: &mut Qureg,
) -> Result<(), RoqoqoBackendError> {
    if !qureg.is_density_matrix {
        return Err(RoqoqoBackendError::GenericError {
            msg: "Noise operator can not be applied to state vector quantum register".to_string(),
        });
    }
    let number_qubits = qureg.number_qubits();
    let unitary_matrix = operation.superoperator()?;
    let complex_matrix = quest_sys::ComplexMatrix4 {
        // Row major version
        real: [
            [
                unitary_matrix[(0, 0)],
                unitary_matrix[(0, 1)],
                unitary_matrix[(0, 2)],
                unitary_matrix[(0, 3)],
            ],
            [
                unitary_matrix[(1, 0)],
                unitary_matrix[(1, 1)],
                unitary_matrix[(1, 2)],
                unitary_matrix[(1, 3)],
            ],
            [
                unitary_matrix[(2, 0)],
                unitary_matrix[(2, 1)],
                unitary_matrix[(2, 2)],
                unitary_matrix[(2, 3)],
            ],
            [
                unitary_matrix[(3, 0)],
                unitary_matrix[(3, 1)],
                unitary_matrix[(3, 2)],
                unitary_matrix[(3, 3)],
            ],
        ],
        // Column major version
        // real: [
        //     [
        //         unitary_matrix[(0, 0)],
        //         unitary_matrix[(1, 0)],
        //         unitary_matrix[(2, 0)],
        //         unitary_matrix[(3, 0)],
        //     ],
        //     [
        //         unitary_matrix[(0, 1)],
        //         unitary_matrix[(1, 1)],
        //         unitary_matrix[(2, 1)],
        //         unitary_matrix[(3, 1)],
        //     ],
        //     [
        //         unitary_matrix[(0, 2)],
        //         unitary_matrix[(1, 2)],
        //         unitary_matrix[(2, 2)],
        //         unitary_matrix[(3, 2)],
        //     ],
        //     [
        //         unitary_matrix[(0, 3)],
        //         unitary_matrix[(1, 3)],
        //         unitary_matrix[(2, 3)],
        //         unitary_matrix[(3, 3)],
        //     ],
        // ],
        imag: [
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ],
    };
    unsafe {
        quest_sys::statevec_twoQubitUnitary(
            qureg.quest_qureg,
            *operation.qubit() as i32,
            *operation.qubit() as i32 + number_qubits as i32,
            complex_matrix,
        )
    }
    Ok(())
}
