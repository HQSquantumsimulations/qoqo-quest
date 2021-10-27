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

use crate::Qureg;
use num_complex::Complex64;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use roqoqo::operations::*;
use roqoqo::registers::{BitOutputRegister, BitRegister, ComplexRegister};
use roqoqo::RoqoqoBackendError;
use std::collections::HashMap;

pub fn execute_pragma_repeated_measurement(
    operation: &PragmaRepeatedMeasurement,
    qureg: &mut Qureg,
    bit_registers: &mut HashMap<String, BitRegister>,
    bit_registers_output: &mut HashMap<String, BitOutputRegister>,
) -> Result<(), RoqoqoBackendError> {
    let index_dict = operation.qubit_mapping();
    let number_qubits = qureg.number_qubits();
    let probabilities = qureg.probabilites();
    let distribution =
        WeightedIndex::new(&probabilities).map_err(|err| RoqoqoBackendError::GenericError {
            msg: format!("Probabilites from quantum register {:?}", err),
        })?;
    let mut rng = thread_rng();
    let output_register: &mut BitOutputRegister = bit_registers_output
        .get_mut(operation.readout())
        .ok_or(RoqoqoBackendError::GenericError {
            msg: format!(
                "Trying to write readout to non-existent register {}",
                operation.readout()
            ),
        })?;
    bit_registers.remove(operation.readout());
    match index_dict {
        None => {
            for _ in 0..*operation.number_measurements() {
                let index = distribution.sample(&mut rng);
                output_register.push(index_to_qubits(index, number_qubits))
            }
        }
        Some(mapping) => {
            for _ in 0..*operation.number_measurements() {
                let index = distribution.sample(&mut rng);
                let tmp_output = index_to_qubits(index, number_qubits);
                let mut new_output: Vec<bool> = vec![false; number_qubits as usize];
                for (k, val) in tmp_output.iter().enumerate() {
                    let tmp_index = match mapping.get(&k) {
                        Some(ind) => ind,
                        None => &k,
                    };
                    new_output[*tmp_index] = *val;
                }
                output_register.push(new_output);
            }
        }
    }
    Ok(())
}

pub fn execute_pragma_set_state_vector(
    operation: &PragmaSetStateVector,
    qureg: &mut Qureg,
) -> Result<(), RoqoqoBackendError> {
    let statevec = operation.statevector();
    let num_amps: i64 = statevec.len() as i64;
    if num_amps != 2_i64.pow(qureg.number_qubits()) {
        return Err(RoqoqoBackendError::GenericError{msg: format!("Can not set state vector number of qubits of statevector {} differs from number of qubits in qubit register {}", num_amps, qureg.number_qubits())});
    }
    if qureg.is_density_matrix {
        let mut reals: Vec<f64> = Vec::new();
        let mut imags: Vec<f64> = Vec::new();
        // iterate over ket state vector to the left of the matrix product
        // to reconstruct density matrix
        // Variant for row major order
        // for value_left in statevec.iter() {
        //     // create real and imaginary entries for `row` by multiplying with bra form of statevector
        //     reals.extend(
        //         statevec
        //             .iter()
        //             .map(|value_right| (value_left * value_right.conj()).re),
        //     );
        //     imags.extend(
        //         statevec
        //             .iter()
        //             .map(|value_right| (value_left * value_right.conj()).im),
        //     );
        // }
        // // Variant for column major order
        for value_right in statevec.iter() {
            // create real and imaginary entries for `row` by multiplying with bra form of statevector
            reals.extend(
                statevec
                    .iter()
                    .map(|value_left| (value_left * value_right.conj()).re),
            );
            imags.extend(
                statevec
                    .iter()
                    .map(|value_left| (value_left * value_right.conj()).im),
            );
        }
        unsafe {
            quest_sys::initStateFromAmps(qureg.quest_qureg, reals.as_mut_ptr(), imags.as_mut_ptr())
        }
        Ok(())
    } else {
        let startind: i64 = 0;
        let mut reals: Vec<f64> = statevec.iter().map(|x| x.re).collect();
        let mut imags: Vec<f64> = statevec.iter().map(|x| x.im).collect();
        unsafe {
            quest_sys::setAmps(
                qureg.quest_qureg,
                startind,
                reals.as_mut_ptr(),
                imags.as_mut_ptr(),
                num_amps,
            )
        }
        Ok(())
    }
}

pub fn execute_pragma_set_density_matrix(
    operation: &PragmaSetDensityMatrix,
    qureg: &mut Qureg,
) -> Result<(), RoqoqoBackendError> {
    let density_matrix = operation.density_matrix();
    let (num_amps, _) = density_matrix.dim();
    if num_amps != 2_i64.pow(qureg.number_qubits()) as usize {
        return Err(RoqoqoBackendError::GenericError{msg: format!("Can not set state vector number of qubits of statevector {} differs from number of qubits in qubit register {}", num_amps, qureg.number_qubits())});
    }
    if qureg.is_density_matrix {
        // Variant for row major order (ndarray default row major)
        // let mut reals: Vec<f64> = density_matrix.iter().map(|x| x.re).collect();
        // let mut imags: Vec<f64> = density_matrix.iter().map(|x| x.im).collect();

        // // Variant for column major order (transpose ndarray default row major)
        let mut reals: Vec<f64> = density_matrix.t().iter().map(|x| x.re).collect();
        let mut imags: Vec<f64> = density_matrix.t().iter().map(|x| x.im).collect();

        unsafe {
            quest_sys::initStateFromAmps(qureg.quest_qureg, reals.as_mut_ptr(), imags.as_mut_ptr())
        }
        Ok(())
    } else {
        Err(RoqoqoBackendError::GenericError {
            msg: "Density matrix can not be set on state vector quantum register".to_string(),
        })
    }
}

// pub fn execute_pragma_random_noise(
//     operation: &PragmaRandomNoise,
//     qureg: &mut Qureg,
// ) -> Result<(), RoqoqoBackendError> {
//     let mut rng = thread_rng();
//     let r0 = rng.gen_range(0.0..1.0);
//     let rates = [
//         operation.depolarising_rate().float()? / 4.0,
//         operation.depolarising_rate().float()? / 4.0,
//         (operation.depolarising_rate().float()? / 4.0) + (operation.dephasing_rate().float()?),
//     ];
//     let gate_time = operation.gate_time().float()?;
//     let mut probas: [f64; 3] = [0.0; 3];
//     for (i, rate) in rates.iter().enumerate() {
//         if rate >= &f64::EPSILON {
//             probas[i] = *rate;
//         }
//     }
//     let sum: f64 = rates.clone().iter().sum();
//     if r0 < (gate_time * sum * -1.0) - 1.0 {
//         Ok(())
//     } else {
//         let choices = [1, 2, 3];
//         let distribution =
//             WeightedIndex::new(&probas).map_err(|err| RoqoqoBackendError::GenericError {
//                 msg: format!("Probabilites from quantum register {:?}", err),
//             })?;
//         match choices[distribution.sample(&mut rng)] {
//             1 => {
//                 unsafe {
//                     quest_sys::pauliX(
//                         qureg.quest_qureg,
//                         *operation.qubit() as ::std::os::raw::c_int,
//                     )
//                 }
//                 Ok(())
//             }
//             2 => {
//                 unsafe {
//                     quest_sys::pauliY(
//                         qureg.quest_qureg,
//                         *operation.qubit() as ::std::os::raw::c_int,
//                     )
//                 }
//                 Ok(())
//             }
//             3 => {
//                 unsafe {
//                     quest_sys::pauliZ(
//                         qureg.quest_qureg,
//                         *operation.qubit() as ::std::os::raw::c_int,
//                     )
//                 }
//                 Ok(())
//             }
//             x => Err(RoqoqoBackendError::GenericError {
//                 msg: format!("Incorrect Pauli selected: {:?}", x),
//             }),
//         }
//     }
// }

pub fn execute_pragma_get_state_vector(
    operation: &PragmaGetStateVector,
    qureg: &mut Qureg,
    complex_registers: &mut HashMap<String, ComplexRegister>,
) -> Result<(), RoqoqoBackendError> {
    let readout = operation.readout();
    if qureg.is_density_matrix {
        Err(RoqoqoBackendError::GenericError {
            msg: "Trying to obtain state vector from density matrix quantum register".to_string(),
        })
    } else {
        let mut statevector: Vec<Complex64> =
            Vec::with_capacity(2_usize.pow(qureg.number_qubits()));
        for i in 0..2_usize.pow(qureg.number_qubits()) as i64 {
            statevector.push(Complex64::new(
                unsafe { quest_sys::getRealAmp(qureg.quest_qureg, i) },
                unsafe { quest_sys::getImagAmp(qureg.quest_qureg, i) },
            ))
        }
        complex_registers.insert(readout.clone(), statevector);
        Ok(())
    }
}

pub fn execute_pragma_get_density_matrix(
    operation: &PragmaGetDensityMatrix,
    qureg: &mut Qureg,
    complex_registers: &mut HashMap<String, ComplexRegister>,
) -> Result<(), RoqoqoBackendError> {
    let readout = operation.readout();
    let dimension = 2_i64.pow(qureg.number_qubits());
    let mut density_matrix_flattened_row_major: Vec<Complex64> =
        Vec::with_capacity(4_usize.pow(qureg.number_qubits()));
    if qureg.is_density_matrix {
        for row in 0..dimension {
            for column in 0..dimension {
                density_matrix_flattened_row_major.push(Complex64::new(
                    unsafe { quest_sys::getDensityAmp(qureg.quest_qureg, row, column).real },
                    unsafe { quest_sys::getDensityAmp(qureg.quest_qureg, row, column).imag },
                ))
            }
        }
    } else {
        for row in 0..dimension {
            for column in 0..dimension {
                let value = Complex64::new(
                    unsafe { quest_sys::getRealAmp(qureg.quest_qureg, row) },
                    unsafe { quest_sys::getImagAmp(qureg.quest_qureg, row) },
                ) * Complex64::new(
                    unsafe { quest_sys::getRealAmp(qureg.quest_qureg, column) },
                    unsafe { quest_sys::getImagAmp(qureg.quest_qureg, column) },
                )
                .conj();
                density_matrix_flattened_row_major.push(value);
            }
        }
    }
    complex_registers.insert(readout.clone(), density_matrix_flattened_row_major);
    Ok(())
}

#[inline]
fn index_to_qubits(index: usize, number_qubits: u32) -> Vec<bool> {
    let mut binary_list: Vec<bool> = Vec::with_capacity(number_qubits as usize);
    for k in 0..number_qubits {
        // (index // 2**k) % 2 => 0 -> false 1 -> true
        binary_list.push(index.div_euclid(2usize.pow(k)).rem_euclid(2) == 1)
    }
    binary_list
}
