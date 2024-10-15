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

use roqoqo::operations::*;
use roqoqo::RoqoqoBackendError;
use std::collections::HashMap;
use std::collections::HashSet;

#[inline]
fn undefined_register_msg(reg_type: &str, reg_name: &str, op: &str, missing_op: &str) -> String {
    format!(
        "No {} readout register {} defined for {} operation. Did you forget to add a {} operation?",
        reg_type, reg_name, op, missing_op
    )
}

#[inline]
fn index_out_of_range_msg(index: &usize, length: &usize) -> String {
    format!(
        "Trying to write a qubit measurement at index {} on a bit register of length {}. Did you \
         define a large enough register with the DefinitionBit operation?",
        index, length
    )
}

// TODO this function computes the number of used qubits (used in the backend.rs to inizialize the
// quest qureg) by looking at the lengths of the defined registers. This does not seem to be
// accurate, because the length of a defined register is arbitrary and can be greater than
// necessary. e.g. if PragmaGetStateVector saves the state to a register, the register only needs to
// have length 2^N where N is the number of qubits, but can be arbitrarily large. If it is larger
// than needed, the number of used qubits will be overestimated. Will this cause a slowdown because
// quest needs to keep track of a larger density matrix than needed?
/// TODO docstring
pub fn get_number_used_qubits_and_registers_lengths(
    circuit: &Vec<&Operation>,
) -> Result<(usize, HashMap<String, usize>), RoqoqoBackendError> {
    let mut used_qubits: HashSet<usize> = HashSet::new();
    let mut bit_registers_lenghts: HashMap<String, usize> = HashMap::new();
    let mut float_registers_lenghts: HashMap<String, usize> = HashMap::new();
    let mut complex_registers_lenghts: HashMap<String, usize> = HashMap::new();

    for op in circuit {
        if let InvolvedQubits::Set(n) = op.involved_qubits() {
            used_qubits.extend(&n)
        }
        match op {
            Operation::DefinitionBit(def) => {
                if *def.is_output() {
                    bit_registers_lenghts.insert(def.name().clone(), *def.length());
                }
            }
            Operation::DefinitionFloat(def) => {
                if *def.is_output() {
                    float_registers_lenghts.insert(def.name().clone(), *def.length());
                }
            }
            Operation::DefinitionComplex(def) => {
                if *def.is_output() {
                    // Size of register = 4^(qubits_used)
                    complex_registers_lenghts.insert(def.name().clone(), *def.length());
                }
            }
            Operation::PragmaGetDensityMatrix(get_op) => {
                if let Some(length) = complex_registers_lenghts.get(get_op.readout()) {
                    let number_qubits = (*length).ilog(4) as usize;
                    used_qubits.extend(0..number_qubits);
                } else {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: undefined_register_msg(
                            "Complex",
                            get_op.readout(),
                            "PragmaGetDensityMatrix",
                            "DefinitionComplex",
                        ),
                    });
                }
            }
            Operation::PragmaGetStateVector(get_op) => {
                if let Some(length) = complex_registers_lenghts.get(get_op.readout()) {
                    let number_qubits = (*length).ilog2() as usize;
                    used_qubits.extend(0..number_qubits);
                } else {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: undefined_register_msg(
                            "Complex",
                            get_op.readout(),
                            "PragmaGetStateVector",
                            "DefinitionComplex",
                        ),
                    });
                }
            }
            Operation::PragmaRepeatedMeasurement(rep_measure) => {
                if let Some(length) = bit_registers_lenghts.get(rep_measure.readout()) {
                    if let Some(mapping) = rep_measure.qubit_mapping() {
                        for x in mapping.values() {
                            if x >= length {
                                return Err(RoqoqoBackendError::GenericError {
                                    msg: index_out_of_range_msg(x, length),
                                });
                            }
                        }
                        used_qubits.extend(mapping.keys());
                    } else {
                        used_qubits.extend(0..*length);
                    }
                } else {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: undefined_register_msg(
                            "Bit",
                            rep_measure.readout(),
                            "PragmaRepeatedMeasurement",
                            "DefinitionBit",
                        ),
                    });
                }
            }
            Operation::PragmaGetOccupationProbability(get_op) => {
                if let Some(length) = float_registers_lenghts.get(get_op.readout()) {
                    used_qubits.extend(0..*length);
                } else {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: undefined_register_msg(
                            "Float",
                            get_op.readout(),
                            "PragmaGetOccupationProbability",
                            "DefinitionFloat",
                        ),
                    });
                }
            }
            Operation::PragmaGetPauliProduct(get_op) => {
                used_qubits.extend(get_op.qubit_paulis().keys());
                if !float_registers_lenghts.contains_key(get_op.readout()) {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: undefined_register_msg(
                            "Float",
                            get_op.readout(),
                            "PragmaGetPauliProduct",
                            "DefinitionFloat",
                        ),
                    });
                }
            }
            Operation::MeasureQubit(measure_op) => {
                if let Some(length) = bit_registers_lenghts.get(measure_op.readout()) {
                    if measure_op.readout_index() >= length {
                        return Err(RoqoqoBackendError::GenericError {
                            msg: index_out_of_range_msg(measure_op.readout_index(), length),
                        });
                    }
                } else {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: undefined_register_msg(
                            "Bit",
                            measure_op.readout(),
                            "MeasureQubit",
                            "DefinitionBit",
                        ),
                    });
                }
            }
            _ => (),
        }
    }
    let largest_used_qubit = match used_qubits.iter().max() {
        Some(l) => *l + 1,
        None => 1,
    };

    Ok((largest_used_qubit, bit_registers_lenghts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use roqoqo::Circuit;

    #[test]
    fn test() {
        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 3, true);
        c += DefinitionBit::new("ro3".to_string(), 4, true);
        c += MeasureQubit::new(0, "ro3".to_string(), 0);
        c += MeasureQubit::new(3, "ro3".to_string(), 3);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);
        c += CNOT::new(1, 0);
        c += CNOT::new(0, 1);
        c += CNOT::new(1, 0);
        c += CNOT::new(0, 1);
        c += PauliX::new(4);
        c += PauliX::new(5);

        let (n, reg) = get_number_used_qubits_and_registers_lengths(&c.iter().collect()).unwrap();
        assert_eq!(6, n);

        let cmp_register = HashMap::from([("ro".to_string(), 3), ("ro3".to_string(), 4)]);
        assert_eq!(cmp_register, reg);
    }

    #[test]
    fn test_err_no_definition_complex() {
        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 3, true);
        c += DefinitionBit::new("ro1".to_string(), 4, true);
        c += MeasureQubit::new(0, "ro3".to_string(), 0);
        c += MeasureQubit::new(3, "ro3".to_string(), 3);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);
        c += CNOT::new(1, 0);
        c += CNOT::new(0, 1);
        c += CNOT::new(1, 0);
        c += CNOT::new(0, 1);
        c += PauliX::new(4);
        c += PauliX::new(5);
        c += PragmaGetStateVector::new("ro".to_string(), None);

        let res = get_number_used_qubits_and_registers_lengths(&c.iter().collect());
        assert!(res.is_err());

        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 3, true);
        c += DefinitionBit::new("ro1".to_string(), 4, true);

        c += PragmaGetDensityMatrix::new("ro".to_string(), None);

        let res = get_number_used_qubits_and_registers_lengths(&c.iter().collect());
        assert!(res.is_err());
    }

    #[test]
    fn test_err_no_definition_float() {
        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 3, true);
        c += DefinitionBit::new("ro1".to_string(), 4, true);
        c += PragmaGetOccupationProbability::new("ro".to_string(), None);

        let res = get_number_used_qubits_and_registers_lengths(&c.iter().collect());
        assert!(res.is_err());
    }

    #[test]
    fn test_err_no_definition_bit() {
        let mut c = Circuit::new();
        c += DefinitionComplex::new("ro".to_string(), 3, true);
        c += DefinitionBit::new("ro1".to_string(), 4, true);
        c += PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);

        let res = get_number_used_qubits_and_registers_lengths(&c.iter().collect());
        assert!(res.is_err());

        let mut c = Circuit::new();
        c += DefinitionComplex::new("ro".to_string(), 3, true);
        c += DefinitionBit::new("ro1".to_string(), 4, true);
        c += MeasureQubit::new(0, "ro".to_string(), 20);

        let res = get_number_used_qubits_and_registers_lengths(&c.iter().collect());
        assert!(res.is_err());

        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 3, true);
        c += MeasureQubit::new(0, "ro".to_string(), 20);

        let res = get_number_used_qubits_and_registers_lengths(&c.iter().collect());
        assert!(res.is_err());
    }

    #[test]
    fn test_get_used_qubits() {
        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 10, true);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);

        let (n, _) = get_number_used_qubits_and_registers_lengths(&c.iter().collect()).unwrap();
        assert_eq!(2, n);

        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 10, true);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);
        c += PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);

        let (n, _) = get_number_used_qubits_and_registers_lengths(&c.iter().collect()).unwrap();
        assert_eq!(10, n);

        let mut c = Circuit::new();
        c += DefinitionComplex::new("ro".to_string(), 16, true);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);
        c += PragmaGetDensityMatrix::new("ro".to_string(), None);

        let (n, _) = get_number_used_qubits_and_registers_lengths(&c.iter().collect()).unwrap();
        assert_eq!(2, n);

        let mut c = Circuit::new();
        c += DefinitionComplex::new("ro".to_string(), 16, true);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);
        c += PragmaGetStateVector::new("ro".to_string(), None);

        let (n, _) = get_number_used_qubits_and_registers_lengths(&c.iter().collect()).unwrap();
        assert_eq!(4, n);

        let mut c = Circuit::new();
        c += RotateX::new(0, 0.0.into());

        let (n, _) = get_number_used_qubits_and_registers_lengths(&c.iter().collect()).unwrap();
        assert_eq!(1, n);

        let mut c = Circuit::new();
        c += RotateX::new(0, 0.0.into());
        c += RotateX::new(12, 0.0.into());

        let (n, _) = get_number_used_qubits_and_registers_lengths(&c.iter().collect()).unwrap();
        assert_eq!(13, n);
    }

    #[test]
    fn test_get_register() {
        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 2, true);
        c += DefinitionBit::new("ri".to_string(), 2, false);
        c += DefinitionComplex::new("rc".to_string(), 4, true);

        c += PragmaGetDensityMatrix::new("rc".to_string(), None);

        let (_, reg) = get_number_used_qubits_and_registers_lengths(&c.iter().collect()).unwrap();

        let cmp_register = HashMap::from([("ro".to_string(), 2)]);
        assert_eq!(cmp_register, reg);

        let mut c = Circuit::new();
        c += DefinitionFloat::new("ro".to_string(), 2, true);
        c += DefinitionFloat::new("ri".to_string(), 2, false);

        let (_, reg) = get_number_used_qubits_and_registers_lengths(&c.iter().collect()).unwrap();

        let cmp_register = HashMap::new();
        assert_eq!(cmp_register, reg);

        let mut c = Circuit::new();
        c += DefinitionComplex::new("ro".to_string(), 64, true);
        c += DefinitionComplex::new("ri".to_string(), 2, false);

        let (used, reg) =
            get_number_used_qubits_and_registers_lengths(&c.iter().collect()).unwrap();

        let cmp_register = HashMap::new();
        assert_eq!(cmp_register, reg);
        assert_eq!(used, 1);

        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 2, true);
        c += DefinitionComplex::new("ri".to_string(), 10, true);

        let (used, reg) =
            get_number_used_qubits_and_registers_lengths(&c.iter().collect()).unwrap();

        let cmp_register = HashMap::from([("ro".to_string(), 2)]);
        assert_eq!(cmp_register, reg);
        assert_eq!(used, 1);
    }
}
