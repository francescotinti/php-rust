#!/usr/bin/env python3
# s145-partition.py — partizione del churn in memcpy / inc-dec / nota
# (modello wp145-harness/s145-sonda-b-modello.md; regola di decisione =
# s144-criterio-B.md p.3). Collaudato da s145-partition-golden-test.sh.
# Convenzioni DICHIARATE: prezzi = MEDIA delle 2 repliche, al netto di cal;
# conteggi = r1 (precedente s144, dopo il gate di replica <=1% per chiave).
# Uso: s145-partition.py <dir con price-r*.txt, census-mem-r*.txt, census-zval-r*.txt>
import glob
import re
import sys

out = sys.argv[1]
PKEYS = [
    "cal", "mv_scalar", "mv_str", "mv_arr", "mv_obj",
    "note_scalar", "note_cont_repeat", "pair_zcell", "pair_arr0",
]
CKEYS = [
    "s145.clone_scalar_n", "s145.clone_str_n", "s145.clone_arr_n",
    "s145.clone_obj_n", "s145.clone_ref_n", "s145.clone_rcother_n",
    "s144.rczval_n", "arr.cum_n", "obj.cum_n",
]

fail = 0

# --- prezzi -----------------------------------------------------------------
price_rep = {}
for f in sorted(glob.glob(f"{out}/price-r*.txt")):
    rep = re.search(r"price-(r\d)", f).group(1)
    kv = {}
    for line in open(f, errors="replace"):
        m = re.match(r"s145\.price\.(\w+?)_ns=([0-9.]+)$", line.strip())
        if m:
            kv[m.group(1)] = float(m.group(2))
    price_rep[rep] = kv
reps = [price_rep[r] for r in sorted(price_rep)]
if len(reps) != 2:
    print(f"FAIL: attese 2 repliche prezzi, trovate {len(reps)}")
    sys.exit(2)
# EMENDA DICHIARATA (S-145, dopo t1/t2/t3 agli atti): le chiavi pair_* hanno
# rumore intrinseco run-to-run ~2-4% (stato mimalloc), sopra il gate 1% —
# gate pair emendato a 5% e prezzo pair FIRMATO COME BANDA (min-max delle
# repliche). Le chiavi di PARTIZIONE restano a 1% (osservate <=0,2%).
for k in PKEYS:
    if k not in reps[0] or k not in reps[1]:
        print(f"FAIL: chiave prezzo assente: {k}")
        fail = 1
        continue
    a, b = reps[0][k], reps[1][k]
    m = max(a, b)
    soglia = 0.05 if k.startswith("pair_") else 0.01
    if m and abs(a - b) / m > soglia:
        print(f"DICHIARA scarto>{soglia:.0%} su prezzo {k}: r1={a} r2={b} (criterio p.9: replica)")
        fail = 1
if fail:
    sys.exit(2)
p = {k: (reps[0][k] + reps[1][k]) / 2 for k in PKEYS}
cal = p["cal"]
net = {k: p[k] - cal for k in PKEYS if k != "cal"}

# --- conteggi ---------------------------------------------------------------
cnt_rep = {}
for f in sorted(glob.glob(f"{out}/census-mem-r*.txt")):
    rep = re.search(r"census-mem-(r\d)", f).group(1)
    tot = dict.fromkeys(CKEYS, 0)
    for line in open(f, errors="replace"):
        m = re.search(r"\btag=(\S+)", line)
        if not m or m.group(1) != "exit":
            continue
        kv = dict(re.findall(r"([\w.]+)=(-?\d+)", line))
        for k in CKEYS:
            if k in kv:
                tot[k] += int(kv[k])
    cnt_rep[rep] = tot
for f in sorted(glob.glob(f"{out}/census-zval-r*.txt")):
    rep = re.search(r"census-zval-(r\d)", f).group(1)
    tot = {"gcnote_total": 0, "gcnote_cont": 0}
    for line in open(f, errors="replace"):
        if line.startswith("zvalcensus_s101 "):
            kv = dict(re.findall(r"(\w+)=(\d+)", line))
            tot["gcnote_total"] += int(kv.get("gcnote_total", 0))
        elif line.startswith("zvalcensus_s145 "):
            kv = dict(re.findall(r"(\w+)=(\d+)", line))
            tot["gcnote_cont"] += int(kv.get("gcnote_cont", 0))
    cnt_rep[rep] = {**cnt_rep.get(rep, dict.fromkeys(CKEYS, 0)), **tot}
crs = [cnt_rep[r] for r in sorted(cnt_rep)]
if len(crs) != 2:
    print(f"FAIL: attese 2 repliche conteggi, trovate {len(crs)}")
    sys.exit(2)
exact = True
for k in CKEYS + ["gcnote_total", "gcnote_cont"]:
    a, b = crs[0].get(k, 0), crs[1].get(k, 0)
    if a != b:
        exact = False
        m = max(a, b)
        if m and abs(a - b) / m > 0.01:
            print(f"DICHIARA scarto>1% su conteggio {k}: r1={a} r2={b} (criterio p.9: replica)")
            fail = 1
if fail:
    sys.exit(2)
c = crs[0]

# --- partizione -------------------------------------------------------------
n_scalar = c["s145.clone_scalar_n"]
n_str = c["s145.clone_str_n"]
n_arr = c["s145.clone_arr_n"]
n_rc = c["s145.clone_obj_n"] + c["s145.clone_ref_n"] + c["s145.clone_rcother_n"]
n_tot = n_scalar + n_str + n_arr + n_rc
inc_str = net["mv_str"] - net["mv_scalar"]
inc_arr = net["mv_arr"] - net["mv_scalar"]
inc_obj = net["mv_obj"] - net["mv_scalar"]
for name, v in [("inc_str", inc_str), ("inc_arr", inc_arr), ("inc_obj", inc_obj)]:
    if v <= 0:
        print(f"DICHIARA anomalia strumento: {name}={v:.3f} <= 0 (classe heap non piu' cara dello scalare)")
        fail = 1
memcpy = net["mv_scalar"] * n_tot
incdec = inc_str * n_str + inc_arr * n_arr + inc_obj * n_rc
n_note_cont = c["gcnote_cont"]
n_note_plain = c["gcnote_total"] - n_note_cont
nota = net["note_scalar"] * n_note_plain + net["note_cont_repeat"] * n_note_cont
den = memcpy + incdec + nota
if den <= 0:
    print("FAIL: denominatore <= 0")
    sys.exit(2)
q_mem, q_inc, q_nota = (100 * x / den for x in (memcpy, incdec, nota))

print(f"prezzi_netti_ns: mv_scalar={net['mv_scalar']:.3f} mv_str={net['mv_str']:.3f} "
      f"mv_arr={net['mv_arr']:.3f} mv_obj={net['mv_obj']:.3f} "
      f"note_scalar={net['note_scalar']:.3f} note_cont_repeat={net['note_cont_repeat']:.3f} (cal={cal:.3f})")
print(f"conteggi (r1==r2 {'ESATTO' if exact else 'entro 1%, DICHIARATO'}): "
      f"clone scalar={n_scalar} str={n_str} arr={n_arr} rc(obj+ref+other)={n_rc} tot={n_tot} "
      f"gcnote_total={c['gcnote_total']} gcnote_cont={n_note_cont}")
print(f"componenti_s: memcpy={memcpy/1e9:.2f} incdec={incdec/1e9:.2f} nota={nota/1e9:.2f} "
      f"(somma={den/1e9:.2f}; denominatore = somma delle tre, dal sorgente della sonda)")
print(f"QUOTE: memcpy={q_mem:.1f}% inc-dec={q_inc:.1f}% nota={q_nota:.1f}%")
pz = sorted([reps[0]["pair_zcell"] - cal, reps[1]["pair_zcell"] - cal])
pa = sorted([reps[0]["pair_arr0"] - cal, reps[1]["pair_arr0"] - cal])
print(f"PREZZI PAIR FIRMATI A BANDA (fuori partizione; emenda gate 5%): "
      f"pair_zcell={pz[0]:.2f}-{pz[1]:.2f} ns pair_arr0={pa[0]:.2f}-{pa[1]:.2f} ns "
      f"(contesto nascite: rczval={c['s144.rczval_n']} arr={c['arr.cum_n']} obj={c['obj.cum_n']})")
print("REGOLA p.3 (s144-criterio-B.md, pre-registrata):")
if q_inc + q_nota >= 60:
    ordine = "B1->B2" if q_inc >= q_nota else "B2->B1"
    print(f"ESITO REGOLA: inc-dec+nota = {q_inc+q_nota:.1f}% >= 60% => fette {ordine} "
          f"(ordine interno dal contributo: inc-dec {q_inc:.1f}% vs nota {q_nota:.1f}%)")
elif q_mem >= 60:
    print(f"ESITO REGOLA: memcpy = {q_mem:.1f}% >= 60% => B1/B2 NON si aprono, filone conteggi (B3) TORNA AL CONCILIO (KS-B4)")
else:
    prima = max([("memcpy", q_mem), ("inc-dec", q_inc), ("nota", q_nota)], key=lambda t: t[1])
    print(f"ESITO REGOLA: nessuno >=60% => ordine per contributo assoluto, fetta maggiore prima ({prima[0]} {prima[1]:.1f}%)")
sys.exit(1 if fail else 0)
