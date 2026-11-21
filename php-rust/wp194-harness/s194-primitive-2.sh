#!/bin/bash
# second pass of the primitive probe: literal-bound loops and the two repaired object fixtures
H="$(cd "$(dirname "$0")" && pwd)"
python3 "$H/s194-primitive.py" s194-primitive-2 noop lit_noop lit_assign_const lit_add_assign lit_add new_obj new_ctor > "$H/s194-primitive-2.log" 2>&1
echo $? > "$H/s194-primitive-2.rc"
