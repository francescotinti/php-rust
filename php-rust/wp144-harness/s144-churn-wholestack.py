#!/usr/bin/env python3
# s144-churn-wholestack.py — az.2 revisione S-144: MAGGIORANTE del churn
# oracle per INTERO STACK (un campione conta se un qualunque frame del suo
# stack matcha la famiglia churn), sui raw sample esistenti (nessuna run
# nuova). Criterio pre-registrato dal revisore: maggiorante < 5,15pp
# (= 50% x churn_zval phpr 10,3%) => canale chiuso per misura.
# Denominatore = campioni ATTIVI (esclusi thread parcheggiati, emenda v2).
# Uso: s144-churn-wholestack.py <sample-file> ...
import re
import sys

CHURN = re.compile(r"zval_ptr_dtor|rc_dtor|zend_assign_to_variable|zval_copy")
IDLE = re.compile(r"__workq_kernreturn|__psynch|kevent|__semwait")
NODE = re.compile(r"^([ +!:|]*)(\d+)\s+(.*)$")

for path in sys.argv[1:]:
    total = 0        # campioni totali (somma nodi Thread di primo livello)
    idle = 0         # campioni il cui stack termina/passa in un parcheggio
    churn = 0        # campioni con >=1 frame churn (nodi churn MASSIMALI)
    stack = []       # (depth, is_churn, is_idle) degli antenati
    in_graph = False
    for line in open(path, errors="replace"):
        if line.startswith("Call graph"):
            in_graph = True
            continue
        if in_graph and (line.startswith("Total number") or line.startswith("Sort by")):
            break
        if not in_graph:
            continue
        m = NODE.match(line.rstrip("\n"))
        if not m:
            continue
        depth = len(m.group(1))
        n = int(m.group(2))
        sym = m.group(3)
        while stack and stack[-1][0] >= depth:
            stack.pop()
        anc_churn = any(c for _, c, _ in stack)
        anc_idle = any(i for _, _, i in stack)
        is_churn = bool(CHURN.search(sym))
        is_idle = bool(IDLE.search(sym))
        if depth == 4:  # nodi Thread di primo livello ("    NNN Thread_x")
            total += n
        if is_idle and not anc_idle:
            idle += n
        if is_churn and not anc_churn and not anc_idle:
            churn += n
        stack.append((depth, is_churn or anc_churn, is_idle or anc_idle))
    att = total - idle
    if att <= 0:
        print(f"{path}: PARSE VUOTO (total={total} idle={idle})")
        continue
    print(f"{path}: total={total} idle={idle} attivi={att} churn_wholestack={churn} => {100*churn/att:.2f}pp (soglia 5,15pp)")
