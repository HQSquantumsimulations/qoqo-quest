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

use roqoqo::backends::EvaluatingBackend;
use roqoqo::operations;
use roqoqo::Circuit;
use roqoqo_quest::Backend;

#[test]
fn test_circuit_with_repeated_measurement() {
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 4, true);
    circuit += operations::PauliX::new(1);
    circuit += operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);
    let backend = Backend::new(4);
    let (bit_result, float_result, complex_result) =
        backend.run_circuit_iterator(circuit.iter()).unwrap();
    assert!(float_result.is_empty());
    assert!(complex_result.is_empty());
    assert!(bit_result.contains_key("ro"));
    let nested_vec = bit_result.get("ro").unwrap();
    assert!(nested_vec.len() == 10);
    for repetition in nested_vec {
        assert!(repetition.len() == 4);
        assert_eq!(repetition[0], false);
        assert_eq!(repetition[1], true);
        assert_eq!(repetition[2], false);
    }
}

#[test]
fn test_circuit_with_set_measurement_number() {
    let mut circuit = Circuit::new();
    circuit += operations::DefinitionBit::new("ro".to_string(), 4, true);
    circuit += operations::PauliX::new(1);
    circuit += operations::MeasureQubit::new(0, "ro".to_string(), 0);
    circuit += operations::MeasureQubit::new(1, "ro".to_string(), 1);
    circuit += operations::MeasureQubit::new(2, "ro".to_string(), 2);
    circuit += operations::PragmaSetNumberOfMeasurements::new(10, "ro".to_string());
    let backend = Backend::new(4);
    let (bit_result, float_result, complex_result) =
        backend.run_circuit_iterator(circuit.iter()).unwrap();
    assert!(float_result.is_empty());
    assert!(complex_result.is_empty());
    assert!(bit_result.contains_key("ro"));
    let nested_vec = bit_result.get("ro").unwrap();
    assert!(nested_vec.len() == 10);
    for repetition in nested_vec {
        assert!(repetition.len() == 4);
        assert_eq!(repetition[0], false);
        assert_eq!(repetition[1], true);
        assert_eq!(repetition[2], false);
    }
}
