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

use rand::{prelude::*, rng};
use roqoqo::{noise_models::ImperfectReadoutModel, registers::*};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Applies a noise model to a bit register.
///
/// Arguments:
///  
/// * `bit_register`- A HashMap containing the names of the registers and their corresponding bit register.
/// * `noise_model`- An instance of `ImperfectReadoutModel` representing the noise model to be applied.
/// * `seed`- An optional seed for the random number generator. If provided, the same seed will be used for all random operations.
pub fn apply_noisy_readouts(
    bit_register: HashMap<String, BitOutputRegister>,
    noise_model: &ImperfectReadoutModel,
    seed: Option<Vec<u64>>,
) -> HashMap<String, BitOutputRegister> {
    let mut rng = match seed {
        Some(seeds) => {
            let mut hasher = Sha256::new();
            for seed in seeds {
                hasher.update(seed.to_le_bytes());
            }
            let mut seed_256 = [0u8; 32];
            seed_256.copy_from_slice(&hasher.finalize());
            StdRng::from_seed(seed_256)
        }
        None => StdRng::from_rng(&mut rng()),
    };
    let mut noisy_bit_register: HashMap<String, BitOutputRegister> = HashMap::new();
    for (name, bit_output) in bit_register {
        let noisy_bit = bit_output
            .iter()
            .map(|bit_results| {
                bit_results
                    .iter()
                    .enumerate()
                    .map(|(qubit, result)| {
                        if *result {
                            !rng.random_bool(noise_model.prob_detect_1_as_0(&qubit))
                        } else {
                            rng.random_bool(noise_model.prob_detect_0_as_1(&qubit))
                        }
                    })
                    .collect()
            })
            .collect();
        noisy_bit_register.insert(name, noisy_bit);
    }
    noisy_bit_register
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_apply_noisy_readouts() {
        use std::collections::HashMap;
        let bit_register = vec![
            vec![false, true, false, true],
            vec![true, false, true, false],
        ];
        let mut noise_model = roqoqo::noise_models::ImperfectReadoutModel::new();
        noise_model = noise_model.set_error_probabilites(0, 0.0, 1.0).unwrap();
        noise_model = noise_model.set_error_probabilites(1, 1.0, 0.0).unwrap();
        noise_model = noise_model.set_error_probabilites(2, 0.0, 0.0).unwrap();
        noise_model = noise_model.set_error_probabilites(3, 1.0, 1.0).unwrap();

        let mut registers = HashMap::new();
        registers.insert("register_0".to_string(), bit_register);

        let noisy_registers = super::apply_noisy_readouts(registers, &noise_model, None);

        let expected_result = vec![
            vec![false, true, false, false],
            vec![false, true, true, true],
        ];
        assert_eq!(noisy_registers.get("register_0").unwrap(), &expected_result);
    }
}
