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
        if self.is_density_matrix {
            for index in 0..dimension {
                unsafe {
                    probabilites.push(
                        quest_sys::getDensityAmp(self.quest_qureg, index.into(), index.into()).real,
                    )
                };
            }
        } else {
            for index in 0..dimension {
                unsafe { probabilites.push(quest_sys::getProbAmp(self.quest_qureg, index.into())) };
            }
        }
        probabilites
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
