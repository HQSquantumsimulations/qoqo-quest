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

use bincode::{deserialize, serialize};
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyByteArray;
use qoqo::convert_into_circuit;
use qoqo::QoqoBackendError;
use roqoqo::backends::EvaluatingBackend;
use roqoqo::registers::{BitOutputRegister, ComplexOutputRegister, FloatOutputRegister};
use roqoqo::Circuit;
use std::collections::HashMap;

/// QuEST backend
///
/// provides functions to run circuits and measurements on with the QuEST quantum simulator.
#[pyclass(name = "Backend", module = "qoqo_quest")]
#[derive(Clone, Debug, PartialEq)]
pub struct BackendWrapper {
    /// Internal storage of [roqoqo_quest::Backend]
    pub internal: roqoqo_quest::Backend,
}

/// Type of registers returned from a run of a Circuit.
pub type Registers = (
    HashMap<String, BitOutputRegister>,
    HashMap<String, FloatOutputRegister>,
    HashMap<String, ComplexOutputRegister>,
);

#[pymethods]
impl BackendWrapper {
    /// Create a new QuEST Backend.
    ///
    /// Args:
    ///     number_qubits (int): Number of qubits simulated in the backend.
    ///
    /// Raises:
    ///     RuntimeError: Error creating new backend
    #[new]
    pub fn new(number_qubits: usize) -> PyResult<Self> {
        Ok(Self {
            internal: roqoqo_quest::Backend::new(number_qubits),
        })
    }

    /// Return a copy of the Backend (copy here produces a deepcopy).
    ///
    /// Returns:
    ///     Backend: A deep copy of self.
    pub fn __copy__(&self) -> BackendWrapper {
        self.clone()
    }

    /// Return a deep copy of the Backend.
    ///
    /// Returns:
    ///     Backend: A deep copy of self.
    pub fn __deepcopy__(&self, _memodict: Py<PyAny>) -> BackendWrapper {
        self.clone()
    }

    /// Return the bincode representation of the Backend using the [bincode] crate.
    ///
    /// Returns:
    ///     ByteArray: The serialized Backend (in [bincode] form).
    ///
    /// Raises:
    ///     ValueError: Cannot serialize Backend to bytes.
    pub fn to_bincode(&self) -> PyResult<Py<PyByteArray>> {
        let serialized = serialize(&self.internal)
            .map_err(|_| PyValueError::new_err("Cannot serialize Backend to bytes"))?;
        let b: Py<PyByteArray> = Python::with_gil(|py| -> Py<PyByteArray> {
            PyByteArray::new(py, &serialized[..]).into()
        });
        Ok(b)
    }

    /// Convert the bincode representation of the Backend to a Backend using the [bincode] crate.
    ///
    /// Args:
    ///     input (ByteArray): The serialized Backend (in [bincode] form).
    ///
    /// Returns:
    ///     Backend: The deserialized Backend.
    ///
    /// Raises:
    ///     TypeError: Input cannot be converted to byte array.
    ///     ValueError: Input cannot be deserialized to Backend.
    pub fn from_bincode(&self, input: &PyAny) -> PyResult<BackendWrapper> {
        let bytes = input
            .extract::<Vec<u8>>()
            .map_err(|_| PyTypeError::new_err("Input cannot be converted to byte array"))?;

        Ok(BackendWrapper {
            internal: deserialize(&bytes[..])
                .map_err(|_| PyValueError::new_err("Input cannot be deserialized to Backend"))?,
        })
    }

    /// Return the json representation of the Backend.
    ///
    /// Returns:
    ///     str: The serialized form of Backend.
    ///
    /// Raises:
    ///     ValueError: Cannot serialize Backend to json.
    fn to_json(&self) -> PyResult<String> {
        let serialized = serde_json::to_string(&self.internal)
            .map_err(|_| PyValueError::new_err("Cannot serialize Backend to json"))?;
        Ok(serialized)
    }

    /// Convert the json representation of a Backend to a Backend.
    ///
    /// Args:
    ///     input (str): The serialized Backend in json form.
    ///
    /// Returns:
    ///     Backend: The deserialized Backend.
    ///
    /// Raises:
    ///     ValueError: Input cannot be deserialized to Backend.
    fn from_json(&self, input: &str) -> PyResult<BackendWrapper> {
        Ok(BackendWrapper {
            internal: serde_json::from_str(input)
                .map_err(|_| PyValueError::new_err("Input cannot be deserialized to Backend"))?,
        })
    }

    /// Run a circuit with the QuEST backend.
    ///
    /// A circuit is passed to the backend and executed.
    /// During execution values are written to and read from classical registers
    /// (List[bool], List[float], List[complex]).
    /// To produce sufficient statistics for evaluating expectation values,
    /// circuits have to be run multiple times.
    /// The results of each repetition are concatenated in OutputRegisters
    /// (List[List[bool]], List[List[float]], List[List[complex]]).
    /// As a simulater Backend the QuEST backend also allows to direclty read out
    /// the statevector, density matrix or the expectation values of products of PauliOperators
    ///
    ///
    /// Args:
    ///     circuit (Circuit): The circuit that is run on the backend.
    ///
    /// Returns:
    ///     Tuple[Dict[str, List[List[bool]]], Dict[str, List[List[float]]]], Dict[str, List[List[complex]]]]: The output registers written by the evaluated circuits.
    ///
    /// Raises:
    ///     TypeError: Circuit argument cannot be converted to qoqo Circuit
    ///     RuntimeError: Running Circuit failed
    pub fn run_circuit(&self, circuit: &PyAny) -> PyResult<Registers> {
        let circuit = convert_into_circuit(circuit).map_err(|err| {
            PyTypeError::new_err(format!(
                "Circuit argument cannot be converted to qoqo Circuit {:?}",
                err
            ))
        })?;
        EvaluatingBackend::run_circuit(&self.internal, &circuit)
            .map_err(|err| PyRuntimeError::new_err(format!("Running Circuit failed {:?}", err)))
    }

    /// Run all circuits corresponding to one measurement with the QuEST backend.
    ///
    /// An expectation value measurement in general involves several circuits.
    /// Each circuit is passes to the backend and executed separately.
    /// A circuit is passed to the backend and executed.
    /// During execution values are written to and read from classical registers
    /// (List[bool], List[float], List[complex]).
    /// To produce sufficient statistics for evaluating expectation values,
    /// circuits have to be run multiple times.
    /// The results of each repetition are concatenated in OutputRegisters
    /// (List[List[bool]], List[List[float]], List[List[complex]]).  
    ///
    ///
    /// Args:
    ///     measurement (Measurement): The measurement that is run on the backend.
    ///
    /// Returns:
    ///     Tuple[Dict[str, List[List[bool]]], Dict[str, List[List[float]]]], Dict[str, List[List[complex]]]]: The output registers written by the evaluated circuits.
    ///
    /// Raises:
    ///     TypeError: Cannot extract constant circuit from measurement
    ///     RuntimeError: Running Circuit failed
    pub fn run_measurement_registers(&self, measurement: &PyAny) -> PyResult<Registers> {
        let mut run_circuits: Vec<Circuit> = Vec::new();

        let get_constant_circuit = measurement
            .call_method0("constant_circuit")
            .map_err(|err| {
                PyTypeError::new_err(format!(
                    "Cannot extract constant circuit from measurement {:?}",
                    err
                ))
            })?;
        let const_circuit = get_constant_circuit
            .extract::<Option<&PyAny>>()
            .map_err(|err| {
                PyTypeError::new_err(format!(
                    "Cannot extract constant circuit from measurement {:?}",
                    err
                ))
            })?;

        let constant_circuit = match const_circuit {
            Some(x) => convert_into_circuit(x).map_err(|err| {
                PyTypeError::new_err(format!(
                    "Cannot extract constant circuit from measurement {:?}",
                    err
                ))
            })?,
            None => Circuit::new(),
        };

        let get_circuit_list = measurement.call_method0("circuits").map_err(|err| {
            PyTypeError::new_err(format!(
                "Cannot extract circuit list from measurement {:?}",
                err
            ))
        })?;
        let circuit_list = get_circuit_list.extract::<Vec<&PyAny>>().map_err(|err| {
            PyTypeError::new_err(format!(
                "Cannot extract circuit list from measurement {:?}",
                err
            ))
        })?;

        for c in circuit_list {
            run_circuits.push(
                constant_circuit.clone()
                    + convert_into_circuit(c).map_err(|err| {
                        PyTypeError::new_err(format!(
                            "Cannot extract circuit of circuit list from measurement {:?}",
                            err
                        ))
                    })?,
            )
        }

        let mut bit_registers: HashMap<String, BitOutputRegister> = HashMap::new();
        let mut float_registers: HashMap<String, FloatOutputRegister> = HashMap::new();
        let mut complex_registers: HashMap<String, ComplexOutputRegister> = HashMap::new();

        for circuit in run_circuits {
            let (tmp_bit_reg, tmp_float_reg, tmp_complex_reg) = self
                .internal
                .run_circuit_iterator(circuit.iter())
                .map_err(|err| {
                    PyRuntimeError::new_err(format!("Running a circuit failed {:?}", err))
                })?;

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

    /// Evaluates expectation values of a measurement with the backend.
    ///
    ///
    /// Args:
    ///     measurement (Measurement): The measurement that is run on the backend.
    ///
    /// Returns:
    ///     Optional[Dict[str, float]]: The  dictionary of expectation values.
    ///
    /// Raises:
    ///     TypeError: Measurement evaluate function could not be used
    ///     RuntimeError: Internal error measurement.evaluation returned unknown type
    pub fn run_measurement(&self, measurement: &PyAny) -> PyResult<Option<HashMap<String, f64>>> {
        let (bit_registers, float_registers, complex_registers) = self.run_measurement_registers(measurement)?;
        let get_expectation_values = measurement
            .call_method1("evaluate", (bit_registers, float_registers, complex_registers))
            .map_err(|err| PyTypeError::new_err(format!("Measurement evaluate function could not be used: {:?}", err)))?;
        get_expectation_values
            .extract::<Option<HashMap<String, f64>>>()
            .map_err(|_| PyRuntimeError::new_err("Internal error measurement.evaluation returned unknown type"))
    }
}

/// Convert generic python object to [roqoqo_quest::Backend].
///
/// Fallible conversion of generic python object to [roqoqo_quest::Backend].
pub fn convert_into_backend(input: &PyAny) -> Result<roqoqo_quest::Backend, QoqoBackendError> {
    if let Ok(try_downcast) = input.extract::<BackendWrapper>() {
        Ok(try_downcast.internal)
    } else {
        // Everything that follows tries to extract the circuit when two separately
        // compiled python packages are involved
        let get_bytes = input
            .call_method0("_enum_to_bincode")
            .map_err(|_| QoqoBackendError::CannotExtractObject)?;
        let bytes = get_bytes
            .extract::<Vec<u8>>()
            .map_err(|_| QoqoBackendError::CannotExtractObject)?;
        deserialize(&bytes[..]).map_err(|_| QoqoBackendError::CannotExtractObject)
    }
}
