import resource
import subprocess
import sys

subprocess.run(sys.argv[1:], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
ru = resource.getrusage(resource.RUSAGE_CHILDREN)
print(f"{ru.ru_utime:.6f}")
