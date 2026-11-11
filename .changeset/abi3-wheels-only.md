---
default: minor
---

# Publish regular CPython ABI3 wheels only

Keep one stable-ABI wheel for CPython 3.11 and newer on each supported platform and architecture, plus the source distribution. Stop publishing Python 3.14 free-threaded wheels; free-threaded environments require a source build and are outside the tested support matrix.
