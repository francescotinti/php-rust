#!/usr/bin/env python3
"""rusage188.py <pid> [pid...] — S-188: user/sys CPU of a process TREE via
proc_pid_rusage(RUSAGE_INFO_V4), summed over the pids given.

Why not `ps -o utime`: it has centisecond resolution — one tick is ~0.9 % of
the oracle's whole K=30 block on /, too coarse for a 2 % spread gate.
Output: one line `user_ns sys_ns instructions phys_footprint lifetime_max_phys`
(times converted from mach absolute units with mach_timebase_info).
Exit 2 on any pid that cannot be read (absent must never read as zero).
Generated with AI assistance (Claude Fable 5).
"""
import ctypes
import ctypes.util
import sys

libc = ctypes.CDLL(ctypes.util.find_library("c"), use_errno=True)


class Timebase(ctypes.Structure):
    _fields_ = [("numer", ctypes.c_uint32), ("denom", ctypes.c_uint32)]


_FIELDS = [
    "user_time", "system_time", "pkg_idle_wkups", "interrupt_wkups", "pageins",
    "wired_size", "resident_size", "phys_footprint", "proc_start_abstime",
    "proc_exit_abstime", "child_user_time", "child_system_time",
    "child_pkg_idle_wkups", "child_interrupt_wkups", "child_pageins",
    "child_elapsed_abstime", "diskio_bytesread", "diskio_byteswritten",
    "cpu_time_qos_default", "cpu_time_qos_maintenance", "cpu_time_qos_background",
    "cpu_time_qos_utility", "cpu_time_qos_legacy", "cpu_time_qos_user_initiated",
    "cpu_time_qos_user_interactive", "billed_system_time", "serviced_system_time",
    "logical_writes", "lifetime_max_phys_footprint", "instructions", "cycles",
    "billed_energy", "serviced_energy", "interval_max_phys_footprint", "runnable_time",
]


class RusageInfoV4(ctypes.Structure):
    _fields_ = [("uuid", ctypes.c_uint8 * 16)] + [(n, ctypes.c_uint64) for n in _FIELDS]


def main(argv):
    if len(argv) < 2:
        print("usage: rusage188.py <pid> [pid...]", file=sys.stderr)
        return 2
    assert ctypes.sizeof(RusageInfoV4) == 296, ctypes.sizeof(RusageInfoV4)
    tb = Timebase()
    libc.mach_timebase_info(ctypes.byref(tb))
    libc.proc_pid_rusage.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_void_p]
    libc.proc_pid_rusage.restype = ctypes.c_int

    def ns(ticks):
        return ticks * tb.numer // tb.denom

    tot = [0, 0, 0, 0, 0]
    for arg in argv[1:]:
        ri = RusageInfoV4()
        rc = libc.proc_pid_rusage(int(arg), 4, ctypes.byref(ri))
        if rc != 0:
            print(f"rusage188: pid {arg} unreadable (errno {ctypes.get_errno()})", file=sys.stderr)
            return 2
        tot[0] += ns(ri.user_time)
        tot[1] += ns(ri.system_time)
        tot[2] += ri.instructions
        tot[3] += ri.phys_footprint
        tot[4] += ri.lifetime_max_phys_footprint
    print(*tot)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
