import { DotNet } from "@microsoft/dotnet-js-interop";

const darkModePreference = window.matchMedia("(prefers-color-scheme: dark)");

export function isSystemDarkMode() {
  return darkModePreference.matches;
}

export function init(objRef: DotNet.DotNetObject) {
  darkModePreference.addEventListener("change", (e) => {
    objRef.invokeMethodAsync("OnDarkModeChanged", e.matches);
  });
}
