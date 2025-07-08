# Changelog

This changelog track changes to the qoqo-quest project starting at version 0.1.0

## Unpublished

## 0.16.2

* Added an optional `imperfect_readout_model` attribute to the `Backend`. If set the noisemodel will be used to simulate readout errors.
* Fixed unsound API by hiding "dimension" field of `ComplexMatrixN`.

## 0.16.1

* Updated to pyo3 0.24.

## 0.16.0

* Updated minimum supported Python version to 3.9.
* Updated minimum supported Rust versio to 1.76.
* Updated to pyo3 0.23.
* Updated to qoqo-calculator 1.5, qoqo 1.19 and rand 0.9.
* Added qoqo/.cargo/config file with aarch64 and x86_64 targets for macos.

## 0.15.1

* Relaxed numpy requirement (removing `>=2.0`).
* Added warning to any circuits using `PragmaGetStateVector` or `PragmaGetDensityMatrix` with a non-empty Circuit argument passed that this circuit isn't used. This was added to `run_measurement_registers` and `run_circuit` in qoqo-quest only.

## 0.15.0

* Updated to qoqo-calculator 1.4.4, qoqo 1.18.0, struqture 1.11.1, struqture 2.0.0-alpha.7, pyo3 0.22 and ndarray 0.16

## 0.14.5

* Updated to qoqo 1.16.0
* Removed unused dependencies

## 0.14.4

* Added the `set_random_seed` and `get_random_seed` methods to the `Backend`
* Set clap version to "=4.4" for it to use rust-version 1.70.0
* Updated to qoqo 1.15.2-alpha.3 and qoqo-calculator 1.2.3
* Removed faulty check for number of qubits in PragmaGetPauliProduct
* Updated to qoqo 1.15.2-alpha.1, rust 1.70 and maturin 1.4

## 0.14.3

* Fixed missing pyo3 build dependency

## 0.14.2

* Neglected to update quest-sys to the correct version. This aims to correct that oversight.

## 0.14.1

* Updated to qoqo 1.15 and struqture to 1.8

## 0.14.0

* Updated to qoqo 1.12.0 and pyo3 0.21.
* Clearer error message for qubits index out of range.
* Fixed dependencies issues caused by Pyo3 0.21 support release

## 0.13.1

* Fixing out-of-index bug when using MeasureQubit together with PragmaSetNumberOfMeasurements when measuring fewer qubits than are used in a circuit 

## 0.13.0

* Better error reporting when readout register size missmatches measurement outputs
* Added `run_program` function to qoqo-quest interface so a QuantumProgram can be run by invoking `backend.run_program(program, [0.1, 0.2])` where the list of floats are the values to be used for the free input parameters of the QuantumProgram.


## 0.12.4

* Changed the meaning of `number_qubits` in Backend. It mean the maximum number of qubits available for simulation. `Qureg` is now initalized with number of qubits being used in the circuit.

## 0.12.3

* Fixed bug in probabilities function in quest bindings.

## 0.12.2

* Fixed PragmaSleep in circuit returning error when it should be supported.

## 0.12.1

* Fixing bug when building for GPU

## 0.12.0

* Update to QuEST 3.7
* Option to build for CUDA
* Relaxed cutoff of reinterpreting negative probabilities of states as zero. Cutoff is now -1e-14 probabilities p with `-1e-14 < p < 0` will now be interpreted as zero.

## 0.11.3

* Updated to qoqo 1.8.0 and pyo3 0.20

## 0.11.2

* Updated to qoqo 1.7
* Bugfix for PragmaStopParallelBlock with MeasureQubit and PragmaSetNumberOfMeasurements

## 0.11.1

* Updated to qoqo 1.5
* Updated to pyo3 0.19

## 0.11

* Update to qoqo 1.4
* Fix missing error when PragmaSetNumberOfMeasurements is used without corresponding MeasureQubit

## 0.10.1

* Fix wrong global phase when applying PhaseShiftState0 and PHaseShiftState1 operations

## 0.10.0

* Update and support of qoqo 1.3
* Improved error when applying operation to qubit outside of qureg
* Ignoring PragmaNoise operations applied to qubits outside of qureg

## 0.9.2

* Fixed build.rs for quest-sys to support building on Linux, macos on x86_64 and aarch64 and on Windows.
* Updated to qoqo 1.3

## 0.9.1

* Update to pyo3 0.18 and enabling cross-compilation with zig

## 0.9.0

* Updated to qoqo 1.2.0

## 0.8.2

* Fixed error with small negative occupation probabilities when using damping by introducing a numerical accuracy cut-off.

## 0.8.1

* Updated to (ro)qoqo 1.1.0
* Added support for PragmaLoop
* Fixed bug using MeasureQubit with PragmaSetNumberMeasurements for only subset of qubits
* Support for `InputBit` operation expected in qoqo 1.1.0
* Support for `async` feature providing an `AsyncEvaluatingBackend` interface in roqoqo
* Support for `parallelization` feature, evaluating circuits from one `Measurement` in parallel
* Updated to QuEST 3.5.0

## 0.7.1

* Fixed: Bug in calculating result for PragmaGetPauliProduct

## 0.7.4

* Updated to qoqo 1.0.0

## 0.7.3

* Fixed general noise pragma simulation

## 0.7.2

* Updated dependencies

## 0.7.0

* Fixed: Using `PragmaRepeatedMeasurement` and `PragmaSetNumberOfMeasurements` now repeats the numerical circuit when other Measurements are present in the circuits.

## 0.6.0

### Changed 0.6.0

* Updated to qoqo 1.0.0-
* Updated to qoqo_calculator(_pyo3) 0.8.3

## 0.5.0

### Changed 0.5.0

* Updated to qoqo 1.0.0-alpha.1
* Dependencies updated for github workflows.

## 0.4.0

### Changed 0.4.0

* Updated to qoqo 0.11

## 0.3.0

### Added 0.3.0

* qoqo-quest can now be built using a source distribution
* Dependency updates to qoqo 0.10 update serialization

### Changed 0.3.0

* Removed support for deprecated Python 3.6

## 0.2.0

### Fixed 0.2.0

* Probability when calling mixDamping in quest to simulate PragmaDamping

## 0.1.0

* First release
