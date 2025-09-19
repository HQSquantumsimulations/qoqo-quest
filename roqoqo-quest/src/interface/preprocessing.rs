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
#[cfg(feature = "unstable_operation_definition")]
use roqoqo::Circuit;
use roqoqo::RoqoqoBackendError;
use std::collections::HashMap;
use std::collections::HashSet;

#[inline]
fn format_error_msg(reg_type: &str, reg_name: &str, op: &str, missing_op: &str) -> String {
    format!(
        "No {reg_type} readout register {reg_name} defined for {op} operation. Did you forget to add a {missing_op} operation?"
    )
}

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
                    return Err(RoqoqoBackendError::GenericError {
                        msg: format_error_msg(
                            "Complex",
                            get_op.readout(),
                            "PragmaGetDensiryMatrix",
                            "DefinitionComplex",
                        ),
                    });
                }
            }
            Operation::PragmaGetStateVector(get_op) => {
                if let Some(length) = complex_registers.get(get_op.readout()) {
                    let number_qubits = (*length).ilog2() as usize;
                    used_qubits.extend(0..number_qubits);
                } else {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: format_error_msg(
                            "Complex",
                            get_op.readout(),
                            "PragmaGetStateVector",
                            "DefinitionComplex",
                        ),
                    });
                }
            }
            Operation::PragmaRepeatedMeasurement(rep_measure) => {
                if let Some(length) = bit_registers.get(rep_measure.readout()) {
                    if let Some(mapping) = rep_measure.qubit_mapping() {
                        for x in mapping.values() {
                            if x >= length {
                                return Err(RoqoqoBackendError::GenericError {
                                    msg: format!(
                                        "Trying to write a qubit measurement in index {x} or a bit \
                                         register of length {length}. Did you define a large enough \
                                         register with the DefinitionBit operation?"
                                    ),
                                });
                            }
                        }
                        used_qubits.extend(mapping.keys());
                    } else {
                        used_qubits.extend(0..*length);
                    }
                } else {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: format_error_msg(
                            "Bit",
                            rep_measure.readout(),
                            "PragmaRepeatedMeasurement",
                            "DefinitionBit",
                        ),
                    });
                }
            }
            Operation::PragmaGetOccupationProbability(get_op) => {
                if let Some(length) = float_registers.get(get_op.readout()) {
                    used_qubits.extend(0..*length);
                } else {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: format_error_msg(
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
                if !float_registers.contains_key(get_op.readout()) {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: format_error_msg(
                            "Float",
                            get_op.readout(),
                            "PragmaGetPauliProduct",
                            "DefinitionFloat",
                        ),
                    });
                }
            }
            Operation::MeasureQubit(measure_op) => {
                if let Some(length) = bit_registers.get(measure_op.readout()) {
                    if measure_op.readout_index() >= length {
                        return Err(RoqoqoBackendError::GenericError {
                            msg: format!(
                                "Trying to write a qubit measurement in index {} or a bit register \
                                 of length {}. Did you define a large enough register with the \
                                 DefinitionBit operation?",
                                measure_op.readout_index(),
                                length
                            ),
                        });
                    }
                } else {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: format_error_msg(
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

    Ok((largest_used_qubit, bit_registers))
}

#[cfg(feature = "unstable_operation_definition")]
/// Replaces custom gates in a circuit with their definitions.
///
/// # Arguments
/// * `circuit` - A reference to a vector of operations representing the circuit.
///
/// # Returns
/// * `Result<Circuit, RoqoqoBackendError>` - A result containing the new circuit with custom gates replaced, or an error if the replacement fails.
///
/// # Errors
/// * Returns a `RoqoqoBackendError` if a custom gate is called without being defined first, or if there are issues with parameter substitution or qubit remapping.
///
pub fn replace_custom_gates<'a>(
    circuit: &Vec<&'a Operation>,
) -> Result<Circuit, RoqoqoBackendError> {
    let mut custom_gates: HashMap<String, (Vec<usize>, Vec<String>, Circuit)> = HashMap::new();
    let mut new_circuit = Circuit::new();
    for &op in circuit {
        if let Operation::GateDefinition(custom_gate) = op {
            custom_gates.insert(
                custom_gate.name().clone(),
                (
                    custom_gate.qubits().clone(),
                    custom_gate.free_parameters().clone(),
                    custom_gate.circuit().clone(),
                ),
            );
        } else if let Operation::CallDefinedGate(call_gate) = op {
            use qoqo_calculator::Calculator;

            let (qubit_indices, param_names, gate_circuit) = custom_gates
                .get(call_gate.gate_name())
                .expect("Custom gate not defined");
            let mut calculator = Calculator::new();
            for (name, value) in param_names.iter().zip(call_gate.free_parameters()) {
                calculator.set_variable(
                    name,
                    *value
                        .float()
                        .expect("msg: Failed to convert parameter value to float"),
                );
            }
            let mut mapping = HashMap::new();
            for (from, to) in qubit_indices.iter().zip(call_gate.qubits()) {
                mapping.insert(*from, *to);
            }
            for target in mapping.values().copied().collect::<Vec<_>>() {
                if !mapping.contains_key(&target) {
                    mapping.insert(target, target);
                }
            }
            let gate_circuit = gate_circuit.substitute_parameters(&calculator)?;
            new_circuit += gate_circuit.remap_qubits(&mapping)?;
        } else {
            new_circuit += op.clone();
        }
    }
    Ok(new_circuit)
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

    #[cfg(feature = "unstable_operation_definition")]
    #[test]
    fn test_replace_custom_gates() {
        use qoqo_calculator::CalculatorFloat;
        use serde::de::Expected;
        let mut gate_circ = Circuit::new();
        gate_circ += PauliX::new(0);
        gate_circ += PauliY::new(1);
        gate_circ += RotateX::new(
            0,
            qoqo_calculator::CalculatorFloat::Str("param1".to_owned()),
        );
        let mut c = Circuit::new();
        c += GateDefinition::new(
            gate_circ,
            "custom_gate".to_owned(),
            vec![0, 1],
            vec!["param1".to_string()],
        );
        c += CallDefinedGate::new("custom_gate".to_owned(), vec![2, 1], vec![1.57.into()]);
        c += CNOT::new(0, 2);
        c += CallDefinedGate::new(
            "custom_gate".to_owned(),
            vec![0, 1],
            vec![CalculatorFloat::PI],
        );

        let replaced_circuit = replace_custom_gates(&c.iter().collect()).unwrap();

        let mut expected_circuit = Circuit::new();
        expected_circuit += PauliX::new(2);
        expected_circuit += PauliY::new(1);
        expected_circuit += RotateX::new(2, 1.57.into());
        expected_circuit += CNOT::new(0, 2);
        expected_circuit += PauliX::new(0);
        expected_circuit += PauliY::new(1);
        expected_circuit += RotateX::new(0, CalculatorFloat::PI);
        assert_eq!(expected_circuit, replaced_circuit);
    }
}
