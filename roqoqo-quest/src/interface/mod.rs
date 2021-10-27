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

// Pragma operations that are ignored by backend and do not throw an error
const ALLOWED_OPERATIONS: &[&str; 10] = &[
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

/// Simulates a single available operation ([roqoqo::operations::Operation]) acting on a quantum register
///
/// When the optional device parameter is not None availability checks will be performed.
/// The availability of the operation on a specific device is checked first.
/// The function returns an error if the operation is not available on the device
/// even if it can be simulated with the QuEST simulator.
///
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
/// `device` - The optional [roqoqo::devices::Device] that determines the availability of operations
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
            unsafe {
                let register = bit_registers.get_mut(op.readout()).ok_or(
                    RoqoqoBackendError::GenericError {
                        msg: format!("Bit register {} not found to write output to", op.readout()),
                    },
                )?;
                register[*op.readout_index()] =
                    quest_sys::measure(qureg.quest_qureg, *op.qubit() as i32) == 1;
            }
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
        Operation::PragmaGetPauliProduct(op) => {
            if op.qubit_paulis().is_empty() {
                float_registers.insert(op.readout().clone(), vec![1.0]);
                return Ok(());
            }
            unsafe {
                let workspace = Qureg::new(qureg.number_qubits(), qureg.is_density_matrix);
                let workspace_pp = Qureg::new(qureg.number_qubits(), qureg.is_density_matrix);
                if !op.circuit().is_empty() {
                    call_circuit_with_device(
                        &op.circuit(),
                        qureg,
                        bit_registers,
                        float_registers,
                        complex_registers,
                        bit_registers_output,
                        device,
                    )?;
                }
                quest_sys::cloneQureg(workspace.quest_qureg, qureg.quest_qureg);
                let mut qubits: Vec<i32> = op
                    .qubit_paulis()
                    .keys()
                    .cloned()
                    .map(|x| x as i32)
                    .collect();
                let mut paulis: Vec<u32> = op
                    .qubit_paulis()
                    .values()
                    .cloned()
                    .map(|x| x as u32)
                    .collect();
                let pp = quest_sys::calcExpecPauliProd(
                    workspace.quest_qureg,
                    qubits.as_mut_ptr(),
                    paulis.as_mut_ptr(),
                    qubits.len() as i32,
                    workspace_pp.quest_qureg,
                );
                drop(workspace);
                drop(workspace_pp);
                float_registers.insert(op.readout().clone(), vec![pp]);
            }
            Ok(())
        }
        Operation::PragmaGetOccupationProbability(op) => {
            unsafe {
                let mut workspace = Qureg::new(qureg.number_qubits(), qureg.is_density_matrix);
                match op.circuit() {
                    Some(x) => {
                        call_circuit_with_device(
                            x,
                            qureg,
                            bit_registers,
                            float_registers,
                            complex_registers,
                            bit_registers_output,
                            device,
                        )?;
                    }
                    None => {}
                }
                quest_sys::cloneQureg(workspace.quest_qureg, qureg.quest_qureg);
                let probas: Vec<f64>;
                if qureg.is_density_matrix {
                    let op = PragmaGetDensityMatrix::new(op.readout().clone(), None);
                    let mut register: HashMap<String, ComplexRegister> = HashMap::new();
                    execute_pragma_get_density_matrix(&op, &mut workspace, &mut register)?;
                    match register.get(op.readout()) {
                        Some(p) => probas = p.iter().map(|x| x.re).collect(),
                        None => {
                            return Err(RoqoqoBackendError::GenericError {
                                msg: "Issue in get_density_matrix".to_string(),
                            });
                        }
                    }
                } else {
                    let op = PragmaGetStateVector::new(op.readout().clone(), None);
                    let mut register: HashMap<String, ComplexRegister> = HashMap::new();
                    execute_pragma_get_state_vector(&op, &mut workspace, &mut register)?;
                    match register.get(op.readout()) {
                        Some(p) => probas = p.iter().map(|x| x.re).collect(),
                        None => {
                            return Err(RoqoqoBackendError::GenericError {
                                msg: "Issue in get_state_vector".to_string(),
                            });
                        }
                    }
                }
                float_registers.insert(op.readout().clone(), probas);
            }
            Ok(())
        }
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
        Operation::PhaseShiftState0(op) => {
            check_single_qubit_availability(op, device)?;
            unsafe {
                quest_sys::rotateZ(
                    qureg.quest_qureg,
                    *op.qubit() as ::std::os::raw::c_int,
                    -*op.theta().float()?,
                )
            }
            Ok(())
        }
        Operation::PhaseShiftState1(op) => {
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
        Operation::PauliX(op) => {
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::pauliX(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::PauliY(op) => {
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::pauliY(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::PauliZ(op) => {
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::pauliZ(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::Hadamard(op) => {
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::hadamard(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::SGate(op) => {
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::sGate(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::TGate(op) => {
            check_single_qubit_availability(op, device)?;
            unsafe { quest_sys::tGate(qureg.quest_qureg, *op.qubit() as ::std::os::raw::c_int) }
            Ok(())
        }
        Operation::SqrtPauliX(op) => {
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
        Operation::SWAP(op) => {
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
            unsafe {
                quest_sys::mixDamping(
                    qureg.quest_qureg,
                    *op.qubit() as ::std::os::raw::c_int,
                    f64::try_from(op.probability())?,
                )
            }
            Ok(())
        }
        Operation::PragmaDephasing(op) => {
            unsafe {
                quest_sys::mixDephasing(
                    qureg.quest_qureg,
                    *op.qubit() as ::std::os::raw::c_int,
                    f64::try_from(op.probability())?,
                )
            }
            Ok(())
        }
        Operation::PragmaDepolarising(op) => {
            unsafe {
                quest_sys::mixDepolarising(
                    qureg.quest_qureg,
                    *op.qubit() as ::std::os::raw::c_int,
                    f64::try_from(op.probability())?,
                )
            }
            Ok(())
        }
        Operation::PragmaChangeDevice(op) => {
            if let Some(device_box) = device {
                device_box.change_device(&op.wrapped_operation)?;
            }
            Ok(())
        }
        // Operation::PragmaRandomNoise(op) => execute_pragma_random_noise(op, qureg),
        _ => {
            if let Ok(op) = TwoQubitGateOperation::try_from(operation) {
                check_two_qubit_availability(&op, device)?;
                execute_generic_two_qubit_operation(&op, qureg)
            } else if let Ok(op) = SingleQubitGateOperation::try_from(operation) {
                check_single_qubit_availability(&op, device)?;
                execute_generic_single_qubit_operation(&op, qureg)
            } else if let Ok(op) = MultiQubitGateOperation::try_from(operation) {
                check_mulit_qubit_availability(&op, device)?;
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

fn check_mulit_qubit_availability<T>(
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
