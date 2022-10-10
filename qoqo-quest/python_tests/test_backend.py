"""Test qoqo mocked backend"""
# Copyright © 2019-2021 HQS Quantum Simulations GmbH. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
# in compliance with the License. You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software distributed under the License
# is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
# or implied. See the License for the specific language governing permissions and limitations under
# the License.
import pytest
import sys
import numpy as np
import numpy.testing as npt
from qoqo import operations as ops
from qoqo import Circuit
from qoqo_quest import Backend
from typing import List


@pytest.mark.parametrize(
    'measurement',
    [(ops.MeasureQubit(qubit=0, readout='ro', readout_index=0), int, 0),
     (ops.PragmaRepeatedMeasurement(readout='ro', number_measurements=10), int, 0),
     (ops.PragmaGetPauliProduct(qubit_paulis={0: 1, 1: 2}, readout='ro', circuit=Circuit()), float, 1),
     (ops.PragmaGetOccupationProbability(readout='ro', circuit=Circuit()), float, 1),
     (ops.PragmaGetStateVector(readout='ro', circuit=Circuit()), complex, 2),
     (ops.PragmaGetDensityMatrix(readout='ro', circuit=Circuit()), complex, 2),
    ])
def test_mocked_backend(measurement):
    """Test mocked backend"""
    circuit = Circuit()
    circuit += ops.DefinitionFloat(name='ro', length=2, is_output=True)
    circuit += ops.DefinitionComplex(name='ro', length=4, is_output=True)
    circuit += ops.DefinitionBit(name='ro', length=2, is_output=True)
    circuit += ops.PauliX(qubit=0)
    circuit += measurement[0]

    backend = Backend(2)

    results = backend.run_circuit(circuit=circuit)
    index = measurement[2]
    print(results)
    print(index)
    results = results[measurement[2]]['ro'][0]
    if isinstance(results[0], List):
        assert isinstance(results[0][0], measurement[1])
    else:
        assert isinstance(results[0], measurement[1])


if __name__ == '__main__':
    pytest.main(sys.argv)
