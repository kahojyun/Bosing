import os
import shutil
import subprocess
from typing import Any, Dict, List

from hatchling.builders.hooks.plugin.interface import BuildHookInterface

SRC_DIR = "src/Qynit.PulseGen.Aot"
DST_DIR = "python/pulsegen_cs/lib"


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


def _dotnet_publish(version: str) -> None:
    if version == "editable":
        configuration = "Debug"
        ci = "false"
    else:
        configuration = "Release"
        ci = "true"
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
                "--use-current-runtime",
                f"-p:ContinuousIntegrationBuild={ci}",
            ],
            check=True,
        )
    except subprocess.CalledProcessError as e:
        msg = "dotnet publish failed"
        raise RuntimeError(msg) from e


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
            _dotnet_publish(version)

    def clean(self, versions: List[str]) -> None:
        shutil.rmtree(DST_DIR, ignore_errors=True)
