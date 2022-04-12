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

use crate::interface::call_operation_with_device;
use qoqo_calculator::CalculatorFloat;
use roqoqo::backends::EvaluatingBackend;
// use roqoqo::measurements::Measure;
use crate::Qureg;
use roqoqo::backends::RegisterResult;
use roqoqo::operations::*;
use roqoqo::registers::{
    BitOutputRegister, BitRegister, ComplexOutputRegister, ComplexRegister, FloatOutputRegister,
    FloatRegister,
};
use roqoqo::RoqoqoBackendError;
use std::collections::HashMap;
/// QuEST backend
///
/// provides functions to run circuits and measurements on with the QuEST quantum simulator.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Backend {
    /// Number of qubits supported by the backend
    pub number_qubits: usize,
    /// Number of repetitions
    pub repetitions: usize,
}

impl Backend {
    /// Creates a new QuEST backend.
    ///
    /// # Arguments
    ///
    /// `number_qubits` - The number of qubits supported by the backend
    pub fn new(number_qubits: usize) -> Self {
        Self {
            number_qubits,
            repetitions: 1,
        }
    }

    /// Sets the number of repetitions used for stochastic circuit simulations
    ///
    /// The number of repetitions of the actual simulation is set to one by default.
    /// The repetitions are not to be confused with the number of simulated measurements per simulation run
    /// set with PragmaRepeatedMeasurement or PragmaSetNumberMeasurements.
    /// Should only be different from one if a stochastic unravelling of a noisy simulation is used
    /// with PragmaRandomNoise or PragmaOverrotation
    ///
    /// # Arguments
    ///
    /// `repetitions` - The number of repetitions that is set
    pub fn set_repetitions(mut self, repetitions: usize) -> Self {
        self.repetitions = repetitions;
        self
    }
}

impl EvaluatingBackend for Backend {
    fn run_circuit_iterator<'a>(
        &self,
        circuit: impl Iterator<Item = &'a Operation>,
    ) -> RegisterResult {
        self.run_circuit_iterator_with_device(circuit, &mut None)
    }
}

impl Backend {
    /// Runs each available operation obtained from an iterator over operations on the backend.
    ///
    /// An iterator over operations is passed to the backend and executed.
    /// During execution values are written to and read from classical registers
    /// ([crate::registers::BitRegister], [crate::registers::FloatRegister] and [crate::registers::ComplexRegister]).
    /// To produce sufficient statistics for evaluating expectationg values,
    /// circuits have to be run multiple times.
    /// The results of each repetition are concatenated in OutputRegisters
    /// ([crate::registers::BitOutputRegister], [crate::registers::FloatOutputRegister] and [crate::registers::ComplexOutputRegister]).  
    ///
    /// /// When the optional device parameter is not None availability checks will be performed.
    /// The availability of the operation on a specific device is checked first.
    /// The function returns an error if the operation is not available on the device
    /// even if it can be simulated with the QuEST simulator.
    ///
    /// # Arguments
    ///
    /// * `circuit` - The iterator over operations that is run on the backend (corresponds to a circuit).
    /// * `device` - The optional [roqoqo::devices::Device] that determines the availability of operations
    ///
    /// # Returns
    ///
    /// `RegisterResult` - The output registers written by the evaluated circuits.
    pub fn run_circuit_iterator_with_device<'a>(
        &self,
        circuit: impl Iterator<Item = &'a Operation>,
        device: &mut Option<Box<dyn roqoqo::devices::Device>>,
    ) -> RegisterResult {
        let circuit_vec: Vec<&'a Operation> = circuit.into_iter().collect();

        // Automatically switch to density matrix mode if operations are present in the
        // circuit that require density matrix mode
        let is_density_matrix = circuit_vec.iter().any(|x| {
            matches!(
                x,
                Operation::PragmaDamping(_)
                    | Operation::PragmaDephasing(_)
                    | Operation::PragmaDepolarising(_)
                    | Operation::PragmaGeneralNoise(_)
                    | Operation::PragmaSetDensityMatrix(_)
            )
        });

        // Calculatre total global phase of the circuit
        let mut global_phase: CalculatorFloat = CalculatorFloat::ZERO;
        for global_phase_tmp in circuit_vec.iter().filter_map(|x| match x {
            Operation::PragmaGlobalPhase(x) => Some(x.phase()),
            _ => None,
        }) {
            global_phase += global_phase_tmp.clone();
        }

        // Determine repetition, how many times the numerical simulation is repeated
        // (not to be confused with the number of measurements drawn from one sample)
        // This is only necessary for stochastic unravelling where a stochastic trajectory
        // of a single state is simulated many times to reconstruct the density matrix
        // (when PragmaRandomNoise is present in circuit)
        // or when allowing for stochastic overrotations where coherent gates are applied
        // with a stocastic offset
        // (when PragmaOverrotation is present in circuit)
        let repetitions = match circuit_vec.iter().find(|x| {
            matches!(
                x,
                Operation::PragmaRandomNoise(_) | Operation::PragmaOverrotation(_)
            )
        }) {
            Some(_) => self.repetitions,
            None => 1,
        };

        let mut qureg = Qureg::new(self.number_qubits as u32, is_density_matrix);

        // Set up output registers
        let mut bit_registers_output: HashMap<String, BitOutputRegister> = HashMap::new();
        let mut float_registers_output: HashMap<String, FloatOutputRegister> = HashMap::new();
        let mut complex_registers_output: HashMap<String, ComplexOutputRegister> = HashMap::new();

        let mut number_measurements: Option<usize> = None;
        let mut repeated_measurement_readout: String = "".to_string();
        let mut replace_measurements = false;
        for op in circuit_vec.iter() {
            match op {
                Operation::PragmaRepeatedMeasurement(o) => {
                    match number_measurements{
                        Some(_) => return Err(RoqoqoBackendError::GenericError{msg: format!("Only one repeated measurement allowed, trying to run repeated measurement for {} but already used for  {:?}", o.readout(), repeated_measurement_readout )}),
                        None => { number_measurements = Some(*o.number_measurements()); repeated_measurement_readout = o.readout().clone()}
                    }
                }
                Operation::PragmaSetNumberOfMeasurements(o) => {
                    match number_measurements{
                        Some(_) => return Err(RoqoqoBackendError::GenericError{msg: format!("Only one repeated measurement allowed, trying to run repeated measurement for {} but already used for  {:?}", o.readout(), repeated_measurement_readout )}),
                        None => { number_measurements = Some(*o.number_measurements()); repeated_measurement_readout = o.readout().clone(); replace_measurements=true;}
                    }
                }
                Operation::DefinitionBit(def) => {
                    if *def.is_output() {
                        bit_registers_output.insert(def.name().clone(), Vec::new());
                    }
                }
                Operation::DefinitionFloat(def) => {
                    if *def.is_output() {
                        float_registers_output.insert(def.name().clone(), Vec::new());
                    }
                }
                Operation::DefinitionComplex(def) => {
                    if *def.is_output() {
                        complex_registers_output.insert(def.name().clone(), Vec::new());
                    }
                }
                _ => (),
            }
        }

        // Create a repeated measurement operation
        let mut repeated_measurement_pragma: Option<PragmaRepeatedMeasurement> =
            if replace_measurements {
                let name = repeated_measurement_readout.clone();
                let mut reordering_map: HashMap<usize, usize> = HashMap::new();
                // Go through operations to build up hash map when readout_index is not equal to measured qubit
                for op in circuit_vec.iter() {
                    if let Operation::MeasureQubit(measure) = op {
                        reordering_map.insert(*measure.qubit(), *measure.readout_index());
                    }
                }
                Some(PragmaRepeatedMeasurement::new(
                    name,
                    number_measurements
                        .expect("Cannot find number of repeated measurement output internal bug"),
                    Some(reordering_map),
                ))
            } else {
                None
            };
        for _ in 0..repetitions {
            let mut bit_registers_internal: HashMap<String, BitRegister> = HashMap::new();
            let mut float_registers_internal: HashMap<String, FloatRegister> = HashMap::new();
            let mut complex_registers_internal: HashMap<String, ComplexRegister> = HashMap::new();
            // If the SetNumberMeasurements pragma is used go through operations and replace first
            // instance of MeasureQubit with matching
            if replace_measurements {
                for op in circuit_vec.iter() {
                    match op {
                        // Find measurement operation
                        Operation::MeasureQubit(measure_op) => {
                            // If readout matches readout of repeated measurements
                            if &repeated_measurement_readout == measure_op.readout() {
                                // Check if repeated_measurement_pragma is Some(x).
                                //Will be reset to None after replacing first measurement
                                // with matching readout
                                if let Some(rm) = repeated_measurement_pragma.clone() {
                                    let repeated_measure = rm.clone();
                                    // replace normal measurement operation call with repeated Pragma
                                    call_operation_with_device(
                                        &Operation::from(repeated_measure),
                                        &mut qureg,
                                        &mut bit_registers_internal,
                                        &mut float_registers_internal,
                                        &mut complex_registers_internal,
                                        &mut bit_registers_output,
                                        device,
                                    )?;
                                    repeated_measurement_pragma = None;
                                }
                            } else {
                                call_operation_with_device(
                                    op,
                                    &mut qureg,
                                    &mut bit_registers_internal,
                                    &mut float_registers_internal,
                                    &mut complex_registers_internal,
                                    &mut bit_registers_output,
                                    device,
                                )?;
                            }
                        }
                        // Normal Operation call for non-measurements
                        _ => {
                            call_operation_with_device(
                                op,
                                &mut qureg,
                                &mut bit_registers_internal,
                                &mut float_registers_internal,
                                &mut complex_registers_internal,
                                &mut bit_registers_output,
                                device,
                            )?;
                        }
                    }
                }
                // Standard path when not using PragmaSetRepeatedMeasurements
            } else {
                for op in circuit_vec.iter() {
                    call_operation_with_device(
                        op,
                        &mut qureg,
                        &mut bit_registers_internal,
                        &mut float_registers_internal,
                        &mut complex_registers_internal,
                        &mut bit_registers_output,
                        device,
                    )?;
                }
            }

            // Append bit result of one circuit execution to output register
            for (name, register) in bit_registers_output.iter_mut() {
                if let Some(tmp_reg) = bit_registers_internal.get(name) {
                    register.push(tmp_reg.to_owned())
                }
            }
            // Append float result of one circuit execution to output register
            for (name, register) in float_registers_output.iter_mut() {
                if let Some(tmp_reg) = float_registers_internal.get(name) {
                    register.push(tmp_reg.to_owned())
                }
            }
            // Append complex result of one circuit execution to output register
            for (name, register) in complex_registers_output.iter_mut() {
                if let Some(tmp_reg) = complex_registers_internal.get(name) {
                    register.push(tmp_reg.to_owned())
                }
            }
        }
        Ok((
            bit_registers_output,
            float_registers_output,
            complex_registers_output,
        ))
    }
}
