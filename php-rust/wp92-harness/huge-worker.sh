#!/bin/bash
# huge-worker.sh — «la questione dei worker» (direttiva utente 2026-08-03,
# post-Concilio WP-94): isola il termine malloc_huge PER WORKER dai 20 raw
# m90 COMMITTATI. Zero run nuove, zero strumenti nuovi: pura lettura.
#
# Origine: verbale-7-leijen (Concilio WP-94) nomina malloc_huge come primo
# indiziato della PENDENZA invisibile; A-DL-59 aveva refutato il page-slack.
# Qui il termine è isolato per checkpoint e diviso per W.
#
# GRADO: ADVISORY (raw già consumati; la lane m90 porta i caveat di
# selezione dichiarati in repair90-estimators). VERDICT-grade solo dal
# canale barriera A-DL-57/58 con PT-1.
#
# FORMATO (A-BG66, Concilio WP-94 — emendamento accolto in anticipo):
# ogni token numerico è in cifre ASCII NUDE, nessun separatore di
# migliaia: un separatore rende il token illeggibile al tokenizer del
# corpus e ne fabbrica di spuri (le «schegge-virgola» di dl59-join.out).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
MOUT="$REPO/wp78-harness/measure-out"

python3 - "$MOUT" <<'PY'
import json, re, sys, os

mout = sys.argv[1]
print("huge-worker: termine malloc_huge PER WORKER dai raw m90 committati")
print("grade=ADVISORY  # raw gia consumati; verdict-grade solo dal canale barriera A-DL-57/58 (PT-1)")
print("formato=ascii-nudo  # A-BG66: nessun separatore di migliaia nei token numerici")
print("fonte=wp78-harness/measure-out/m90.slope.w{4,8,12,16}.r{1..5}.a1.memcensus (20 raw)")
print("sorgente-riga=tag=mi_arena_json campo malloc_huge{current,total,peak} + malloc_huge_count")
print("nota-canale: in mimalloc v3 il decremento della statistica huge ESISTE")
print("  (v3/src/free.c mi_theap_stat_decrease(theap, malloc_huge, bpsize) e v3/src/theap.c),")
print("  quindi current==total NON e un contatore monco: e assenza di free.")
print()

rows = []
for W in (4, 8, 12, 16):
    for r in range(1, 6):
        f = f"{mout}/m90.slope.w{W}.r{r}.a1.memcensus"
        if not os.path.exists(f):
            continue
        data = open(f, "rb").read().replace(b"\x00", b"").decode("utf-8", "replace")
        seen = {}
        for line in data.splitlines():
            if "tag=mi_arena_json" not in line:
                continue
            m = re.search(r"ckpt=(\S+)", line)
            ck = m.group(1) if m else "?"
            d = json.loads(line[line.index("json=") + 5:])
            h = d["malloc_huge"]
            seen[ck] = (h["current"], h["total"], h["peak"], d["malloc_huge_count"])
        rows.append((W, r, seen))

CKS = ["peak_inreq_pboot", "peak_inreq_pwork", "peak_inreq", "exit_mi", "exit_collect_mi"]
print("== per-run, per-checkpoint: current / count / current-diviso-W ==")
print("W  run ckpt                 huge_current    count  perW      cnt_perW")
for W, r, seen in rows:
    for ck in CKS:
        if ck not in seen:
            continue
        cur, tot, pk, cnt = seen[ck]
        pw = cur // W if W else 0
        cpw = cnt // W if W else 0
        print(f"{W:<2} r{r}  {ck:<20} {cur:>12} {cnt:>8} {pw:>9} {cpw:>9}")
print()

# invarianti
def uniq(ck, idx):
    return sorted({(v[ck][idx] // W if idx == 0 else v[ck][idx] // W)
                   for W, r, v in rows if ck in v})

pw_work = sorted({seen["peak_inreq_pwork"][0] // W for W, r, seen in rows if "peak_inreq_pwork" in seen})
cpw_work = sorted({seen["peak_inreq_pwork"][3] // W for W, r, seen in rows if "peak_inreq_pwork" in seen})
pboot = sorted({seen["peak_inreq_pboot"][0] for W, r, seen in rows if "peak_inreq_pboot" in seen})
n_runs = len(rows)
print("== INVARIANTI sui 20 raw ==")
print(f"run considerate: {n_runs}")
print(f"per-worker distinti al pwork (byte): {pw_work}")
print(f"blocchi per-worker distinti al pwork: {cpw_work}")
print(f"valori distinti al pboot (byte): {pboot}")

# never-freed: current==total==peak su ogni checkpoint non-boot
bad = []
for W, r, seen in rows:
    for ck in CKS[1:]:
        if ck not in seen:
            continue
        cur, tot, pk, cnt = seen[ck]
        if not (cur == tot == pk):
            bad.append((W, r, ck, cur, tot, pk))
print(f"checkpoint non-boot con current!=total!=peak: {len(bad)}  (0 = mai liberata)")
for b in bad[:5]:
    print("   ", b)

# sopravvivenza all'uscita
surv = []
for W, r, seen in rows:
    if "peak_inreq" in seen and "exit_collect_mi" in seen:
        surv.append(seen["exit_collect_mi"][0] == seen["peak_inreq"][0])
print(f"run in cui il termine al exit_collect_mi EGUAGLIA il picco: {sum(surv)}/{len(surv)}")

if len(pw_work) == 1 and len(cpw_work) == 1 and pboot == [0] and not bad:
    b = pw_work[0]
    c = cpw_work[0]
    print()
    print("== VERDETTO (advisory) ==")
    print(f"per worker: {b} byte in {c} blocchi huge, media {b // c} byte a blocco")
    print("nasce nella fase di LAVORO (zero al boot), non cresce col numero di richieste,")
    print("non viene MAI liberata e sopravvive alla collect finale del processo.")
    print("carico della campagna: hello + pad (NON WordPress) — il termine e quindi")
    print("indipendente dal carico, cioe una riserva FISSA per worker.")
PY
