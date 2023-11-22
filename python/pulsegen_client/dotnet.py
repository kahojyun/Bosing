# pylint: disable=all
# pyright: reportMissingImports=false
import sys
import typing as _typing
import numpy as _np
import pulsegen_client.models as _models
from pathlib import Path

DOTNET_LIB_PATH = Path(__file__).parent / "lib"
RUNTIME_CONFIG_PATH = DOTNET_LIB_PATH / "Qynit.PulseGen.Server.runtimeconfig.json"
sys.path.append(str(DOTNET_LIB_PATH))

from pythonnet import load

load(
    "coreclr",
    runtime_config=str(RUNTIME_CONFIG_PATH),
)

import clr

clr.AddReference("Qynit.PulseGen.Server")
from Qynit.PulseGen.Server import PythonApi


def generate_waveforms(
    request: _models.Request,
) -> _typing.Dict[str, _typing.Tuple[_np.ndarray, _typing.Optional[_np.ndarray]]]:
    msg = request.packb()
    waveforms = {}
    for channel in request.channels:
        length = channel.length
        waveforms[channel.name] = (
            _np.empty(length, dtype=_np.float32),
            _np.empty(length, dtype=_np.float32),
        )
    PythonApi.Run(msg, waveforms)
    return waveforms

def start_server():
    PythonApi.StartServer()
