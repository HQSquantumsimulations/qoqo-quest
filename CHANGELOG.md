# Changelog

This changelog track changes to the qoqo-quest project starting at version 0.1.0

## Unpublished

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

## 0.7.2

* Updated dependencies

## 0.7.1

* Fixed: Bug in calculating result for PragmaGetPauliProduct

## 0.7.4

* Updated to qoqo 1.0.0

## 0.7.3

* Fixed general noise pragma simulation

## 0.7.2

* Updated dependencies

## 0.7.1

* Fixed: Bug in calculating result for PragmaGetPauliProduct



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
