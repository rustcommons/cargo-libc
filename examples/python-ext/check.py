"""Import the extension and exercise it: python check.py EXPECTED_RUNTIME_GLIBC"""

import os
import sys
import tempfile

import glibc_python_ext as ext

runtime = ext.runtime_glibc()
print(f"python extension: running on glibc {runtime}")
if len(sys.argv) > 1 and runtime != sys.argv[1]:
    sys.exit(f"expected glibc {sys.argv[1]}, got {runtime}")
with tempfile.TemporaryDirectory() as tmp:
    total = ext.work(1000, os.path.join(tmp, "sum.txt"))
assert total == 500500, total
print(f"python extension: work = {total}")
