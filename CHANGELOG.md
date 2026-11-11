# Changelog

All notable changes to this project will be documented in this file.

The project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 3.0.0 (2026-11-11)

### Breaking Changes

- require Python 3.11 and refresh the project lockfile (#431)

#### Require Python 3.11 or newer

Drop Python 3.10 support and build stable-ABI wheels for Python 3.11 and newer. Update the type-checking target and project lockfile to match the new minimum Python version.

### Features

#### Publish regular CPython ABI3 wheels only

Keep one stable-ABI wheel for CPython 3.11 and newer on each supported platform and architecture, plus the source distribution. Stop publishing Python 3.14 free-threaded wheels; free-threaded environments require a source build and are outside the tested support matrix.

#### Align direct dependency floors with Python 3.11

Require NumPy 1.23.5 and Matplotlib 3.6.0 or newer, and declare typing-extensions 4.15.0 for the shipped type annotations. Test the declared minimum versions in CI.

#### Require Rust 1.92 for source builds

Align the declared minimum Rust version with the locked dependencies and check it in CI. Prebuilt Python wheels are unaffected.

### Fixes

- align minimum Rust version with dependencies (#429)
- declare and test direct dependency floors (#433)

## 2.8.0 (2026-07-28)

### Features

- support schedule-based automatic waveform lengths (#367)

#### Support schedule-based automatic waveform lengths

Allow `Channel.length` to default to `None` and derive waveform lengths from the logical schedule
duration. Sampling bounds errors now report the required sample range and available waveform
length.

## 2.7.0 (2026-01-15)

### Notes

- This release establishes the changelog baseline before knope-managed releases.
