<img src="qoqo_Logo_vertical_color.png" alt="qoqo logo" width="300" />

# qoqo-quest

[QuEST](https://github.com/QuEST-Kit/QuEST) simulator backend for the qoqo/roqoqo quantum toolkit by [HQS Quantum Simulations](https://quantumsimulations.de).

This repository contains three components:

* The qoqo_quest backend to simulate quantum programms from the qoqo python interface to roqoqo,
* The roqoqo-quest backend backend to simulate quantum programms from roqoqo directly,
* The quest-sys crate providing rust bindings for the QuEST C library.

## qoqo-quest

[![Documentation Status](https://readthedocs.org/projects/qoqo-quest/badge/?version=latest)](https://qoqo-quest.readthedocs.io/en/latest/?badge=latest)
[![GitHub Workflow Status](https://github.com/HQSquantumsimulations/qoqo-quest/workflows/ci_tests/badge.svg)](https://github.com/HQSquantumsimulations/qoqo-quest/actions)
[![PyPI](https://img.shields.io/pypi/v/qoqo-quest)](https://pypi.org/project/qoqo-quest/)
![PyPI - License](https://img.shields.io/pypi/l/qoqo-quest)
[![PyPI - Format](https://img.shields.io/pypi/format/qoqo-quest)](https://pypi.org/project/qoqo-quest/)

[QuEST](https://github.com/QuEST-Kit/QuEST) based simulator backend for the qoqo quantum toolkit by [HQS Quantum Simulations](https://quantumsimulations.de).

qoqo-quest allows to simulate the execution of qoqo quantum circuits with the help of the QuEST quantum simulator.
Based on QuEST qoqo supports the simulation of error-free and noisy quantum computers.
qoqo-quest is designed to be able to simulate all operations that are part of qoqo.
For usage examples see the examples section of [qoqo](https://github.com/HQSquantumsimulations/qoqo/)
At the moment due to build problems in manylinux containers only python packages for macOS are created automatically and added to PyPi.

A source distribution now exists but requires a Rust install with a rust version > 1.47 and a maturin version { >= 0.12, <0.13 } in order to be built.

## roqoqo-quest

[![Crates.io](https://img.shields.io/crates/v/roqoqo-quest)](https://crates.io/crates/roqoqo-quest)
[![GitHub Workflow Status](https://github.com/HQSquantumsimulations/qoqo-quest/workflows/ci_tests/badge.svg)](https://github.com/HQSquantumsimulations/qoqo-quest/actions)
[![docs.rs](https://img.shields.io/docsrs/roqoqo-quest)](https://docs.rs/roqoqo-quest/)
![Crates.io](https://img.shields.io/crates/l/roqoqo-quest)

[QuEST](https://github.com/QuEST-Kit/QuEST) based simulator backend for the roqoqo quantum toolkit by [HQS Quantum Simulations](https://quantumsimulations.de).

roqoqo-quest allows to simulate the execution of roqoqo quantum circuits directly from rust code with the help of the QuEST quantum simulator.
roqoqo-quest is designed to be able to simulate all operations that are part of roqoqo.
For usage examples see the examples section of [roqoqo](https://github.com/HQSquantumsimulations/qoqo/).

### QuEST build options

QuEST supports distributed computing and the use of GPU computing. The support can be controlled with cmake options. roqoqo-quest is not tested together with these advanced features. If you want to try using these features we recommend cloning this repository and modifying the cmake options in the build.rs rust build script.

## General Notes

This software is still in the beta stage. Functions and documentation are not yet complete and breaking changes can occur.

## Contributing

We welcome contributions to the project. If you want to contribute code, please have a look at CONTRIBUTE.md for our code contribution guidelines.
