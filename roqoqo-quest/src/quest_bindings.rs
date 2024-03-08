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
use roqoqo::RoqoqoBackendError;

/// Wrapper around QuEST quantum register
///
/// A wrapper around the quantum register struct of QuEST.
///
/// A state-vector quantum register contains the pure quantum state of the simulator.
/// A density-matrix quantum register contains the full density matrix
/// of a potentially mixed quantum state of the simulator.
///
#[derive(Debug, Clone)]
pub struct Qureg {
    /// Internally stored C QuEST environment.
    pub quest_env: quest_sys::QuESTEnv,
    /// Internally stores C QuEST quantum register
    pub quest_qureg: quest_sys::Qureg,
    /// Is a density matrix
    pub is_density_matrix: bool,
}

impl Qureg {
    /// Creates a new quantum register.
    ///
    /// # Arguments
    ///
    /// * `number_qubits` - The number of qubits in the quantum register.
    /// * `is_density_matrix` - Create a
    pub fn new(number_qubits: u32, is_density_matrix: bool) -> Self {
        unsafe {
            let quest_env = quest_sys::createQuESTEnv();
            let quest_qureg = if is_density_matrix {
                quest_sys::createDensityQureg(number_qubits as ::std::os::raw::c_int, quest_env)
            } else {
                quest_sys::createQureg(number_qubits as ::std::os::raw::c_int, quest_env)
            };
            Qureg {
                quest_env,
                quest_qureg,
                is_density_matrix,
            }
        }
    }

    /// Resets the quantum register to the default initial state |000...0>
    ///
    /// Works for state vector and density matrix registers.
    pub fn reset(&mut self) {
        unsafe {
            quest_sys::initClassicalState(self.quest_qureg, 0);
        }
    }

    /// Returns the number of qubits in the qureg.
    pub fn number_qubits(&self) -> u32 {
        self.quest_qureg.numQubitsRepresented as u32
    }

    /// Returns probability amplitudes for each state in the quantum register.
    ///
    /// Probability amplitudes give the probability that a quantum register collapses to the corresponding state after a measurement.
    pub fn probabilites(&self) -> Vec<f64> {
        let number_qubits = self.number_qubits();
        let dimension: u32 = 2u32.pow(number_qubits);
        let mut probabilites: Vec<f64> = Vec::with_capacity(dimension as usize);
        unsafe {
            quest_sys::copyStateFromGPU(self.quest_qureg);
        }
        if self.is_density_matrix {
            for index in 0..dimension {
                let density_index = index + index * dimension;
                unsafe {
                    let real_amp = *self
                        .quest_qureg
                        .stateVec
                        .real
                        .wrapping_add(density_index.try_into().expect("Indexing error"));
                    probabilites.push(real_amp);
                };
            }
        } else {
            for index in 0..dimension {
                unsafe {
                    let real_amp = *self
                        .quest_qureg
                        .stateVec
                        .real
                        .wrapping_add(index.try_into().expect("Indexing error"));
                    let imag_amp = *self
                        .quest_qureg
                        .stateVec
                        .imag
                        .wrapping_add(index.try_into().expect("Indexing error"));
                    probabilites.push(real_amp * real_amp + imag_amp * imag_amp);
                };
            }
        }
        probabilites
    }

    /// Returns the state_vector in the quantum register.
    pub fn state_vector(&self) -> Result<Vec<Complex64>, RoqoqoBackendError> {
        let number_qubits = self.number_qubits();
        let dimension: u32 = 2u32.pow(number_qubits);
        let mut statevector: Vec<Complex64> = Vec::with_capacity(dimension.try_into().unwrap());
        unsafe {
            quest_sys::copyStateFromGPU(self.quest_qureg);
        }
        if self.is_density_matrix {
            return Err(RoqoqoBackendError::GenericError {
                msg: "Trying to obtain state vector from density matrix quantum register"
                    .to_string(),
            });
        } else {
            for i in 0..2_usize.pow(self.number_qubits()) as i64 {
                unsafe {
                    let real_amp = *self
                        .quest_qureg
                        .stateVec
                        .real
                        .wrapping_add(i.try_into().expect("Indexing error"));
                    let imag_amp = *self
                        .quest_qureg
                        .stateVec
                        .imag
                        .wrapping_add(i.try_into().expect("Indexing error"));

                    statevector.push(Complex64::new(real_amp, imag_amp))
                }
            }
        }
        Ok(statevector)
    }

    /// Returns the density matrix in the quantum register in flattened form (row major).
    pub fn density_matrix_flattened_row_major(&self) -> Result<Vec<Complex64>, RoqoqoBackendError> {
        let number_qubits = self.number_qubits();
        let dimension: u32 = 2u32.pow(number_qubits);
        unsafe {
            quest_sys::copyStateFromGPU(self.quest_qureg);
        }
        let mut density_matrix_flattened_row_major: Vec<Complex64> =
            Vec::with_capacity(4_usize.pow(self.number_qubits()));
        if self.is_density_matrix {
            for row in 0..dimension {
                for column in 0..dimension {
                    let density_index = row + column * dimension;
                    unsafe {
                        let real_amp = *self
                            .quest_qureg
                            .stateVec
                            .real
                            .wrapping_add(density_index.try_into().expect("Indexing error"));
                        let imag_amp = *self
                            .quest_qureg
                            .stateVec
                            .imag
                            .wrapping_add(density_index.try_into().expect("Indexing error"));

                        density_matrix_flattened_row_major.push(Complex64::new(real_amp, imag_amp))
                    }
                }
            }
        } else {
            for row in 0..dimension {
                for column in 0..dimension {
                    let value = unsafe {
                        let real_amp_row = *self
                            .quest_qureg
                            .stateVec
                            .real
                            .wrapping_add(row.try_into().expect("Indexing error"));
                        let imag_amp_row = *self
                            .quest_qureg
                            .stateVec
                            .imag
                            .wrapping_add(row.try_into().expect("Indexing error"));
                        let real_amp_column = *self
                            .quest_qureg
                            .stateVec
                            .real
                            .wrapping_add(column.try_into().expect("Indexing error"));
                        let imag_amp_column = *self
                            .quest_qureg
                            .stateVec
                            .imag
                            .wrapping_add(column.try_into().expect("Indexing error"));

                        Complex64::new(real_amp_row, imag_amp_row)
                            * Complex64::new(real_amp_column, imag_amp_column).conj()
                    };
                    density_matrix_flattened_row_major.push(value);
                }
            }
        }
        Ok(density_matrix_flattened_row_major)
    }
}

impl Drop for Qureg {
    fn drop(&mut self) {
        unsafe {
            quest_sys::destroyQureg(self.quest_qureg, self.quest_env);
            quest_sys::destroyQuESTEnv(self.quest_env);
        }
    }
}

/// Wrapper around a QuEST C ComplexMatrixN
///
/// ComplexMatrices are the internal QuEST data type for arbitrary size ComplexMatrices
/// used in the simulation.
#[derive(Debug, Clone)]
pub struct ComplexMatrixN {
    /// Internally stored C ComplexMatrix.
    pub complex_matrix: quest_sys::ComplexMatrixN,
    /// The dimension of the complex matrix
    pub dimension: usize,
}

impl ComplexMatrixN {
    /// Creates a new ComplexMatrix for N qubits.
    ///
    /// QuEST internally uses ComplexMatrices of
    ///
    /// # Arguments
    ///
    /// * `number_qubits` - The number of qubits that determine the dimension of the matrix (2**number_qubits).
    pub fn new(number_qubits: u32) -> Self {
        unsafe {
            let complex_matrix = quest_sys::createComplexMatrixN(number_qubits as i32);
            let dimension = 2_usize.pow(number_qubits);
            ComplexMatrixN {
                complex_matrix,
                dimension,
            }
        }
    }

    /// Sets the value of a ComplexMatrixN at a position defined by row and column.
    ///
    /// # Arguments
    ///
    /// * `row` - The row value of the position in the matrix.
    /// * `column` - The column value of the position in the matrix.
    /// * `value` - The value that is set.
    pub fn set(&mut self, row: usize, column: usize, value: Complex64) -> Result<(), &'static str> {
        if row >= self.dimension || column >= self.dimension {
            return Err("Row or column index out of bounds");
        }
        let real = value.re;
        let imag = value.im;
        unsafe {
            let real_pointer = self.complex_matrix.real;
            let real_row_pointer = *real_pointer.add(row);
            *real_row_pointer.add(column) = real;
            let imag_pointer = self.complex_matrix.imag;
            let imag_row_pointer = *imag_pointer.add(row);
            *imag_row_pointer.add(column) = imag;
        }
        Ok(())
    }
}
impl Drop for ComplexMatrixN {
    fn drop(&mut self) {
        unsafe {
            quest_sys::destroyComplexMatrixN(self.complex_matrix);
        }
    }
}

/// Wrapper around a QuEST C Vector
///
/// ComplexMatrices are the internal QuEST data type for arbitrary size ComplexMatrices
/// used in the simulation.
#[derive(Debug, Clone)]
pub struct Vector {
    /// Internally stored C Vector.
    pub vector: quest_sys::Vector,
}

impl Vector {
    /// Creates a new Vector.
    ///
    /// QuEST internally uses ComplexMatrices of
    ///
    /// # Arguments
    ///
    /// * `number_qubits` - The number of qubits that determine the dimension of the matrix (2**number_qubits).
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        let vector = quest_sys::Vector { x, y, z };
        Vector { vector }
    }
}
