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
use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;

pub fn get_number_used_qubits_and_registers(
    circuit: &Vec<&Operation>,
) -> (usize, HashMap<String, usize>) {
    let mut used_qubits: HashSet<usize> = HashSet::new();
    let mut registers: HashMap<String, usize> = HashMap::new();
    let mut max_qubit: usize = 0;
    for op in circuit {
        if let InvolvedQubits::Set(n) = op.involved_qubits() { used_qubits.extend(&n) }
        match op {
            Operation::DefinitionBit(def) => {
                if *def.is_output() {
                    registers.insert(def.name().clone(), *def.length());
                    max_qubit = cmp::max(max_qubit, *def.length())
                }
            }
            Operation::DefinitionFloat(def) => {
                if *def.is_output() {
                    registers.insert(def.name().clone(), *def.length());
                    max_qubit = cmp::max(max_qubit, *def.length())
                }
            }
            Operation::DefinitionComplex(def) => {
                if *def.is_output() {
                    // Used bits is given by ceiling of Log2
                    let bits = u64::BITS - (def.length() - 1).leading_zeros();
                    registers.insert(def.name().clone(), bits as usize);
                    max_qubit = cmp::max(max_qubit, bits as usize)
                }
            }
            _ => (),
        }
    }
    let largest_used_qubit = match used_qubits.iter().max() {
        Some(l) => *l + 1,
        None => 0,
    };

    max_qubit = cmp::max(largest_used_qubit, max_qubit);

    (max_qubit, registers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use roqoqo::Circuit;

    #[test]
    fn test() {
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

        let (n, reg) = get_number_used_qubits_and_registers(&c.iter().into_iter().collect());
        assert_eq!(6, n);

        let cmp_register = HashMap::from([
            ("ro".to_string(), 3 as usize),
            ("ro1".to_string(), 4 as usize),
        ]);
        assert_eq!(cmp_register, reg);
    }

    #[test]
    fn test_get_used_qubits() {
        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 10, true);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);
        c += PragmaGetDensityMatrix::new("ro".to_string(), None);

        let (n, _) = get_number_used_qubits_and_registers(&c.iter().into_iter().collect());
        assert_eq!(10, n);

        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 10, true);
        c += DefinitionFloat::new("ro".to_string(), 5, true);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);
        c += PragmaGetDensityMatrix::new("ro".to_string(), None);

        let (n, _) = get_number_used_qubits_and_registers(&c.iter().into_iter().collect());
        assert_eq!(10, n);

        let mut c = Circuit::new();
        c += RotateX::new(0, 0.0.into());

        let (n, _) = get_number_used_qubits_and_registers(&c.iter().into_iter().collect());
        assert_eq!(1, n);

        let mut c = Circuit::new();
        c += RotateX::new(0, 0.0.into());
        c += RotateX::new(12, 0.0.into());

        let (n, _) = get_number_used_qubits_and_registers(&c.iter().into_iter().collect());
        assert_eq!(13, n);
    }

    #[test]
    fn test_get_register() {
        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 2, true);
        c += DefinitionBit::new("ri".to_string(), 2, false);
        c += PragmaGetDensityMatrix::new("ro".to_string(), None);

        let (_, reg) = get_number_used_qubits_and_registers(&c.iter().into_iter().collect());

        let cmp_register = HashMap::from([("ro".to_string(), 2 as usize)]);
        assert_eq!(cmp_register, reg);

        let mut c = Circuit::new();
        c += DefinitionFloat::new("ro".to_string(), 2, true);
        c += DefinitionFloat::new("ri".to_string(), 2, false);
        c += PragmaGetDensityMatrix::new("ro".to_string(), None);

        let (_, reg) = get_number_used_qubits_and_registers(&c.iter().into_iter().collect());

        let cmp_register = HashMap::from([("ro".to_string(), 2 as usize)]);
        assert_eq!(cmp_register, reg);

        let mut c = Circuit::new();
        c += DefinitionComplex::new("ro".to_string(), 8, true);
        c += DefinitionComplex::new("ri".to_string(), 2, false);
        c += PragmaGetDensityMatrix::new("ro".to_string(), None);

        let (used, reg) = get_number_used_qubits_and_registers(&c.iter().into_iter().collect());

        let cmp_register = HashMap::from([("ro".to_string(), 3 as usize)]);
        assert_eq!(cmp_register, reg);
        assert_eq!(used, 3);

        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 2, true);
        c += DefinitionComplex::new("ri".to_string(), 10, true);
        c += PragmaGetDensityMatrix::new("ro".to_string(), None);

        let (used, reg) = get_number_used_qubits_and_registers(&c.iter().into_iter().collect());

        let cmp_register = HashMap::from([
            ("ro".to_string(), 2 as usize),
            ("ri".to_string(), 4 as usize),
        ]);
        assert_eq!(cmp_register, reg);
        assert_eq!(used, 4);
    }
}
