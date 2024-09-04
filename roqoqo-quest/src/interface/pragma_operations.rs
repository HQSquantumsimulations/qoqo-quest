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
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use roqoqo::devices::Device;
use roqoqo::operations::*;
use roqoqo::registers::{BitOutputRegister, BitRegister, ComplexRegister, FloatRegister};
use roqoqo::Circuit;
use roqoqo::RoqoqoBackendError;
use std::collections::HashMap;

/// Numerical accuracy for ignoring negative occupation probabilities
///
/// Negative probabilities with a smaller absolute value will be interpreted as 0.
/// Negative probabilities with a larger absolute value will cause an error.
const NEGATIVE_PROBABILITIES_CUTOFF: f64 = -1.0e-14;

// type alias for the signature of call_circuit_with_device, to decouple this module from mod.rs and
// avoid circular imports
type CallCircuitWithDevice = fn(
    &Circuit,
    &mut Qureg,
    &mut HashMap<String, BitRegister>,
    &mut HashMap<String, FloatRegister>,
    &mut HashMap<String, ComplexRegister>,
    &mut HashMap<String, BitOutputRegister>,
    &mut Option<Box<dyn roqoqo::devices::Device>>,
) -> Result<(), RoqoqoBackendError>;

pub fn execute_pragma_repeated_measurement(
    operation: &PragmaRepeatedMeasurement,
    qureg: &mut Qureg,
    bit_registers: &mut HashMap<String, BitRegister>,
    bit_registers_output: &mut HashMap<String, BitOutputRegister>,
) -> Result<(), RoqoqoBackendError> {
    let index_dict = operation.qubit_mapping();
    let number_qubits = qureg.number_qubits();
    let mut probabilities = qureg.probabilites();
    sanitize_probabilities(&mut probabilities)?;

    let distribution =
        WeightedIndex::new(&probabilities).map_err(|err| RoqoqoBackendError::GenericError {
            msg: format!("Probabilites from quantum register {:?}", err),
        })?;
    let mut rng = thread_rng();
    let existing_register = bit_registers
        .get(operation.readout())
        .map(|x| x.to_owned())
        .unwrap_or_else(|| vec![false; usize::try_from(number_qubits).unwrap()]);
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
                let mut tmp = existing_register.clone();
                for (a, b) in index_to_qubits(index, number_qubits)
                    .into_iter()
                    .enumerate()
                {
                    tmp[a] = b
                }
                output_register.push(tmp)
            }
        }
        Some(mapping) => {
            for _ in 0..*operation.number_measurements() {
                let index = distribution.sample(&mut rng);
                let tmp_output = index_to_qubits(index, number_qubits);
                let mut new_output: Vec<bool> = existing_register.clone();
                for (k, val) in tmp_output.into_iter().enumerate() {
                    let tmp_index = match mapping.get(&k) {
                        Some(ind) => ind,
                        None => &k,
                    };
                    new_output[*tmp_index] = val;
                }
                output_register.push(new_output);
            }
        }
    }
    Ok(())
}

pub fn execute_replaced_repeated_measurement(
    operation: &PragmaRepeatedMeasurement,
    qureg: &mut Qureg,
    bit_registers: &mut HashMap<String, BitRegister>,
    bit_registers_output: &mut HashMap<String, BitOutputRegister>,
) -> Result<(), RoqoqoBackendError> {
    let index_dict = operation.qubit_mapping();
    let number_qubits = qureg.number_qubits();
    let mut probabilities = qureg.probabilites();
    sanitize_probabilities(&mut probabilities)?;

    let distribution =
        WeightedIndex::new(&probabilities).map_err(|err| RoqoqoBackendError::GenericError {
            msg: format!("Probabilites from quantum register {:?}", err),
        })?;
    let mut rng = thread_rng();
    let existing_register = bit_registers
        .get(operation.readout())
        .map(|x| x.to_owned())
        .unwrap_or_else(|| {
            vec![
                false;
                usize::try_from(number_qubits)
                    .expect("Cannot transform u32 to usize on this platform")
            ]
        });
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
            panic!("Internal bug in roqoqo-quest. qubits not defined for repeated measurement.");
        }
        Some(mapping) => {
            for _ in 0..*operation.number_measurements() {
                let index = distribution.sample(&mut rng);
                let tmp_output = index_to_qubits(index, number_qubits);
                let mut new_output: Vec<bool> = existing_register.clone();
                for (k, val) in tmp_output.into_iter().enumerate() {
                    if let Some(ind) = mapping.get(&k) {
                        new_output[*ind] = val;
                    }
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
        return Err(RoqoqoBackendError::GenericError{
            msg: format!(
                "Can not set statevector: number of qubits of statevector ({}) differs from number of qubits in qubit register ({}).",
                num_amps,
                qureg.number_qubits()
            )
        });
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
        return Err(RoqoqoBackendError::GenericError {
            msg: format!(
                "Can not set density matrix: number of qubits of density matrix ({}) differs from \
                 number of qubits in qubit register ({}).",
                num_amps,
                qureg.number_qubits()
            ),
        });
    }
    if qureg.is_density_matrix {
        // Variant for row major order (ndarray default row major)
        // let mut reals: Vec<f64> = density_matrix.iter().map(|x| x.re).collect();
        // let mut imags: Vec<f64> = density_matrix.iter().map(|x| x.im).collect();

        // // Variant for column major order (transpose ndarray default row major)
        let mut reals: Vec<f64> = density_matrix.t().iter().map(|x| x.re).collect();
        let mut imags: Vec<f64> = density_matrix.t().iter().map(|x| x.im).collect();
        let start_row: ::std::os::raw::c_longlong = 0;
        let start_column: ::std::os::raw::c_longlong = 0;
        let number_amplitudes: ::std::os::raw::c_longlong =
            imags.len() as ::std::os::raw::c_longlong;
        unsafe {
            quest_sys::setDensityAmps(
                qureg.quest_qureg,
                start_row,
                start_column,
                reals.as_mut_ptr(),
                imags.as_mut_ptr(),
                number_amplitudes,
            )
        }
        Ok(())
    } else {
        Err(RoqoqoBackendError::GenericError {
            msg: "Density matrix can not be set on statevector quantum register".to_string(),
        })
    }
}

pub fn execute_pragma_input_bit(
    operation: &InputBit,
    bit_registers: &mut HashMap<String, BitRegister>,
) -> Result<(), RoqoqoBackendError> {
    let existing_register: &mut BitRegister =
        bit_registers
            .get_mut(operation.name())
            .ok_or(RoqoqoBackendError::GenericError {
                msg: format!(
                    "Trying to write readout to non-existent register {}",
                    operation.name()
                ),
            })?;
    if operation.index() >= &existing_register.len() {
        return Err(RoqoqoBackendError::GenericError {
            msg: format!(
                "Trying to write readout to non-existent index {}",
                operation.index()
            ),
        });
    }
    existing_register[*operation.index()] = *operation.value();
    Ok(())
}

pub fn execute_pragma_random_noise(
    operation: &PragmaRandomNoise,
    qureg: &mut Qureg,
) -> Result<(), RoqoqoBackendError> {
    let mut rng = thread_rng();
    let r0 = rng.gen_range(0.0..1.0);
    let rates = [
        operation.depolarising_rate().float()? / 4.0,
        operation.depolarising_rate().float()? / 4.0,
        (operation.depolarising_rate().float()? / 4.0) + (operation.dephasing_rate().float()?),
    ];
    let gate_time = operation.gate_time().float()?;
    let mut probas: [f64; 3] = [0.0; 3];
    for (i, rate) in rates.iter().enumerate() {
        if rate >= &f64::EPSILON {
            probas[i] = *rate;
        }
    }
    let sum: f64 = rates.clone().iter().sum();
    if r0 < (gate_time * sum) - 1.0 {
        Ok(())
    } else {
        let choices = [1, 2, 3];
        let distribution =
            WeightedIndex::new(probas).map_err(|err| RoqoqoBackendError::GenericError {
                msg: format!("Probabilites from quantum register {:?}", err),
            })?;
        match choices[distribution.sample(&mut rng)] {
            1 => {
                unsafe {
                    quest_sys::pauliX(
                        qureg.quest_qureg,
                        *operation.qubit() as ::std::os::raw::c_int,
                    )
                }
                Ok(())
            }
            2 => {
                unsafe {
                    quest_sys::pauliY(
                        qureg.quest_qureg,
                        *operation.qubit() as ::std::os::raw::c_int,
                    )
                }
                Ok(())
            }
            3 => {
                unsafe {
                    quest_sys::pauliZ(
                        qureg.quest_qureg,
                        *operation.qubit() as ::std::os::raw::c_int,
                    )
                }
                Ok(())
            }
            x => Err(RoqoqoBackendError::GenericError {
                msg: format!("Incorrect Pauli selected: {:?}", x),
            }),
        }
    }
}

pub fn execute_pragma_loop(
    operation: &PragmaLoop,
    qureg: &mut Qureg,
    bit_registers: &mut HashMap<String, BitRegister>,
    float_registers: &mut HashMap<String, Vec<f64>>,
    complex_registers: &mut HashMap<String, ComplexRegister>,
    bit_registers_output: &mut HashMap<String, BitOutputRegister>,
    device: &mut Option<Box<dyn Device>>,
) -> Result<(), RoqoqoBackendError> {
    let repetitions: f64 = *operation.repetitions().float()?;
    let floor_repetitions: i32 = repetitions.floor() as i32;
    let floor_repetitions: usize = if floor_repetitions > 0 {
        floor_repetitions as usize
    } else {
        0
    };
    for _ in 0..floor_repetitions {
        crate::interface::call_circuit_with_device(
            operation.circuit(),
            qureg,
            bit_registers,
            float_registers,
            complex_registers,
            bit_registers_output,
            device,
        )?;
    }
    Ok(())
}

pub fn execute_pragma_get_state_vector(
    operation: &PragmaGetStateVector,
    qureg: &mut Qureg,
    complex_registers: &mut HashMap<String, ComplexRegister>,
) -> Result<(), RoqoqoBackendError> {
    let readout = operation.readout();
    let statevector = qureg.state_vector()?;
    complex_registers.insert(readout.clone(), statevector);
    Ok(())
}

pub fn execute_pragma_get_density_matrix(
    operation: &PragmaGetDensityMatrix,
    qureg: &mut Qureg,
    complex_registers: &mut HashMap<String, ComplexRegister>,
) -> Result<(), RoqoqoBackendError> {
    let readout = operation.readout();
    let density_matrix_flattened_row_major = qureg.density_matrix_flattened_row_major()?;
    complex_registers.insert(readout.clone(), density_matrix_flattened_row_major);
    Ok(())
}

#[inline]
fn index_to_qubits(index: usize, number_qubits: u32) -> Vec<bool> {
    let mut binary_list: Vec<bool> = Vec::with_capacity(number_qubits as usize);
    for k in 0..number_qubits {
        // (index // 2**k) % 2 => 0 -> false 1 -> true
        // Converts index to a binary number
        binary_list.push(index.div_euclid(2usize.pow(k)).rem_euclid(2) == 1)
    }
    binary_list
}

pub fn execute_get_pauli_prod(
    op: &PragmaGetPauliProduct,
    qureg: &mut Qureg,
    float_registers: &mut HashMap<String, Vec<f64>>,
    bit_registers: &mut HashMap<String, Vec<bool>>,
    complex_registers: &mut HashMap<String, Vec<num_complex::Complex<f64>>>,
    bit_registers_output: &mut HashMap<String, Vec<Vec<bool>>>,
    device: &mut Option<Box<dyn Device>>,
) -> Result<(), RoqoqoBackendError> {
    if op.qubit_paulis().is_empty() {
        float_registers.insert(op.readout().clone(), vec![1.0]);
        return Ok(());
    }
    let workspace_pp = Qureg::new(qureg.number_qubits(), qureg.is_density_matrix);

    let mut qubits: Vec<i32> = op
        .qubit_paulis()
        .keys()
        .cloned()
        .map(|x| x as i32)
        .collect();
    let mut paulis: Vec<u32> = op
        .qubit_paulis()
        .values()
        .cloned()
        .map(|x| x as u32)
        .collect();

    let pp = if !op.circuit().is_empty() {
        let mut workspace = Qureg::new(qureg.number_qubits(), qureg.is_density_matrix);
        unsafe {
            quest_sys::cloneQureg(workspace.quest_qureg, qureg.quest_qureg);
        }
        crate::interface::call_circuit_with_device(
            op.circuit(),
            &mut workspace,
            bit_registers,
            float_registers,
            complex_registers,
            bit_registers_output,
            device,
        )?;
        unsafe {
            let pp = quest_sys::calcExpecPauliProd(
                workspace.quest_qureg,
                qubits.as_mut_ptr(),
                paulis.as_mut_ptr(),
                qubits.len() as i32,
                workspace_pp.quest_qureg,
            );
            drop(workspace);
            drop(workspace_pp);
            pp
        }
    } else {
        unsafe {
            let pp = quest_sys::calcExpecPauliProd(
                qureg.quest_qureg,
                qubits.as_mut_ptr(),
                paulis.as_mut_ptr(),
                qubits.len() as i32,
                workspace_pp.quest_qureg,
            );
            drop(workspace_pp);
            pp
        }
    };

    float_registers.insert(op.readout().clone(), vec![pp]);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn execute_get_occupation_probability(
    op: &PragmaGetOccupationProbability,
    qureg: &mut Qureg,
    float_registers: &mut HashMap<String, Vec<f64>>,
    bit_registers: &mut HashMap<String, Vec<bool>>,
    complex_registers: &mut HashMap<String, Vec<num_complex::Complex<f64>>>,
    bit_registers_output: &mut HashMap<String, Vec<Vec<bool>>>,
    device: &mut Option<Box<dyn Device>>,
    circuit_handler: CallCircuitWithDevice,
) -> Result<(), RoqoqoBackendError> {
    unsafe {
        let mut workspace = Qureg::new(qureg.number_qubits(), qureg.is_density_matrix);
        match op.circuit() {
            Some(x) => {
                circuit_handler(
                    x,
                    qureg,
                    bit_registers,
                    float_registers,
                    complex_registers,
                    bit_registers_output,
                    device,
                )?;
            }
            None => {}
        }
        quest_sys::cloneQureg(workspace.quest_qureg, qureg.quest_qureg);
        let probas: Vec<f64>;
        if qureg.is_density_matrix {
            let op = PragmaGetDensityMatrix::new(op.readout().clone(), None);
            let mut register: HashMap<String, ComplexRegister> = HashMap::new();
            execute_pragma_get_density_matrix(&op, &mut workspace, &mut register)?;
            match register.get(op.readout()) {
                Some(p) => probas = p.iter().map(|x| x.re).collect(),
                None => {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: "Issue in get_density_matrix".to_string(),
                    });
                }
            }
        } else {
            let op = PragmaGetStateVector::new(op.readout().clone(), None);
            let mut register: HashMap<String, ComplexRegister> = HashMap::new();
            execute_pragma_get_state_vector(&op, &mut workspace, &mut register)?;
            match register.get(op.readout()) {
                Some(p) => probas = p.iter().map(|x| x.re).collect(),
                None => {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: "Issue in get_state_vector".to_string(),
                    });
                }
            }
        }
        float_registers.insert(op.readout().clone(), probas);
    }
    Ok(())
}

pub fn execute_pragma_conditional(
    op: &PragmaConditional,
    qureg: &mut Qureg,
    float_registers: &mut HashMap<String, Vec<f64>>,
    bit_registers: &mut HashMap<String, Vec<bool>>,
    complex_registers: &mut HashMap<String, Vec<num_complex::Complex<f64>>>,
    bit_registers_output: &mut HashMap<String, Vec<Vec<bool>>>,
    device: &mut Option<Box<dyn Device>>,
    circuit_handler: CallCircuitWithDevice,
) -> Result<(), RoqoqoBackendError> {
    match bit_registers.get(op.condition_register()) {
        None => {
            return Err(RoqoqoBackendError::GenericError {
                msg: format!(
                    "Conditional register {:?} not found in classical bit registers.",
                    op.condition_register()
                ),
            });
        }
        Some(x) => {
            if x[*op.condition_index()] {
                circuit_handler(
                    op.circuit(),
                    qureg,
                    bit_registers,
                    float_registers,
                    complex_registers,
                    bit_registers_output,
                    device,
                )?;
            }
        }
    }
    Ok(())
}

#[inline]
/// Sanitizes negative occupation probabilities
///
/// Setting negative probabilites with an absolute value less than a threshold to 0
fn sanitize_probabilities(probabilities: &mut Vec<f64>) -> Result<(), RoqoqoBackendError> {
    for val in probabilities.iter_mut() {
        if *val < NEGATIVE_PROBABILITIES_CUTOFF {
            return Err(RoqoqoBackendError::GenericError {
                msg: format!(
                    "Negative state occupation probabilites encountered {:?}",
                    probabilities
                ),
            });
        } else if *val < 0.0 {
            *val = 0.0
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn sanitize_probabilities_nothing() {
        let mut probabilities = vec![0.3, 0.4, 0.1, 0.2];
        let res = sanitize_probabilities(&mut probabilities);
        assert!(res.is_ok());
        assert_eq!(probabilities, vec![0.3, 0.4, 0.1, 0.2])
    }

    #[test]
    fn sanitize_probabilities_set_to_zero() {
        let mut probabilities = vec![0.3, 0.4, 0.23 * NEGATIVE_PROBABILITIES_CUTOFF, 0.2];
        let res = sanitize_probabilities(&mut probabilities);
        assert!(res.is_ok());
        assert_eq!(probabilities, vec![0.3, 0.4, 0.0, 0.2])
    }

    #[test]
    fn sanitize_probabilities_set_error() {
        let mut probabilities = vec![0.3, 0.4, 1.3 * NEGATIVE_PROBABILITIES_CUTOFF, 0.2];
        let res = sanitize_probabilities(&mut probabilities);
        assert!(res.is_err());
    }
}
