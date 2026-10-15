"""Finite child-side observations; never a process/group authority source."""
import errno
import os
import sys


def number(value, maximum):
    if not value or len(value) > 39 or not value.isascii() or not value.isdecimal():
        raise ValueError("invalid unsigned integer")
    parsed = int(value)
    if parsed > maximum:
        raise ValueError("integer out of range")
    return parsed


if len(sys.argv) != 14:
    raise ValueError("expected report path and four fd identities")
report = sys.argv[1]
if not report or len(report) > 4096 or "\x00" in report:
    raise ValueError("invalid report path")
witnesses = []
for offset in range(2, 14, 3):
    fd = number(sys.argv[offset], 9)
    if fd < 3 or any(existing[0] == fd for existing in witnesses):
        raise ValueError("invalid or duplicate fd slot")
    witnesses.append((fd, number(sys.argv[offset + 1], 2**128 - 1),
                      number(sys.argv[offset + 2], 2**128 - 1)))

# Observe every slot before opening the report: its fd cannot become a witness.
rows = []
for fd, device, inode in witnesses:
    try:
        stat = os.fstat(fd)
        state = "inherited" if (stat.st_dev, stat.st_ino) == (device, inode) else "other"
        rows.append((fd, state, 0))
    except OSError as error:
        rows.append((fd, "closed" if error.errno == errno.EBADF else "error",
                     error.errno or 0))
with open(report, "x", encoding="ascii") as output:
    output.write(f"P4APIv1\t{os.getpgrp()}\n")
    for row in rows:
        output.write("\t".join(map(str, row)) + "\n")
