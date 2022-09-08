# Changelog

This changelog track changes to the qoqo-quest project starting at version 0.1.0

## Unpublished

## 0.8.0-beta.1

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

* Updated to qoqo 1.0.0-alpha.2
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
