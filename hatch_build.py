import shutil
import subprocess
import sys
from typing import Any, Dict, List

from hatchling.builders.hooks.plugin.interface import BuildHookInterface

SERVER_PROJECT_DIR = "src/Qynit.PulseGen.Server"
if sys.platform == "win32":
    NPM = "npm.cmd"
else:
    NPM = "npm"

class CustomBuildHook(BuildHookInterface):
    def initialize(self, version: str, build_data: Dict[str, Any]) -> None:
        if self.target_name == "wheel":
            self._check_dotnet()
            self._check_npm()
            self._npm_ci()
            self._npm_build()
            self._dotnet_publish(version)

    def clean(self, versions: List[str]) -> None:
        shutil.rmtree("artifacts", ignore_errors=True)

    def _check_dotnet(self) -> None:
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
        
    def _check_npm(self) -> None:
        try:
            subprocess.run(
                [
                    NPM,
                    "--version",
                ],
                check=True,
                capture_output=True,
            )
        except FileNotFoundError as e:
            msg = "npm is not installed"
            raise RuntimeError(msg) from e
            
        
    def _npm_ci(self) -> None:
        try:
            subprocess.run(
                [
                    NPM,
                    "ci",
                ],
                check=True,
                cwd=SERVER_PROJECT_DIR,
            )
        except subprocess.CalledProcessError as e:
            msg = "npm ci failed"
            raise RuntimeError(msg) from e
        
    def _npm_build(self) -> None:
        try:
            subprocess.run(
                [
                    NPM,
                    "run",
                    "build",
                ],
                check=True,
                cwd=SERVER_PROJECT_DIR,
            )
        except subprocess.CalledProcessError as e:
            msg = "npm build failed"
            raise RuntimeError(msg) from e

    def _dotnet_publish(self, version: str) -> None:
        if version == "editable":
            configuration = "Debug"
        else:
            configuration = "Release"
        try:
            subprocess.run(
                [
                    "dotnet",
                    "publish",
                    "--configuration",
                    configuration,
                    "--nologo",
                ],
                check=True,
            )
        except subprocess.CalledProcessError as e:
            msg = "dotnet publish failed"
            raise RuntimeError(msg) from e

