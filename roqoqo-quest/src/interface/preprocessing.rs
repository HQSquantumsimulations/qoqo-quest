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
pub fn get_number_used_qubits_and_registers(
    circuit: &Vec<&Operation>,
) -> Result<(usize, HashMap<String, usize>), RoqoqoBackendError> {
    let mut used_qubits: HashSet<usize> = HashSet::new();
    let mut bit_registers: HashMap<String, usize> = HashMap::new();
    let mut float_registers: HashMap<String, usize> = HashMap::new();
    let mut complex_registers: HashMap<String, usize> = HashMap::new();

    for op in circuit {
        if let InvolvedQubits::Set(n) = op.involved_qubits() {
            used_qubits.extend(&n)
        }
        match op {
            Operation::DefinitionBit(def) => {
                if *def.is_output() {
                    bit_registers.insert(def.name().clone(), *def.length());
                }
            }
            Operation::DefinitionFloat(def) => {
                if *def.is_output() {
                    float_registers.insert(def.name().clone(), *def.length());
                }
            }
            Operation::DefinitionComplex(def) => {
                if *def.is_output() {
                    // Size of register = 4^(qubits_used)
                    complex_registers.insert(def.name().clone(), *def.length());
                }
            }
            Operation::PragmaGetDensityMatrix(get_op) => {
                if let Some(length) = complex_registers.get(get_op.readout()) {
                    let number_qubits = (*length).ilog(4) as usize;
                    used_qubits.extend(0..number_qubits);
                } else {
                    return Err(RoqoqoBackendError::GenericError{msg: format!("No Complex readout register {} defined for PragmaGetDensityMatrix operation. Did you forget to add a DefinitionComplex operation?", get_op.readout())});
                }
            }
            Operation::PragmaGetStateVector(get_op) => {
                if let Some(length) = complex_registers.get(get_op.readout()) {
                    let number_qubits = (*length).ilog2() as usize;
                    used_qubits.extend(0..number_qubits);
                } else {
                    return Err(RoqoqoBackendError::GenericError{msg: format!("No Complex readout register {} defined for PragmaGetStateVector operation. Did you forget to add a DefinitionComplex operation?", get_op.readout())});
                }
            }
            Operation::PragmaRepeatedMeasurement(rep_measure) => {
                if let Some(length) = bit_registers.get(rep_measure.readout()) {
                    if let Some(mapping) = rep_measure.qubit_mapping() {
                        for x in mapping.values() {
                            if x >= length {
                                return Err(RoqoqoBackendError::GenericError{msg: format!("Trying to write a qubit measurement in index {} or a bit register of length {}. Did you define a large enough register with the DefinitionBit operation?", x, length)});
                            }
                        }
                        used_qubits.extend(mapping.keys());
                    } else {
                        used_qubits.extend(0..*length);
                    }
                } else {
                    return Err(RoqoqoBackendError::GenericError{msg: format!("No Bit readout register {} defined for PragmaRepeatedMeasurement operation. Did you forget to add a DefinitionBit operation?", rep_measure.readout())});
                }
            }
            Operation::PragmaGetOccupationProbability(get_op) => {
                if let Some(length) = float_registers.get(get_op.readout()) {
                    used_qubits.extend(0..*length);
                } else {
                    return Err(RoqoqoBackendError::GenericError{msg: format!("No Float readout register {} defined for PragmaGetOccupationProbability operation. Did you forget to add a DefinitionFloat operation?", get_op.readout())});
                }
            }
            Operation::PragmaGetPauliProduct(get_op) => {
                used_qubits.extend(get_op.qubit_paulis().keys());
                if !float_registers.contains_key(get_op.readout()) {
                    return Err(RoqoqoBackendError::GenericError{msg: format!("No Float readout register {} defined for PragmaGetPauliProduct operation. Did you forget to add a DefinitionFloat operation?",get_op.readout() )});
                }
            }
            Operation::MeasureQubit(measure_op) => {
                if let Some(length) = bit_registers.get(measure_op.readout()) {
                    if measure_op.readout_index() >= length {
                        return Err(RoqoqoBackendError::GenericError{msg: format!("Trying to write a qubit measurement in index {} or a bit register of length {}. Did you define a large enough register with the DefinitionBit operation?", measure_op.readout_index(), length)});
                    }
                } else {
                    return Err(RoqoqoBackendError::GenericError{msg: format!("No Bit readout register {} defined for MeasureQubit operation. Did you forget to add a DefinitionBit operation?", measure_op.readout())});
                }
            }
            _ => (),
        }
    }
    let largest_used_qubit = match used_qubits.iter().max() {
        Some(l) => *l + 1,
        None => 1,
    };

    Ok((largest_used_qubit, bit_registers))
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

        let (n, reg) = get_number_used_qubits_and_registers(&c.iter().collect()).unwrap();
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

        let res = get_number_used_qubits_and_registers(&c.iter().collect());
        assert!(res.is_err());

        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 3, true);
        c += DefinitionBit::new("ro1".to_string(), 4, true);

        c += PragmaGetDensityMatrix::new("ro".to_string(), None);

        let res = get_number_used_qubits_and_registers(&c.iter().collect());
        assert!(res.is_err());
    }

    #[test]
    fn test_err_no_definition_float() {
        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 3, true);
        c += DefinitionBit::new("ro1".to_string(), 4, true);
        c += PragmaGetOccupationProbability::new("ro".to_string(), None);

        let res = get_number_used_qubits_and_registers(&c.iter().collect());
        assert!(res.is_err());
    }

    #[test]
    fn test_err_no_definition_bit() {
        let mut c = Circuit::new();
        c += DefinitionComplex::new("ro".to_string(), 3, true);
        c += DefinitionBit::new("ro1".to_string(), 4, true);
        c += PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);

        let res = get_number_used_qubits_and_registers(&c.iter().collect());
        assert!(res.is_err());

        let mut c = Circuit::new();
        c += DefinitionComplex::new("ro".to_string(), 3, true);
        c += DefinitionBit::new("ro1".to_string(), 4, true);
        c += MeasureQubit::new(0, "ro".to_string(), 20);

        let res = get_number_used_qubits_and_registers(&c.iter().collect());
        assert!(res.is_err());

        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 3, true);
        c += MeasureQubit::new(0, "ro".to_string(), 20);

        let res = get_number_used_qubits_and_registers(&c.iter().collect());
        assert!(res.is_err());
    }

    #[test]
    fn test_get_used_qubits() {
        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 10, true);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);

        let (n, _) = get_number_used_qubits_and_registers(&c.iter().collect()).unwrap();
        assert_eq!(2, n);

        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 10, true);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);
        c += PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);

        let (n, _) = get_number_used_qubits_and_registers(&c.iter().collect()).unwrap();
        assert_eq!(10, n);

        let mut c = Circuit::new();
        c += DefinitionComplex::new("ro".to_string(), 16, true);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);
        c += PragmaGetDensityMatrix::new("ro".to_string(), None);

        let (n, _) = get_number_used_qubits_and_registers(&c.iter().collect()).unwrap();
        assert_eq!(2, n);

        let mut c = Circuit::new();
        c += DefinitionComplex::new("ro".to_string(), 16, true);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);
        c += PragmaGetStateVector::new("ro".to_string(), None);

        let (n, _) = get_number_used_qubits_and_registers(&c.iter().collect()).unwrap();
        assert_eq!(4, n);

        let mut c = Circuit::new();
        c += RotateX::new(0, 0.0.into());

        let (n, _) = get_number_used_qubits_and_registers(&c.iter().collect()).unwrap();
        assert_eq!(1, n);

        let mut c = Circuit::new();
        c += RotateX::new(0, 0.0.into());
        c += RotateX::new(12, 0.0.into());

        let (n, _) = get_number_used_qubits_and_registers(&c.iter().collect()).unwrap();
        assert_eq!(13, n);
    }

    #[test]
    fn test_get_register() {
        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 2, true);
        c += DefinitionBit::new("ri".to_string(), 2, false);
        c += DefinitionComplex::new("rc".to_string(), 4, true);

        c += PragmaGetDensityMatrix::new("rc".to_string(), None);

        let (_, reg) = get_number_used_qubits_and_registers(&c.iter().collect()).unwrap();

        let cmp_register = HashMap::from([("ro".to_string(), 2)]);
        assert_eq!(cmp_register, reg);

        let mut c = Circuit::new();
        c += DefinitionFloat::new("ro".to_string(), 2, true);
        c += DefinitionFloat::new("ri".to_string(), 2, false);

        let (_, reg) = get_number_used_qubits_and_registers(&c.iter().collect()).unwrap();

        let cmp_register = HashMap::new();
        assert_eq!(cmp_register, reg);

        let mut c = Circuit::new();
        c += DefinitionComplex::new("ro".to_string(), 64, true);
        c += DefinitionComplex::new("ri".to_string(), 2, false);

        let (used, reg) = get_number_used_qubits_and_registers(&c.iter().collect()).unwrap();

        let cmp_register = HashMap::new();
        assert_eq!(cmp_register, reg);
        assert_eq!(used, 1);

        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 2, true);
        c += DefinitionComplex::new("ri".to_string(), 10, true);

        let (used, reg) = get_number_used_qubits_and_registers(&c.iter().collect()).unwrap();

        let cmp_register = HashMap::from([("ro".to_string(), 2)]);
        assert_eq!(cmp_register, reg);
        assert_eq!(used, 1);
    }
}
