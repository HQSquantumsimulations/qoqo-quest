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

use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;
use roqoqo::operations::*;

pub fn get_number_used_qubits_and_registers(circuit: &Vec<& Operation> ) -> (usize,HashMap<String, usize>){
    let mut used_qubits: HashSet<usize> = HashSet::new();
    let mut registers: HashMap<String, usize> = HashMap::new();
    let mut max_qubit: usize = 0;
    for op in circuit {
        match op.involved_qubits() {
            InvolvedQubits::Set(n) => {used_qubits.extend(&n)}
            _ => ()
        }
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
                    registers.insert(def.name().clone(), *def.length());
                    max_qubit = cmp::max(max_qubit, *def.length())
                }
            }
            _ => (),
        }
    }
    let largest_used_qubit = match used_qubits.iter().max() {
        Some(l) => *l + 1,
        None => 0
    };

    max_qubit = cmp::max(largest_used_qubit, max_qubit);

    (max_qubit, registers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use roqoqo::Circuit;

    #[test]
    fn test_1() {
        let mut c = Circuit::new();
        c += DefinitionBit::new("ro".to_string(), 2, true);
        c += RotateX::new(0, 0.0.into());
        c += CNOT::new(0, 1);
        c+= PragmaGetStateVector::new("ro".to_string(), None);

        let (n,_) = get_number_used_qubits_and_registers( &c.iter().into_iter().collect());
        assert_eq!(2, n)
        
    }
}