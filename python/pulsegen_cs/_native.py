import ctypes
import sys
import typing
from enum import Enum
from pathlib import Path

import numpy as np

from . import models

if sys.platform == "win32":
    lib_path = Path(__file__).parent / "lib" / "Qynit.PulseGen.Aot.dll"
else:
    lib_path = Path(__file__).parent / "lib" / "Qynit.PulseGen.Aot.so"

lib = ctypes.cdll.LoadLibrary(str(lib_path.resolve()))


# enum ErrorCode
# {
#     Success = 0,
#     DeserializeError = 1,
#     GenerateWaveformsError = 2,
#     KeyNotFound = 3,
#     CopyWaveformError = 4,
#     InvalidHandle = 5,
#     InternalError = 6,
# }
class ErrorCode(Enum):
    Success = 0
    DeserializeError = 1
    GenerateWaveformsError = 2
    KeyNotFound = 3
    CopyWaveformError = 4
    InvalidHandle = 5
    InternalError = 6


# int Qynit_PulseGen_Run(char* request, int length, void** out_handle)
Qynit_PulseGen_Run = lib.Qynit_PulseGen_Run
Qynit_PulseGen_Run.argtypes = [
    ctypes.c_char_p,
    ctypes.c_int,
    ctypes.POINTER(ctypes.c_void_p),
]
Qynit_PulseGen_Run.restype = ctypes.c_int


def _run(msg: bytes) -> ctypes.c_void_p:
    handle = ctypes.c_void_p()
    ret = Qynit_PulseGen_Run(msg, len(msg), ctypes.byref(handle))
    if ret != 0:
        err = ErrorCode(ret)
        raise Exception(f"Failed to run PulseGen, error code: {err}")
    return handle


# int Qynit_PulseGen_CopyWaveform(void* handle, char* name, float* i, float* q, int length)
Qynit_PulseGen_CopyWaveform = lib.Qynit_PulseGen_CopyWaveform
Qynit_PulseGen_CopyWaveform.argtypes = [
    ctypes.c_void_p,
    ctypes.c_char_p,
    ctypes.POINTER(ctypes.c_float),
    ctypes.POINTER(ctypes.c_float),
    ctypes.c_int,
]
Qynit_PulseGen_CopyWaveform.restype = ctypes.c_int


def _copy_waveform(
    handle: ctypes.c_void_p, name: str, length: int
) -> typing.Tuple[np.ndarray, np.ndarray]:
    wave_i = np.empty(length, dtype=np.float32)
    wave_q = np.empty(length, dtype=np.float32)
    pstr = name.encode("utf-8")
    ptr_i_float = wave_i.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
    ptr_q_float = wave_q.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
    ret = Qynit_PulseGen_CopyWaveform(handle, pstr, ptr_i_float, ptr_q_float, length)
    if ret != 0:
        err = ErrorCode(ret)
        raise Exception(f"Failed to copy waveform, error code: {err}")
    return wave_i, wave_q


# int Qynit_PulseGen_FreeWaveform(void* handle)
Qynit_PulseGen_FreeWaveform = lib.Qynit_PulseGen_FreeWaveform
Qynit_PulseGen_FreeWaveform.argtypes = [ctypes.c_void_p]
Qynit_PulseGen_FreeWaveform.restype = ctypes.c_int


def _free_waveform(handle: ctypes.c_void_p) -> None:
    ret = Qynit_PulseGen_FreeWaveform(handle)
    if ret != 0:
        err = ErrorCode(ret)
        raise Exception(f"Failed to free waveform, error code: {err}")


def generate_waveforms(
    request: models.Request,
) -> typing.Dict[str, typing.Tuple[np.ndarray, np.ndarray]]:
    msg = request.packb()
    handle = _run(msg)
    try:
        waveforms = {}
        for ch in request.channels:
            waveforms[ch.name] = _copy_waveform(handle, ch.name, ch.length)
        return waveforms
    finally:
        _free_waveform(handle)
