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
use crate::Vector;
use num_complex::Complex64;
use roqoqo::operations::*;
use roqoqo::registers::{BitOutputRegister, BitRegister, ComplexRegister, FloatRegister};
use roqoqo::Circuit;
use roqoqo::RoqoqoBackendError;
use std::collections::HashMap;
use std::convert::TryFrom;
mod pragma_operations;
use pragma_operations::*;
mod gate_operations;
use gate_operations::*;
mod preprocessing;
pub(crate) use pragma_operations::{
    execute_pragma_repeated_measurement, execute_replaced_repeated_measurement,
};
pub use preprocessing::get_number_used_qubits_and_registers;

// Pragma operations that are ignored by backend and do not throw an error
const ALLOWED_OPERATIONS: &[&str; 11] = &[
    "PragmaSetNumberOfMeasurements",
    "PragmaBoostNoise",
    "PragmaStopParallelBlock",
    "PragmaGlobalPhase",
    "DefinitionUsize",
    "InputSymbolic",
    "PragmaRepeatGate",
    "PragmaStartDecompositionBlock",
    "PragmaStopDecompositionBlock",
    "PragmaOverrotation",
    "PragmaSleep",
];

/// Simulate all operations in a [roqoqo::Circuit] acting on a quantum register
///
/// # Arguments
///
/// `circuit` - The [roqoqo::Circuit] that is simulated
/// `qureg` - The wrapper around a QuEST quantum register on which the operations act
/// `bit_registers` - The HashMap of bit registers ([Vec<bool>]) to write measurement results to
/// `float_registers` - The HashMap of float registers ([Vec<f64>]) to write real values extracted from the simulator to
/// `complex_registers` - The HashMap of complex registers ([Vec<Complex64>])
///                     to write complex values extracted from the simulator to
/// `bit_registers_output` - The HashMap of bit output registers ([Vec<Vec<bool>>])
///                          to write measurements of simulated repetitions of circuit execution to
pub fn call_circuit(
    circuit: &Circuit,
    qureg: &mut Qureg,
    bit_registers: &mut HashMap<String, BitRegister>,
    float_registers: &mut HashMap<String, FloatRegister>,
    complex_registers: &mut HashMap<String, ComplexRegister>,
    bit_registers_output: &mut HashMap<String, BitOutputRegister>,
) -> Result<(), RoqoqoBackendError> {
    for op in circuit.iter() {
        call_operation(
            op,
            qureg,
            bit_registers,
            float_registers,
            complex_registers,
            bit_registers_output,
        )?
    }
    Ok(())
}

/// Simulate all operations that are available in a [roqoqo::Circuit] acting on a quantum register
///
/// When the optional device parameter is not None availability checks will be performed.
/// The availability of the operation on a specific device is checked first.
/// The function returns an error if the operation is not available on the device
/// even if it can be simulated with the QuEST simulator.
///
/// # Arguments
///
/// `circuit` - The [roqoqo::Circuit] that is simulated
/// `qureg` - The wrapper around a QuEST quantum register on which the operations act
/// `bit_registers` - The HashMap of bit registers ([Vec<bool>]) to write measurement results to
/// `float_registers` - The HashMap of float registers ([Vec<f64>]) to write real values extracted from the simulator to
/// `complex_registers` - The HashMap of complex registers ([Vec<Complex64>])
///                     to write complex values extracted from the simulator to
/// `bit_registers_output` - The HashMap of bit output registers ([Vec<Vec<bool>>])
///                          to write measurements of simulated repetitions of circuit execution
/// `device` - The optional [roqoqo::devices::Device] that determines the availability of operations
pub fn call_circuit_with_device(
    circuit: &Circuit,
    qureg: &mut Qureg,
    bit_registers: &mut HashMap<String, BitRegister>,
    float_registers: &mut HashMap<String, FloatRegister>,
    complex_registers: &mut HashMap<String, ComplexRegister>,
    bit_registers_output: &mut HashMap<String, BitOutputRegister>,
    device: &mut Option<Box<dyn roqoqo::devices::Device>>,
) -> Result<(), RoqoqoBackendError> {
    for op in circuit.iter() {
        call_operation_with_device(
            op,
            qureg,
            bit_registers,
            float_registers,
            complex_registers,
            bit_registers_output,
            device,
        )?
    }
    Ok(())
}

/// Simulates a single operation ([roqoqo::operations::Operation]) acting on a quantum register
///
/// # Arguments
///
/// `operations` - The [roqoqo::operations::Operation] that is simulated
/// `qureg` - The wrapper around a QuEST quantum register on which the operations act
/// `bit_registers` - The HashMap of bit registers ([Vec<bool>]) to write measurement results to
/// `float_registers` - The HashMap of float registers ([Vec<f64>]) to write real values extracted from the simulator to
/// `complex_registers` - The HashMap of complex registers ([Vec<Complex64>])
///                     to write complex values extracted from the simulator to
/// `bit_registers_output` - The HashMap of bit output registers ([Vec<Vec<bool>>])
///                          to write measurements of simulated repetitions of circuit execution to
pub fn call_operation(
    operation: &Operation,
    qureg: &mut Qureg,
    bit_registers: &mut HashMap<String, BitRegister>,
    float_registers: &mut HashMap<String, FloatRegister>,
    complex_registers: &mut HashMap<String, ComplexRegister>,
    bit_registers_output: &mut HashMap<String, BitOutputRegister>,
) -> Result<(), RoqoqoBackendError> {
    call_operation_with_device(
        operation,
        qureg,
        bit_registers,
        float_registers,
        complex_registers,
        bit_registers_output,
        &mut None,
    )
}

/// Simulates a single available operation ([roqoqo::operations::Operation]) acting on a quantum
/// register.
///
/// When the optional device parameter is not None availability checks will be performed. The
/// availability of the operation on a specific device is checked first. The function returns an
/// error if the operation is not available on the device even if it can be simulated with the QuEST
/// simulator.
///
///
/// # Arguments
///
/// * `operation` - The [roqoqo::operations::Operation] that is simulated
/// * `qureg` - The wrapper around a QuEST quantum register on which the operations act
/// * `bit_registers` - The HashMap of bit registers ([Vec<bool>]) to write measurement results to
/// * `float_registers` - The HashMap of float registers ([Vec<f64>]) to write real values extracted
///   from the simulator to
/// * `complex_registers` - The HashMap of complex registers ([Vec<Complex64>])
///   to write complex values extracted from the simulator to
/// * `bit_registers_output` - The HashMap of bit output registers ([Vec<Vec<bool>>])
///   to write measurements of simulated repetitions of circuit execution to
/// * `device` - The optional [roqoqo::devices::Device] that determines the availability of
///   operations
///
/// # Returns
///
/// * `Err(RoqoqoBackendError)` - Something went wrong while processing the operation.
pub fn call_operation_with_device(
    operation: &Operation,
    qureg: &mut Qureg,
    bit_registers: &mut HashMap<String, BitRegister>,
    float_registers: &mut HashMap<String, FloatRegister>,
    complex_registers: &mut HashMap<String, ComplexRegister>,
    bit_registers_output: &mut HashMap<String, BitOutputRegister>,
    device: &mut Option<Box<dyn roqoqo::devices::Device>>,
) -> Result<(), RoqoqoBackendError> {
    match operation {
        Operation::PragmaStopParallelBlock(_) => Ok(()),
        Operation::DefinitionBit(def) => {
            if *def.is_output() {
                bit_registers.insert(def.name().clone(), vec![false; *def.length()]);
            }
            Ok(())
        }
        Operation::DefinitionFloat(def) => {
            if *def.is_output() {
                float_registers.insert(def.name().clone(), vec![0.0; *def.length()]);
            }
            Ok(())
        }
        Operation::DefinitionComplex(def) => {
            if *def.is_output() {
                complex_registers.insert(
                    def.name().clone(),
                    vec![Complex64::new(0.0, 0.0); *def.length()],
                );
            }
            Ok(())
        }
        Operation::PragmaRepeatedMeasurement(op) => {
            execute_pragma_repeated_measurement(op, qureg, bit_registers, bit_registers_output)
        }
        Operation::MeasureQubit(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            let register =
                bit_registers
                    .get_mut(op.readout())
                    .ok_or(RoqoqoBackendError::GenericError {
                        msg: format!("Bit register {} not found to write output to", op.readout()),
                    })?;
            let res = unsafe { quest_sys::measure(qureg.quest_qureg, *op.qubit() as i32) };
            register[*op.readout_index()] = res == 1;
            Ok(())
        }
        Operation::PragmaSetStateVector(op) => execute_pragma_set_state_vector(op, qureg),
        Operation::PragmaSetDensityMatrix(op) => execute_pragma_set_density_matrix(op, qureg),
        Operation::PragmaGetStateVector(op) => {
            execute_pragma_get_state_vector(op, qureg, complex_registers)
        }
        Operation::PragmaGetDensityMatrix(op) => {
            execute_pragma_get_density_matrix(op, qureg, complex_registers)
        }
        Operation::PragmaGetPauliProduct(op) => execute_get_pauli_prod(
            op,
            qureg,
            float_registers,
            bit_registers,
            complex_registers,
            bit_registers_output,
            device,
        ),
        Operation::PragmaGetOccupationProbability(op) => execute_get_occupation_probability(
            op,
            qureg,
            float_registers,
            bit_registers,
            complex_registers,
            bit_registers_output,
            device,
            call_circuit_with_device,
        ),
        Operation::PragmaActiveReset(op) => {
            unsafe {
                if quest_sys::measure(qureg.quest_qureg, *op.qubit() as i32) == 1 {
                    quest_sys::pauliX(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int)
                }
            }
            Ok(())
        }
        Operation::PragmaConditional(op) => {
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
                        call_circuit_with_device(
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
        Operation::RotateX(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe {
                quest_sys::rotateX(
                    qureg.quest_qureg,
                    *op.qubit() as ::std::os::raw::c_int,
                    *op.theta().float()?,
                )
            }
            Ok(())
        }
        Operation::RotateY(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe {
                quest_sys::rotateY(
                    qureg.quest_qureg,
                    *op.qubit() as ::std::os::raw::c_int,
                    *op.theta().float()?,
                )
            }
            Ok(())
        }
        Operation::RotateZ(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe {
                quest_sys::rotateZ(
                    qureg.quest_qureg,
                    *op.qubit() as ::std::os::raw::c_int,
                    *op.theta().float()?,
                )
            }
            Ok(())
        }
        Operation::PhaseShiftState1(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe {
                quest_sys::phaseShift(
                    qureg.quest_qureg,
                    *op.qubit() as ::std::os::raw::c_int,
                    *op.theta().float()?,
                )
            }
            Ok(())
        }
        Operation::PauliX(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::pauliX(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::PauliY(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::pauliY(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::PauliZ(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::pauliZ(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::Hadamard(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::hadamard(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::SGate(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::sGate(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::TGate(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::tGate(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::SqrtPauliX(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe {
                quest_sys::rotateX(
                    qureg.quest_qureg,
                    *op.qubit() as ::std::os::raw::c_int,
                    std::f64::consts::FRAC_PI_2,
                )
            }
            Ok(())
        }
        Operation::InvSqrtPauliX(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            unsafe {
                quest_sys::rotateX(
                    qureg.quest_qureg,
                    *op.qubit() as ::std::os::raw::c_int,
                    std::f64::consts::FRAC_PI_2 * -1.0,
                )
            }
            Ok(())
        }
        Operation::RotateAroundSphericalAxis(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_single_qubit_availability(op, device)?;
            let vector: Vector = Vector::new(
                op.spherical_theta().sin().float()? * op.spherical_phi().cos().float()?,
                op.spherical_theta().sin().float()? * op.spherical_phi().sin().float()?,
                *op.spherical_theta().cos().float()?,
            );
            unsafe {
                quest_sys::rotateAroundAxis(
                    qureg.quest_qureg,
                    *op.qubit() as ::std::os::raw::c_int,
                    *op.theta().float()?,
                    vector.vector,
                )
            }
            Ok(())
        }
        Operation::CNOT(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_two_qubit_availability(op, device)?;
            unsafe {
                quest_sys::controlledNot(
                    qureg.quest_qureg,
                    *op.control() as ::std::os::raw::c_int,
                    *op.target() as ::std::os::raw::c_int,
                )
            }
            Ok(())
        }
        Operation::ControlledPhaseShift(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_two_qubit_availability(op, device)?;
            unsafe {
                quest_sys::controlledPhaseShift(
                    qureg.quest_qureg,
                    *op.control() as ::std::os::raw::c_int,
                    *op.target() as ::std::os::raw::c_int,
                    *op.theta().float()?,
                )
            }
            Ok(())
        }
        Operation::ControlledPauliY(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_two_qubit_availability(op, device)?;
            unsafe {
                quest_sys::controlledPauliY(
                    qureg.quest_qureg,
                    *op.control() as ::std::os::raw::c_int,
                    *op.target() as ::std::os::raw::c_int,
                )
            }
            Ok(())
        }
        Operation::ControlledPauliZ(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_two_qubit_availability(op, device)?;
            unsafe {
                quest_sys::controlledPhaseFlip(
                    qureg.quest_qureg,
                    *op.control() as ::std::os::raw::c_int,
                    *op.target() as ::std::os::raw::c_int,
                )
            }
            Ok(())
        }
        Operation::ControlledRotateX(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_two_qubit_availability(op, device)?;
            unsafe {
                quest_sys::controlledRotateX(
                    qureg.quest_qureg,
                    *op.control() as ::std::os::raw::c_int,
                    *op.target() as ::std::os::raw::c_int,
                    *op.theta().float()?,
                )
            }
            Ok(())
        }
        Operation::ControlledRotateXY(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_two_qubit_availability(op, device)?;
            let rotate_op = RotateXY::new(0, op.theta().clone(), op.phi().clone());
            let unitary_matrix = rotate_op.unitary_matrix()?;
            let complex_matrix = quest_sys::ComplexMatrix2 {
                // row major version only used for Complex2/4/N intio
                real: [
                    [unitary_matrix[(0, 0)].re, unitary_matrix[(0, 1)].re],
                    [unitary_matrix[(1, 0)].re, unitary_matrix[(1, 1)].re],
                ],
                imag: [
                    [unitary_matrix[(0, 0)].im, unitary_matrix[(0, 1)].im],
                    [unitary_matrix[(1, 0)].im, unitary_matrix[(1, 1)].im],
                ],
            };
            unsafe {
                quest_sys::controlledUnitary(
                    qureg.quest_qureg,
                    *op.control() as ::std::os::raw::c_int,
                    *op.target() as ::std::os::raw::c_int,
                    complex_matrix,
                )
            }
            Ok(())
        }
        Operation::SWAP(op) => {
            check_acts_on_qubits_in_qureg(operation, qureg)?;
            check_two_qubit_availability(op, device)?;
            unsafe {
                quest_sys::swapGate(
                    qureg.quest_qureg,
                    *op.control() as ::std::os::raw::c_int,
                    *op.target() as ::std::os::raw::c_int,
                )
            }
            Ok(())
        }
        Operation::PragmaDamping(op) => {
            if acts_on_qubits_in_qureg(operation, qureg) {
                unsafe {
                    quest_sys::mixDamping(
                        qureg.quest_qureg,
                        *op.qubit() as ::std::os::raw::c_int,
                        f64::try_from(op.probability())?,
                    )
                }
            }
            Ok(())
        }
        Operation::PragmaDephasing(op) => {
            if acts_on_qubits_in_qureg(operation, qureg) {
                unsafe {
                    quest_sys::mixDephasing(
                        qureg.quest_qureg,
                        *op.qubit() as ::std::os::raw::c_int,
                        f64::try_from(op.probability())?,
                    )
                }
            }
            Ok(())
        }
        Operation::PragmaDepolarising(op) => {
            if acts_on_qubits_in_qureg(operation, qureg) {
                unsafe {
                    quest_sys::mixDepolarising(
                        qureg.quest_qureg,
                        *op.qubit() as ::std::os::raw::c_int,
                        f64::try_from(op.probability())?,
                    )
                }
            }
            Ok(())
        }
        Operation::PragmaGeneralNoise(op) => {
            if acts_on_qubits_in_qureg(operation, qureg) {
                execute_generic_single_qubit_noise(op, qureg)
            } else {
                Ok(())
            }
        }
        Operation::PragmaChangeDevice(op) => {
            if let Some(device_box) = device {
                device_box.change_device(&op.wrapped_hqslang, &op.wrapped_operation)?;
            }
            Ok(())
        }
        Operation::PragmaLoop(op) => execute_pragma_loop(
            op,
            qureg,
            bit_registers,
            float_registers,
            complex_registers,
            bit_registers_output,
            device,
        ),
        Operation::InputBit(op) => execute_pragma_input_bit(op, bit_registers),
        Operation::PragmaRandomNoise(op) => execute_pragma_random_noise(op, qureg),
        _ => {
            if let Ok(op) = TwoQubitGateOperation::try_from(operation) {
                check_acts_on_qubits_in_qureg(operation, qureg)?;
                check_two_qubit_availability(&op, device)?;
                execute_generic_two_qubit_operation(&op, qureg)
            } else if let Ok(op) = SingleQubitGateOperation::try_from(operation) {
                check_acts_on_qubits_in_qureg(operation, qureg)?;
                check_single_qubit_availability(&op, device)?;
                execute_generic_single_qubit_operation(&op, qureg)
            } else if let Ok(op) = ThreeQubitGateOperation::try_from(operation) {
                check_acts_on_qubits_in_qureg(operation, qureg)?;
                check_three_qubit_availability(&op, device)?;
                execute_generic_three_qubit_operation(&op, qureg)
            } else if let Ok(op) = MultiQubitGateOperation::try_from(operation) {
                check_acts_on_qubits_in_qureg(operation, qureg)?;
                check_multi_qubit_availability(&op, device)?;
                execute_generic_multi_qubit_operation(&op, qureg)
            } else if let Ok(_op) = PragmaNoiseOperation::try_from(operation) {
                // Not working yet WIP
                // execute_generic_single_qubit_noise(&_op, qureg)
                Err(RoqoqoBackendError::OperationNotInBackend {
                    backend: "QuEST",
                    hqslang: operation.hqslang(),
                })
            } else if ALLOWED_OPERATIONS.contains(&operation.hqslang()) {
                Ok(())
            } else {
                Err(RoqoqoBackendError::OperationNotInBackend {
                    backend: "QuEST",
                    hqslang: operation.hqslang(),
                })
            }
        }
    }
}

#[inline]
fn check_single_qubit_availability<T>(
    op: &T,
    device: &Option<Box<dyn roqoqo::devices::Device>>,
) -> Result<(), RoqoqoBackendError>
where
    T: OperateSingleQubit,
{
    if let Some(device_box) = device {
        match device_box.single_qubit_gate_time(op.hqslang(), op.qubit()) {
            Some(_) => Ok(()),
            None => Err(RoqoqoBackendError::GenericError {
                msg: format!(
                    "Operation {:?} not available for qubit {} in device",
                    op,
                    op.qubit()
                ),
            }),
        }
    } else {
        Ok(())
    }
}

#[inline]
fn check_two_qubit_availability<T>(
    op: &T,
    device: &Option<Box<dyn roqoqo::devices::Device>>,
) -> Result<(), RoqoqoBackendError>
where
    T: OperateTwoQubit,
{
    if let Some(device_box) = device {
        match device_box.two_qubit_gate_time(op.hqslang(), op.control(), op.target()) {
            Some(_) => Ok(()),
            None => Err(RoqoqoBackendError::GenericError {
                msg: format!(
                    "Operation {:?} not available for control {} target {} in device",
                    op,
                    op.control(),
                    op.target()
                ),
            }),
        }
    } else {
        Ok(())
    }
}

#[inline]
fn check_three_qubit_availability<T>(
    op: &T,
    device: &Option<Box<dyn roqoqo::devices::Device>>,
) -> Result<(), RoqoqoBackendError>
where
    T: OperateThreeQubit,
{
    if let Some(device_box) = device {
        match device_box.three_qubit_gate_time(op.hqslang(), op.control_0(), op.control_1(), op.target()) {
            Some(_) => Ok(()),
            _ => Err(RoqoqoBackendError::GenericError {
                msg: format!(
                    "Operation {:?} not available for control_0 {} control0_1 {} target {} in device",
                    op,
                    op.control_0(),
                    op.control_1(),
                    op.target()
                ),
            }),
        }
    } else {
        Ok(())
    }
}

#[inline]
fn check_acts_on_qubits_in_qureg(
    operation: &Operation,
    qureg: &Qureg,
) -> Result<(), RoqoqoBackendError> {
    let number_qubits = qureg.number_qubits() as usize;
    if let InvolvedQubits::Set(involved_qubits) = operation.involved_qubits() {
        for q in involved_qubits.iter() {
            if *q >= number_qubits {
                return Err(RoqoqoBackendError::GenericError {
                    msg: format!(
                        "Not enough qubits reserved. QuEST simulator used {} qubits but operation \
                         acting on {}",
                        number_qubits, q
                    ),
                });
            }
        }
    }
    Ok(())
}

#[inline]
fn acts_on_qubits_in_qureg(operation: &Operation, qureg: &Qureg) -> bool {
    let number_qubits = qureg.number_qubits() as usize;
    if let InvolvedQubits::Set(involved_qubits) = operation.involved_qubits() {
        for q in involved_qubits.iter() {
            if *q >= number_qubits {
                return false;
            }
        }
    }
    true
}

#[inline]
fn check_multi_qubit_availability<T>(
    op: &T,
    device: &Option<Box<dyn roqoqo::devices::Device>>,
) -> Result<(), RoqoqoBackendError>
where
    T: OperateMultiQubit,
{
    if let Some(device_box) = device {
        match device_box.multi_qubit_gate_time(op.hqslang(), op.qubits()) {
            Some(_) => Ok(()),
            None => Err(RoqoqoBackendError::GenericError {
                msg: format!(
                    "Operation {:?} not available for qubits {:?} in device",
                    op,
                    op.qubits()
                ),
            }),
        }
    } else {
        Ok(())
    }
}
