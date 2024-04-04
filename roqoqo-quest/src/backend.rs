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

use crate::interface::{
    call_operation_with_device, execute_pragma_repeated_measurement,
    get_number_used_qubits_and_registers,
};
#[cfg(feature = "async")]
use async_trait::async_trait;
use qoqo_calculator::CalculatorFloat;
#[cfg(feature = "parallelization")]
use rayon::prelude::*;
#[cfg(feature = "async")]
use roqoqo::backends::AsyncEvaluatingBackend;
use roqoqo::backends::EvaluatingBackend;
#[cfg(feature = "parallelization")]
use roqoqo::measurements::Measure;
#[cfg(feature = "parallelization")]
use roqoqo::registers::Registers;
#[cfg(feature = "parallelization")]
use roqoqo::Circuit;
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

    #[cfg(feature = "parallelization")]
    fn run_measurement_registers<T>(&self, measurement: &T) -> RegisterResult
    where
        T: Measure,
    {
        let mut bit_registers: HashMap<String, BitOutputRegister> = HashMap::new();
        let mut float_registers: HashMap<String, FloatOutputRegister> = HashMap::new();
        let mut complex_registers: HashMap<String, ComplexOutputRegister> = HashMap::new();

        let circuits: Vec<&Circuit> = measurement.circuits().collect();
        let constant_circuit = match measurement.constant_circuit() {
            Some(c) => Some(c),
            None => None,
        };
        let tmp_regs_res: Result<Vec<Registers>, RoqoqoBackendError> = circuits
            .par_iter()
            .map(|circuit| match constant_circuit {
                Some(x) => self.run_circuit_iterator(x.iter().chain(circuit.iter())),
                None => self.run_circuit_iterator(circuit.iter()),
            })
            .collect();
        let tmp_regs = tmp_regs_res?;

        for (tmp_bit_reg, tmp_float_reg, tmp_complex_reg) in tmp_regs.into_iter() {
            for (key, mut val) in tmp_bit_reg.into_iter() {
                if let Some(x) = bit_registers.get_mut(&key) {
                    x.append(&mut val);
                } else {
                    let _ = bit_registers.insert(key, val);
                }
            }
            for (key, mut val) in tmp_float_reg.into_iter() {
                if let Some(x) = float_registers.get_mut(&key) {
                    x.append(&mut val);
                } else {
                    let _ = float_registers.insert(key, val);
                }
            }
            for (key, mut val) in tmp_complex_reg.into_iter() {
                if let Some(x) = complex_registers.get_mut(&key) {
                    x.append(&mut val);
                } else {
                    let _ = complex_registers.insert(key, val);
                }
            }
        }
        Ok((bit_registers, float_registers, complex_registers))
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
    /// When the optional device parameter is not None availability checks will be performed.
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
        let is_density_matrix = circuit_vec.iter().any(find_pragma_op);

        // Calculate total global phase of the circuit
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
        let mut repetitions = match circuit_vec.iter().find(|x| {
            matches!(
                x,
                Operation::PragmaRandomNoise(_) | Operation::PragmaOverrotation(_)
            )
        }) {
            Some(_) => self.repetitions,
            None => 1,
        };

        let (number_used_qubits, register_lengths) =
            get_number_used_qubits_and_registers(&circuit_vec, is_density_matrix);

        println!("{}", is_density_matrix);
        println!("In Quest: {}, {:?}", number_used_qubits, register_lengths);
        if number_used_qubits > self.number_qubits {
            return Err(RoqoqoBackendError::GenericError { msg: format!(" Insufficient qubits in backend. Available qubits:`{}`. Number of qubits used in circuit:`{}` ", self.number_qubits, number_used_qubits) });
        }

        let mut qureg = Qureg::new((number_used_qubits) as u32, is_density_matrix);

        // Set up output registers
        let mut bit_registers_output: HashMap<String, BitOutputRegister> = HashMap::new();
        let mut float_registers_output: HashMap<String, FloatOutputRegister> = HashMap::new();
        let mut complex_registers_output: HashMap<String, ComplexOutputRegister> = HashMap::new();

        for op in circuit_vec.iter() {
            match op {
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

        // Check if we are using repeated measurements
        let mut number_measurements: Option<usize> = None;
        let mut repeated_measurement_readout: String = "".to_string();
        let mut replace_measurements: Option<usize> = None;
        for op in circuit_vec.iter() {
            match op {
                Operation::PragmaRepeatedMeasurement(o) => {
                    match number_measurements{
                        Some(_) => return Err(RoqoqoBackendError::GenericError{msg: format!("Only one repeated measurement allowed, trying to run repeated measurement for {} but already used for  {:?}", o.readout(), repeated_measurement_readout )}),
                        None => { number_measurements = Some(*o.number_measurements()); repeated_measurement_readout = o.readout().clone(); replace_measurements=Some(0);}
                        }
                }
                Operation::PragmaSetNumberOfMeasurements(o) => {
                    match number_measurements{
                        Some(_) => return Err(RoqoqoBackendError::GenericError{msg: format!("Only one repeated measurement allowed, trying to run repeated measurement for {} but already used for  {:?}", o.readout(), repeated_measurement_readout )}),
                        None => {number_measurements = Some(*o.number_measurements()); repeated_measurement_readout = o.readout().clone(); replace_measurements=Some(0);}
                    }
                }
                _ => ()
            }
        }
        let found_fitting_measurement = circuit_vec.iter().any(|op| match op {
            Operation::MeasureQubit(inner_op) => {
                inner_op.readout().as_str() == repeated_measurement_readout.as_str()
            }
            Operation::PragmaRepeatedMeasurement(inner_op) => {
                inner_op.readout().as_str() == repeated_measurement_readout.as_str()
            }
            _ => false,
        });
        if number_measurements.is_some() && !found_fitting_measurement {
            return Err(RoqoqoBackendError::GenericError { msg: format!("No matching measurement found for PragmaSetNumberOfMeasurements for readout `{}`",repeated_measurement_readout) });
        }

        // Switch between repeated measurement mode and rerunning the whole circuit
        // Circuit needs to be rerun if
        //
        // 1. Any qubit in the repeated measurement is measured before the repeated measurement
        // 2. A qubit in the conditional measurement is acted upon after the conditional measurement
        let mut measured_qubits: Vec<usize> = Vec::new();
        let mut measured_qubits_in_repeated_measurement: Vec<usize> = Vec::new();
        let mut temporary_repetitions = self.repetitions;
        for op in circuit_vec.iter() {
            match op {
                Operation::MeasureQubit(o) => {
                    if let Some(nm) = number_measurements {
                        // If we have a repeated measurement (number_measurements not None)
                        // Check that

                        if o.readout() != &repeated_measurement_readout
                            || measured_qubits_in_repeated_measurement.contains(o.qubit())
                        {
                            replace_measurements = None;
                            temporary_repetitions = nm * repetitions;
                            number_measurements = None;
                            measured_qubits_in_repeated_measurement.push(*o.qubit());
                        } else if o.readout() == &repeated_measurement_readout {
                            measured_qubits_in_repeated_measurement.push(*o.qubit());
                            measured_qubits.push(*o.qubit());
                            replace_measurements = Some(*o.qubit());
                        } else {
                            measured_qubits.push(*o.qubit())
                        }
                    }
                }
                Operation::PragmaRepeatedMeasurement(o) => {
                    if let Some(nm) = number_measurements {
                        // Construct involved qubits
                        let involved_qubits: Vec<usize> = match register_lengths.get(o.readout()) {
                            Some(pragma_nm) => {
                                let n = *pragma_nm;
                                (0..(n - 1)).collect()
                            }

                            None => {
                                return Err(RoqoqoBackendError::GenericError{msg: "No register corresponding to PragmaRepeatedMeasurement readout found".to_string()});
                            }
                        };

                        if o.readout() != &repeated_measurement_readout {
                            return Err(RoqoqoBackendError::GenericError{msg: format!("Only one repeated measurement allowed, trying to run repeated measurement for {} but already used for  {:?}", o.readout(), repeated_measurement_readout )});
                        } else if measured_qubits.iter().any(|q| involved_qubits.contains(q)) {
                            replace_measurements = None;
                            temporary_repetitions = nm * repetitions;
                            number_measurements = None;
                            measured_qubits_in_repeated_measurement.extend(involved_qubits.clone());
                            measured_qubits.extend(involved_qubits);
                        } else {
                            measured_qubits.extend(involved_qubits);
                        }
                    }
                }
                Operation::PragmaStopParallelBlock(_) => {}
                // Check that no operation acts on repeated measured qubits after measurement
                // If it does revert to full iterations of circuit
                _ => match op.involved_qubits() {
                    InvolvedQubits::All => {
                        if let Some(nm) = number_measurements {
                            if !measured_qubits_in_repeated_measurement.is_empty() {
                                replace_measurements = None;
                                temporary_repetitions = nm * repetitions;
                                number_measurements = None;
                            }
                        }
                    }
                    InvolvedQubits::None => (),
                    InvolvedQubits::Set(qubit_set) => {
                        if let Some(nm) = number_measurements {
                            if qubit_set
                                .iter()
                                .any(|q| measured_qubits_in_repeated_measurement.contains(q))
                            {
                                replace_measurements = None;
                                temporary_repetitions = nm * repetitions;
                                number_measurements = None;
                            }
                        }
                    }
                },
            }
        }
        repetitions = temporary_repetitions;

        // Create a repeated measurement operation
        let repeated_measurement_pragma: Option<PragmaRepeatedMeasurement> =
            if replace_measurements.is_some() {
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
            qureg.reset();
            let mut bit_registers_internal: HashMap<String, BitRegister> = HashMap::new();
            let mut float_registers_internal: HashMap<String, FloatRegister> = HashMap::new();
            let mut complex_registers_internal: HashMap<String, ComplexRegister> = HashMap::new();
            run_inner_circuit_loop(
                &register_lengths,
                &circuit_vec,
                (replace_measurements, &repeated_measurement_pragma),
                &mut qureg,
                (
                    &mut bit_registers_internal,
                    &mut float_registers_internal,
                    &mut complex_registers_internal,
                ),
                &mut bit_registers_output,
                device,
            )?;

            // Append bit result of one circuit execution to output register
            for (name, register) in bit_registers_output.iter_mut() {
                if let Some(tmp_reg) = bit_registers_internal.get(name) {
                    if replace_measurements.is_none() || name != &repeated_measurement_readout {
                        register.push(tmp_reg.to_owned())
                    }
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

// groups replace_measurements and repeated_measurement_pragma
type ReplacedMeasurementInformation<'a> = (Option<usize>, &'a Option<PragmaRepeatedMeasurement>);

fn run_inner_circuit_loop(
    register_lengths: &HashMap<String, usize>,
    circuit_vec: &[&Operation],
    replaced_measurement_information: ReplacedMeasurementInformation,
    qureg: &mut Qureg,
    registers_internal: InternalRegisters,
    bit_registers_output: &mut HashMap<String, Vec<Vec<bool>>>,
    device: &mut Option<Box<dyn roqoqo::devices::Device>>,
) -> Result<(), RoqoqoBackendError> {
    let (replace_measurements, repeated_measurement_pragma) = replaced_measurement_information;
    let (bit_registers_internal, float_registers_internal, complex_registers_internal) =
        registers_internal;
    for op in circuit_vec.iter() {
        match op {
            Operation::PragmaRepeatedMeasurement(rm) => match replace_measurements {
                None => {
                    let number_qubits: usize = match register_lengths.get(rm.readout()) {
                        Some(pragma_nm) => {
                            let n = *pragma_nm;
                            n - 1
                        }
                        None => {
                            return Err(RoqoqoBackendError::GenericError{msg: "No register corresponding to PragmaRepeatedMeasurement readout found".to_string()});
                        }
                    };
                    for qb in 0..number_qubits {
                        let ro_index = match rm.qubit_mapping() {
                            Some(mp) => mp.get(&qb).unwrap_or(&qb),
                            None => &qb,
                        };
                        let mqb_new: Operation =
                            MeasureQubit::new(qb, rm.readout().to_owned(), *ro_index).into();
                        call_operation_with_device(
                            &mqb_new,
                            qureg,
                            bit_registers_internal,
                            float_registers_internal,
                            complex_registers_internal,
                            bit_registers_output,
                            device,
                        )?;
                    }
                }
                Some(_) => {
                    execute_pragma_repeated_measurement(
                        rm,
                        qureg,
                        bit_registers_internal,
                        bit_registers_output,
                    )?;
                }
            },
            Operation::MeasureQubit(internal_op) => {
                if let Some(position) = replace_measurements {
                    if internal_op.qubit() == &position {
                        if let Some(helper) = repeated_measurement_pragma.as_ref() {
                            execute_pragma_repeated_measurement(
                                helper,
                                qureg,
                                bit_registers_internal,
                                bit_registers_output,
                            )?;
                        }
                    }
                } else {
                    call_operation_with_device(
                        op,
                        qureg,
                        bit_registers_internal,
                        float_registers_internal,
                        complex_registers_internal,
                        bit_registers_output,
                        device,
                    )?;
                }
            }
            _ => {
                call_operation_with_device(
                    op,
                    qureg,
                    bit_registers_internal,
                    float_registers_internal,
                    complex_registers_internal,
                    bit_registers_output,
                    device,
                )?;
            }
        }
    }
    Ok(())
}

type InternalRegisters<'a> = (
    &'a mut HashMap<String, Vec<bool>>,
    &'a mut HashMap<String, Vec<f64>>,
    &'a mut HashMap<String, Vec<num_complex::Complex<f64>>>,
);

#[inline]
fn find_pragma_op(op: &&Operation) -> bool {
    match op {
        Operation::PragmaConditional(x) => x.circuit().iter().any(|x| find_pragma_op(&x)),
        Operation::PragmaLoop(x) => x.circuit().iter().any(|x| find_pragma_op(&x)),
        Operation::PragmaGetPauliProduct(x) => x.circuit().iter().any(|x| find_pragma_op(&x)),
        Operation::PragmaGetOccupationProbability(x) => {
            if let Some(circ) = x.circuit() {
                circ.iter().any(|x| find_pragma_op(&x))
            } else {
                false
            }
        }
        Operation::PragmaGetDensityMatrix(x) => {
            if let Some(circ) = x.circuit() {
                circ.iter().any(|x| find_pragma_op(&x))
            } else {
                false
            }
        }
        Operation::PragmaDamping(_) => true,
        Operation::PragmaDephasing(_) => true,
        Operation::PragmaDepolarising(_) => true,
        Operation::PragmaGeneralNoise(_) => true,
        Operation::PragmaSetDensityMatrix(_) => true,
        _ => false,
    }
}

#[cfg(feature = "async")]
#[async_trait]
impl AsyncEvaluatingBackend for Backend {
    async fn async_run_circuit_iterator<'a>(
        &self,
        circuit: impl Iterator<Item = &'a Operation> + std::marker::Send,
    ) -> RegisterResult {
        self.run_circuit_iterator(circuit)
    }

    #[cfg(feature = "parallelization")]
    async fn async_run_measurement_registers<T>(&self, measurement: &T) -> RegisterResult
    where
        T: Measure,
        T: std::marker::Sync,
    {
        self.run_measurement_registers(measurement)
    }
}
