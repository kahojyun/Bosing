import os
import shutil
import subprocess
import sys
import sysconfig
from typing import Any, Dict, List

from hatchling.builders.hooks.plugin.interface import BuildHookInterface

SRC_DIR = "src/Qynit.PulseGen.Aot"
DST_DIR = "python/pulsegen_cs/lib"

BUILD_TARGET_ARCH = os.environ.get("BUILD_TARGET_ARCH")


def _check_dotnet() -> None:
    try:
        subprocess.run(
            [
                "dotnet",
                "--version",
            ],
            check=True,
            capture_output=True,
        )
    except FileNotFoundError as e:
        msg = "dotnet is not installed"
        raise RuntimeError(msg) from e


def _dotnet_publish(version: str, build_data: Dict[str, Any]) -> None:
    if version == "editable":
        configuration = "Debug"
    else:
        configuration = "Release"
    if BUILD_TARGET_ARCH is None or BUILD_TARGET_ARCH == "":
        rid = ["--use-current-runtime"]
    elif BUILD_TARGET_ARCH in ["x64", "arm64"]:
        rid = ["--arch", BUILD_TARGET_ARCH]
    else:
        raise RuntimeError(f"Unsupported architecture: {BUILD_TARGET_ARCH}")

    try:
        subprocess.run(
            [
                "dotnet",
                "publish",
                SRC_DIR,
                "--output",
                DST_DIR,
                "--configuration",
                configuration,
                "--nologo",
            ]
            + rid,
            check=True,
        )
    except subprocess.CalledProcessError as e:
        msg = "dotnet publish failed"
        raise RuntimeError(msg) from e
    if sys.platform == "win32":
        build_data["artifacts"].append(DST_DIR + "/Qynit.PulseGen.Aot.dll")
    elif sys.platform == "linux":
        build_data["artifacts"].append(DST_DIR + "/Qynit.PulseGen.Aot.so")
    elif sys.platform == "darwin":
        build_data["artifacts"].append(DST_DIR + "/Qynit.PulseGen.Aot.dylib")


def _infer_tag() -> str:
    if BUILD_TARGET_ARCH is None or BUILD_TARGET_ARCH == "":
        plat_tag = sysconfig.get_platform().replace("-", "_").replace(".", "_")
    elif BUILD_TARGET_ARCH == "x64":
        if sys.platform == "win32":
            plat_tag = "win_amd64"
        elif sys.platform == "linux":
            plat_tag = "manylinux_2_28_x86_64"
        elif sys.platform == "darwin":
            plat_tag = "macosx_10_12_x86_64"
        else:
            raise RuntimeError(f"Unsupported platform: {sys.platform}")
    elif BUILD_TARGET_ARCH == "arm64":
        if sys.platform == "win32":
            plat_tag = "win_arm64"
        elif sys.platform == "linux":
            plat_tag = "manylinux_2_28_aarch64"
        elif sys.platform == "darwin":
            plat_tag = "macosx_11_0_arm64"
        else:
            raise RuntimeError(f"Unsupported platform: {sys.platform}")
    else:
        raise RuntimeError(f"Unsupported architecture: {BUILD_TARGET_ARCH}")
    return f"py3-none-{plat_tag}"


class CustomBuildHook(BuildHookInterface):
    def initialize(self, version: str, build_data: Dict[str, Any]) -> None:
        # Skip building the C# library when building the docs
        if (
            os.environ.get("HATCH_ENV_ACTIVE") == "docs"
            or os.environ.get("READTHEDOCS") == "True"
        ):
            return
        if self.target_name == "wheel":
            _check_dotnet()
            _dotnet_publish(version, build_data)
            build_data["pure_python"] = False
            build_data["tag"] = _infer_tag()

    def clean(self, versions: List[str]) -> None:
        shutil.rmtree(DST_DIR, ignore_errors=True)
