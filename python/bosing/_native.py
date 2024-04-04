import ctypes
import sys
import typing
from enum import Enum
from pathlib import Path

import numpy as np

if sys.platform == "win32":
    lib_path = Path(__file__).parent / "lib" / "Bosing.Aot.dll"
elif sys.platform == "linux":
    lib_path = Path(__file__).parent / "lib" / "Bosing.Aot.so"
elif sys.platform == "darwin":
    lib_path = Path(__file__).parent / "lib" / "Bosing.Aot.dylib"
else:
    raise Exception(f"Unsupported platform: {sys.platform}")

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


# int Bosing_Run(char* request, int length, void** out_handle)
Bosing_Run = lib.Bosing_Run
Bosing_Run.argtypes = [
    ctypes.c_char_p,
    ctypes.c_int,
    ctypes.POINTER(ctypes.c_void_p),
]
Bosing_Run.restype = ctypes.c_int


def run(msg: bytes) -> ctypes.c_void_p:
    handle = ctypes.c_void_p()
    ret = Bosing_Run(msg, len(msg), ctypes.byref(handle))
    if ret != 0:
        err = ErrorCode(ret)
        raise Exception(f"Failed to run Bosing, error code: {err}")
    return handle


# int Bosing_CopyWaveform(void* handle, char* name, float* i, float* q, int length)
Bosing_CopyWaveform = lib.Bosing_CopyWaveform
Bosing_CopyWaveform.argtypes = [
    ctypes.c_void_p,
    ctypes.c_char_p,
    ctypes.POINTER(ctypes.c_float),
    ctypes.POINTER(ctypes.c_float),
    ctypes.c_int,
]
Bosing_CopyWaveform.restype = ctypes.c_int


def copy_waveform(
    handle: ctypes.c_void_p, name: str, length: int
) -> typing.Tuple[np.ndarray, np.ndarray]:
    wave_i = np.empty(length, dtype=np.float32)
    wave_q = np.empty(length, dtype=np.float32)
    pstr = name.encode("utf-8")
    ptr_i_float = wave_i.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
    ptr_q_float = wave_q.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
    ret = Bosing_CopyWaveform(handle, pstr, ptr_i_float, ptr_q_float, length)
    if ret != 0:
        err = ErrorCode(ret)
        raise Exception(f"Failed to copy waveform, error code: {err}")
    return wave_i, wave_q


# int Bosing_FreeWaveform(void* handle)
Bosing_FreeWaveform = lib.Bosing_FreeWaveform
Bosing_FreeWaveform.argtypes = [ctypes.c_void_p]
Bosing_FreeWaveform.restype = ctypes.c_int


def free_waveform(handle: ctypes.c_void_p) -> None:
    ret = Bosing_FreeWaveform(handle)
    if ret != 0:
        err = ErrorCode(ret)
        raise Exception(f"Failed to free waveform, error code: {err}")
