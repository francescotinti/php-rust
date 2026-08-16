#!/usr/bin/env python3
# s147-parse.py — aggregatore del CENSUS UNICO ORM (criterio
# s147-criterio-census.md p.5; committato + golden PRIMA del run).
# Input: <census-out> <prezzi-verdetto-s145> <rimisura-verdetto-s147>.
# Emette su stdout la sezione-analisi del verdetto: per-sito, digrammi,
# ponte, F1 take-per-tipo, bande in SECONDI (conteggi x prezzi sonda) e il
# KILL ARITMETICO KS-146-1 (soglia = 0,7% della coppia ORM @ s145, derivata
# MECCANICAMENTE dal verdetto p.1a). SOLO conteggi: mai cifre di tempo dal
# binario census.
import re
import sys

CLASSES = ["scalar", "str", "arr", "obj", "ref", "rcother"]
# Ponte (criterio p.1-ii): lista CHIUSA dei siti slot-load; ogni altro sito
# 'Load*' visto nel dump viene NOMINATO a verbale, mai sommato in silenzio.
BRIDGE = {"LoadSlot", "LoadVar"}
# Prezzo per classe = mv_<k>_ns - cal_ns (convenzione partizione s145);
# ref/rcother al prezzo obj (dichiarato nel modello s145-sonda-b-modello.md).
PRICE_KEY = {"scalar": "mv_scalar_ns", "str": "mv_str_ns", "arr": "mv_arr_ns",
             "obj": "mv_obj_ns", "ref": "mv_obj_ns", "rcother": "mv_obj_ns"}


def parse_mem(path):
    mv = {}      # name -> [6 classi + tot]
    dg = {}      # (prev, cur) -> n
    s145 = dict.fromkeys(CLASSES, 0)
    for line in open(path):
        m = re.match(
            r"s147mv pid=\d+ site=\d+ name=(\S+) scalar=(\d+) str=(\d+) "
            r"arr=(\d+) obj=(\d+) ref=(\d+) rcother=(\d+) tot=(\d+)$", line)
        if m:
            row = mv.setdefault(m.group(1), [0] * 7)
            for k in range(7):
                row[k] += int(m.group(2 + k))
            continue
        m = re.match(r"s147dg pid=\d+ prev=(\S+) cur=(\S+) n=(\d+)$", line)
        if m:
            key = (m.group(1), m.group(2))
            dg[key] = dg.get(key, 0) + int(m.group(3))
            continue
        if " tag=exit " in line:
            for k in CLASSES:
                m = re.search(r" s145\.clone_%s_n=(\d+)" % k, line)
                if m:
                    s145[k] += int(m.group(1))
    return mv, dg, s145


def parse_zval(path):
    tot = {}
    for line in open(path):
        if line.startswith(("zvalcensus ", "zvalcensus_s147 ", "zvalcensus_s145 ")):
            for k, v in re.findall(r"(\w+)=(\d+)", line):
                tot[k] = tot.get(k, 0) + int(v)
    return tot


def parse_prices(path):
    reps = []
    for line in open(path):
        m = re.match(r"r\d+: OK (.*)", line.strip())
        if m:
            reps.append(dict(
                (k, float(v))
                for k, v in re.findall(r"s145\.price\.(\w+)=([\d.]+)", m.group(1))))
    if len(reps) < 2:
        sys.exit("PREZZI: attese >=2 repliche r*: OK")
    mean = {k: (reps[0][k] + reps[1][k]) / 2 for k in reps[0]}
    cal = mean["cal_ns"]
    return {c: mean[PRICE_KEY[c]] - cal for c in CLASSES}


def parse_soglia(path):
    users, floor = [], None
    for line in open(path):
        m = re.search(r"orm o=([\d.]+) p=([\d.]+)", line)
        if m:
            floor = float(m.group(2))
        m = re.match(r"orm_leg\d+: .*phpr_user=([\d.]+)", line)
        if m:
            users.append(float(m.group(1)))
    if floor is None or not users:
        sys.exit("SOGLIA: verdetto rimisura ORM illeggibile")
    net = sum(users) / len(users) - floor
    return 0.007 * net, net


def sec(count, price_ns):
    return count * price_ns * 1e-9


def main():
    out_dir, prezzi_path, rimisura_path = sys.argv[1], sys.argv[2], sys.argv[3]
    price = parse_prices(prezzi_path)
    soglia_s, orm_net = parse_soglia(rimisura_path)
    reps = []
    for r in (1, 2):
        mv, dg, s145 = parse_mem(f"{out_dir}/census-mem-r{r}.txt")
        zv = parse_zval(f"{out_dir}/census-zval-r{r}.txt")
        reps.append((mv, dg, s145, zv))

    print("--- s147-parse (prezzi: mv-cal ns/coppia %s; soglia KS-146-1 = 0,7%% x orm_net %.2f s = %.3f s) ---"
          % (" ".join("%s=%.3f" % (c, price[c]) for c in CLASSES), orm_net, soglia_s))

    # coerenza interna per replica: somma s147mv == somma s145.clone_* (ESATTA)
    for i, (mv, _, s145, _) in enumerate(reps, 1):
        tot147 = sum(row[6] for row in mv.values())
        tot145 = sum(s145.values())
        flag = "OK" if tot147 == tot145 else "INCOERENTE(KS-146-2)"
        print(f"r{i}_coerenza: s147mv_tot={tot147} s145_clone_tot={tot145} {flag}")

    # replica r1==r2 <=1% sulle chiavi aggregate
    def agg(rep):
        mv, dg, s145, zv = rep
        a = {"tot_mv": sum(r[6] for r in mv.values())}
        for k, c in enumerate(CLASSES):
            a["mv_" + c] = sum(r[k] for r in mv.values())
        a["ponte"] = sum(r[6] for n, r in mv.items() if n in BRIDGE)
        for k in ("slot_reads", "slot_reads_rc", "would_take_safe",
                  "would_take_safe_rc", "would_take_safe_str",
                  "would_take_safe_ref", "would_take_safe_arr",
                  "would_take_safe_obj"):
            a[k] = rep[3].get(k, 0)
        return a
    a1, a2 = agg(reps[0]), agg(reps[1])
    worst = ("", 0.0)
    for k in a1:
        hi, lo = max(a1[k], a2[k]), min(a1[k], a2[k])
        d = 0.0 if hi == 0 else (hi - lo) / hi
        if d > worst[1]:
            worst = (k, d)
        flag = "" if d <= 0.01 else "  FUORI-1%(KS-146-2)"
        print(f"replica {k}: r1={a1[k]} r2={a2[k]} delta={d * 100:.3f}%{flag}")
    print(f"replica_worst: {worst[0]} {worst[1] * 100:.3f}%")

    # media delle repliche per le bande
    mvm = {}
    for name in set(reps[0][0]) | set(reps[1][0]):
        r1 = reps[0][0].get(name, [0] * 7)
        r2 = reps[1][0].get(name, [0] * 7)
        mvm[name] = [(x + y) / 2 for x, y in zip(r1, r2)]

    def site_sec(row):
        return sum(sec(row[k], price[c]) for k, c in enumerate(CLASSES))

    print("--- ranking siti (media r1/r2; movimenti e SECONDI = conteggio x prezzo classe) ---")
    ranked = sorted(mvm.items(), key=lambda kv: -kv[1][6])
    tot_mv = sum(r[6] for r in mvm.values())
    tot_s = sum(site_sec(r) for r in mvm.values())
    for name, row in ranked[:25]:
        print("sito %-22s mv=%12.0f (%5.2f%%) sec=%7.3f  [%s]"
              % (name, row[6], 100 * row[6] / tot_mv if tot_mv else 0, site_sec(row),
                 " ".join("%s=%.0f" % (c, row[k]) for k, c in enumerate(CLASSES) if row[k])))
    print(f"tot_mv={tot_mv:.0f} tot_sec_modellati={tot_s:.3f}")

    # ponte (criterio p.1-ii): convenzioni affiancate, MAI mescolate
    ponte_mv = sum(mvm[n][6] for n in BRIDGE if n in mvm)
    ponte_s = sum(site_sec(mvm[n]) for n in BRIDGE if n in mvm)
    altri_load = sorted(n for n in mvm if "Load" in n and n not in BRIDGE)
    zv_sr = (a1["slot_reads"] + a2["slot_reads"]) / 2
    zv_srrc = (a1["slot_reads_rc"] + a2["slot_reads_rc"]) / 2
    print("--- ponte slot-load <-> movimenti ---")
    print(f"ponte_mv (s147mv, lista {sorted(BRIDGE)}): {ponte_mv:.0f} = {100 * ponte_mv / tot_mv if tot_mv else 0:.2f}% del tot_mv; sec={ponte_s:.3f}")
    print(f"slot_reads (convenzione census): {zv_sr:.0f}  slot_reads_rc={zv_srrc:.0f}")
    print(f"altri siti Load* NON nel ponte (nominati, criterio p.1-ii): {altri_load if altri_load else 'nessuno'}")

    # digrammi top-20 (media)
    dgm = {}
    for key in set(reps[0][1]) | set(reps[1][1]):
        dgm[key] = (reps[0][1].get(key, 0) + reps[1][1].get(key, 0)) / 2
    print("--- digrammi top-20 (prev->cur per movimento) ---")
    for (p_, c_), n in sorted(dgm.items(), key=lambda kv: -kv[1])[:20]:
        print(f"dg {p_}->{c_}: {n:.0f}")

    # F1 take-per-tipo (media) in secondi: prezzo pieno e solo-incdec
    print("--- F1 take-per-tipo (would_take_safe_*, media r1/r2) ---")
    incdec = {c: price[c] - price["scalar"] for c in CLASSES}
    for t, c in (("str", "str"), ("arr", "arr"), ("obj", "obj"), ("ref", "obj")):
        n = (a1[f"would_take_safe_{t}"] + a2[f"would_take_safe_{t}"]) / 2
        print(f"take_{t}: n={n:.0f} pieno={sec(n, price[c]):.3f}s incdec={sec(n, incdec[c]):.3f}s")
    wts = (a1["would_take_safe"] + a2["would_take_safe"]) / 2
    wtsrc = (a1["would_take_safe_rc"] + a2["would_take_safe_rc"]) / 2
    print(f"take_safe_tot: n={wts:.0f} (rc={wtsrc:.0f})")

    # KILL ARITMETICO (criterio p.6)
    ratio = ponte_s / soglia_s if soglia_s else 0.0
    if ratio < 1:
        verdict = "ZERO codice sul bersaglio (banda < 1x soglia)"
    elif ratio < 2:
        verdict = "SOLO fette micro-judged (1x-2x)"
    else:
        verdict = "scommessa suite AMMESSA (>=2x; KS-B1 da ri-registrare)"
    print("--- KILL ARITMETICO KS-146-1 ---")
    print(f"banda_borrow_first={ponte_s:.3f}s soglia={soglia_s:.3f}s rapporto={ratio:.2f} => {verdict}")


if __name__ == "__main__":
    main()
