#!/usr/bin/env python3
"""gen-fetta1.py — genera le due COPIE DICHIARATE della FETTA 1 S-189 (criterio s189-criterio-guardia-ax.md):
  s189-guardia-ax.sh   da ../wp188-harness/s188-throughput.sh   (manifest s189-guardia-ax-copia.diff)
  s189-lancio.sh       da ../wp188-harness/s188-lancio.sh       (manifest s189-lancio-copia.diff)
Regola shell-harness: ogni sostituzione è asserita UNICA (o col conteggio dichiarato); il giudice python
della guardia è SOSTITUITO per intero (AXnew/AXold per cella e per ciclo, bande pre-registrate, p99 e
cpu/req companion) ed è dichiarato nel manifest. Generato con assistenza AI (Claude Fable 5).
"""
import os
import stat
import subprocess

H = os.path.dirname(os.path.abspath(__file__))
W188 = os.path.join(H, "..", "wp188-harness")


def sub(s, old, new, count=1):
    n = s.count(old)
    assert n == count, f"sostituzione non unica ({n} != {count}): {old[:70]!r}"
    return s.replace(old, new)


def emit(path, text, src):
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)
    os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    man = path[:-3] + "-copia.diff"
    with open(man, "w", encoding="utf-8") as f:
        subprocess.run(["diff", "-u", src, path], stdout=f, check=False)
    print("scritto", os.path.basename(path), "+ manifest", os.path.basename(man))


# ---------------------------------------------------------------- guardia AX
SRC = os.path.join(W188, "s188-throughput.sh")
s = open(SRC, encoding="utf-8").read()
HEAD_OLD = """# s188-throughput.sh <tentativo> — S-188 Tappa 2: throughput WordPress, server residente
# (`php-server --axum --workers 4`, pin s188) vs oracle `php -S` a 4 worker, carico `ab` chiuso
# per livello C ∈ {1,4,8}; criterio s188-criterio-throughput.md. Struttura e igiene ereditate da
# s188-wp-server-pair.sh (quiescenza per boot, gate richieste, rusage dell'albero, GHz da time -l).
# Variabili: C_LEVELS ("1 4 8") N (150) WARM (20) R (3) PIN_ATTESO (hash server pin s188).
# rc SOLO da pair-out/tp188-<t>.done. Generato con assistenza AI (Claude Fable 5).
"""
HEAD_NEW = """# s189-guardia-ax.sh <tentativo> — S-189 FETTA 1: guardia di NON-REGRESSIONE del throughput WordPress
# fra `php-server --axum --workers 4` pin s188 (AXold, stash) e pin s189 (AXnew, canonica), interallacciati
# nella STESSA finestra, carico `ab` chiuso per livello C ∈ {1,4,8}; criterio s189-criterio-guardia-ax.md.
# COPIA DICHIARATA di ../wp188-harness/s188-throughput.sh (manifest s189-guardia-ax-copia.diff, generatore
# gen-fetta1.py) coi SOLI adattamenti: nomi s189/ga189, bracci AXold/AXnew (due binari, due hash), warm-up
# a -c W (unit cache di ogni worker calda), lock TOKEN s189, giudice SOSTITUITO (AXnew/AXold per cella e
# per ciclo, bande [0,97;1,05] a C=4/8 e [0,95;1,05] a C=1, p99 e cpu/req companion). Il resto INVARIATO.
# Variabili: C_LEVELS N WARM R PIN_ATTESO (hash s189) PIN_OLD (hash s188) SRVOLD (stash). rc SOLO da
# pair-out/ga189-<t>.done. Generato con assistenza AI (Claude Fable 5).
"""
s = sub(s, HEAD_OLD, HEAD_NEW)
s = sub(s, 'H="$REPO/wp188-harness"', 'H="$REPO/wp189-harness"')
s = sub(s, 'RU="$H/rusage188.py"', 'RU="$REPO/wp188-harness/rusage188.py"')
s = sub(s, 'VERD="$H/s188-throughput-verdetto-$T.out"', 'VERD="$H/s189-guardia-ax-verdetto-$T.out"')
s = sub(s, "tp188-$T", "ga189-$T", count=3)
s = sub(s, 'ARMS="${ARMS:-AX O4}"', 'ARMS="${ARMS:-AXold AXnew}"')
s = sub(s, 'SRVBIN="$HOME/Claude/php-rust-output/release/php-server"',
        'SRVNEW="$HOME/Claude/php-rust-output/release/php-server"\n'
        'SRVOLD="${SRVOLD:-/Volumes/Extreme Pro/Claude/phpr-old-target/release/php-server-s188}"')
s = sub(s, 'PIN_ATTESO="${PIN_ATTESO:?PIN_ATTESO (hash server pin s188) obbligatorio}"',
        'PIN_ATTESO="${PIN_ATTESO:?PIN_ATTESO (hash server pin s189 = AXnew) obbligatorio}"\n'
        'PIN_OLD="${PIN_OLD:-97b58fd225ca7289}"')
s = sub(s, 'SRV="$(shasum -a 256 "$SRVBIN" | cut -c1-16)"\n[ "$SRV" = "$PIN_ATTESO" ] || fin 9 "pin server $SRV ≠ $PIN_ATTESO"',
        'SRV="$(shasum -a 256 "$SRVNEW" | cut -c1-16)"; OLDH="$(shasum -a 256 "$SRVOLD" | cut -c1-16)"\n'
        '[ "$SRV" = "$PIN_ATTESO" ] || fin 9 "pin server nuovo $SRV ≠ $PIN_ATTESO"\n'
        '[ "$OLDH" = "$PIN_OLD" ] || fin 9 "pin server vecchio $OLDH ≠ $PIN_OLD"')
s = sub(s, "grep -q 's188' \"$MLOCK\"", "grep -q 's189' \"$MLOCK\"")
s = sub(s, '"$H/s188-throughput.sh" "$RU"', '"$H/s189-guardia-ax.sh" "$RU"')
s = sub(s, 'step "identità: server=$SRV', 'step "identità: server_new=$SRV server_old=$OLDH')
AX_OLD = '    AX) env WP_BENCH_DISABLE_CRON=1 /usr/bin/time -l "$SRVBIN" --axum --workers "$WORKERS" --host 127.0.0.1 --port "$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;\n'
AX_NEW = ('    AXold) env WP_BENCH_DISABLE_CRON=1 /usr/bin/time -l "$SRVOLD" --axum --workers "$WORKERS" --host 127.0.0.1 --port "$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;\n'
          '    AXnew) env WP_BENCH_DISABLE_CRON=1 /usr/bin/time -l "$SRVNEW" --axum --workers "$WORKERS" --host 127.0.0.1 --port "$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;\n')
s = sub(s, AX_OLD, AX_NEW)
s = sub(s, '      ab -q -c "$lev" -n "$WARM" "$BASE$path" > /dev/null 2>&1',
        '      ab -q -c "$WORKERS" -n "$WARM" "$BASE$path" > /dev/null 2>&1  # S-189: warm-up a -c W (ogni unit cache calda)')
# giudice: sostituito per intero (dichiarato)
i0 = s.index('python3 - "$OUT" "$T" "$SRV"')
i1 = s.index("\nPY\n", i0) + len("\nPY\n")
JUDGE = r'''python3 - "$OUT" "$T" "$SRV" "$OLDH" "$GIT_REV" "$DRIVER_SHA" "$C_LEVELS" "$N" "$R" "$PHPV" "$GRADE" "$URLS" "$ARMS" "$EXPECTED_REQ" > "$VERD" <<'PY'
import os, re, statistics, sys, glob
out, T, NEW, OLD, GIT, DSHA, CL, N, R, PHPV, GRADE, URLS, ARMS, EXPREQ = sys.argv[1:15]
levels = CL.split(); N = int(N); R = int(R); urls = [e.split(":", 1)[0] for e in URLS.split()]; arms = ARMS.split()
label = {"AXold": f"php-server --axum --workers 4 pin s188 {OLD} (stash)", "AXnew": f"php-server --axum --workers 4 pin s189 {NEW} (canonica)"}
BAND = {lev: ((0.95, 1.05) if lev == "1" else (0.97, 1.05)) for lev in levels}
def rd(p): return open(p, errors="replace").read() if os.path.exists(p) else ""
def ru(p):
    v = rd(p).split(); return [int(x) for x in v] if len(v) == 5 else None
def abf(txt, key):
    m = re.search(key + r":\s+([\d.]+)", txt); return float(m.group(1)) if m else None
def p99(txt):
    m = re.search(r"^\s*99%\s+(\d+)", txt, re.M); return float(m.group(1)) if m else None
boot = {}
for c in range(1, R + 1):
    for a in arms:
        d = f"{out}/tp-{T}/c{c}-{a}"; log = rd(f"{d}/server.log")
        tl = {}
        for pat, key in ((r"([\d.]+)\s+user", "user"), (r"([\d.]+)\s+sys", "sys"), (r"^\s*(\d+)\s+cycles elapsed", "cyc"), (r"^\s*(\d+)\s+instructions retired", "ins")):
            m = re.findall(pat, log, re.M)
            if m: tl[key] = float(m[-1])
        b = {"void": rd(f"{d}/VOID").strip(), "req": rd(f"{d}/reqcount.txt").strip() or "n/a", "voids": len(glob.glob(f"{d}.void*")),
             "tree": (re.search(r"tree_pids=(\d+)", rd(f"{d}/identity.txt")) or [None, "-1"])[1],
             "ghz": (tl["cyc"] / (tl["user"] + tl["sys"]) / 1e9) if tl.get("cyc") and (tl.get("user", 0) + tl.get("sys", 0)) > 0 else None,
             "ipc": (tl["ins"] / tl["cyc"]) if tl.get("cyc") and tl.get("ins") else None, "cell": {}}
        for lev in levels:
            for u in urls:
                t = rd(f"{d}/c{lev}-{u}.ab"); r0, r1 = ru(f"{d}/c{lev}-{u}.ru0"), ru(f"{d}/c{lev}-{u}.ru1")
                rps = abf(t, "Requests per second"); failed = abf(t, "Failed requests"); non2 = abf(t, "Non-2xx responses")
                cell = {"rps": rps, "failed": failed, "non2xx": non2 or 0.0, "p99": p99(t),
                        "ok": bool(rps and failed == 0.0 and not non2 and not b["void"] and r0 and r1)}
                if r0 and r1: cell["cpu"] = (r1[0] + r1[1] - r0[0] - r0[1]) / N / 1e6
                b["cell"][(lev, u)] = cell
        boot[(c, a)] = b
print(f"== s189 FETTA 1 guardia AX (NON-REGRESSIONE, mai record): AXnew {NEW} vs AXold {OLD}, --axum --workers 4, WordPress, PHP {PHPV} assente per disegno | git={GIT} driver_sha={DSHA} tentativo={T} epoch={int(__import__('time').time())} ==")
print(f"grade={GRADE}  # VERDICT solo con N>=150 R>=3 e i tre livelli; rc autoritativo = pair-out/ga189-{T}.done")
print(f"protocollo: ab chiuso per livello C={levels}, warm-up -n 20 a -c 4 escluso, N={N} per cella, R={R} cicli ordine AXold,AXnew nella STESSA finestra · canonica = AXnew/AXold req/s per cella nello stesso ciclo, mediana dei {R} · bande pre-registrate C=4/8 [0,97;1,05], C=1 [0,95;1,05] · spread ≤5% per braccio altrimenti cella VOID · mediana fuori banda con un ciclo dentro ⇒ VOID (rimisura), {R} cicli concordi fuori ⇒ KILL (KS-189-5) · companion p99 (C=8 nuovo ≤ vecchio; >1,10× REPERTO) e cpu(u+s)/req (±2%) · gate richieste/boot={EXPREQ}")
voids = [f"c{c}-{a}({boot[(c,a)]['void'][:50]})" for c in range(1, R + 1) for a in arms if boot[(c, a)]["void"]]
retried = [f"c{c}-{a}×{boot[(c,a)]['voids']}" for c in range(1, R + 1) for a in arms if boot[(c, a)]["voids"]]
print(f"igiene: tree_pids per braccio {sorted(set((a, boot[(c,a)]['tree']) for c in range(1,R+1) for a in arms))} · richieste/boot={sorted(set(boot[k]['req'] for k in boot))} (attese {EXPREQ}) · boot scartati: {' '.join(retried) or 'nessuno'} · boot VOID finali: {len(voids)} {' '.join(voids)}")
print("classe di core (time -l): " + " ".join(f"c{c}-{a}={boot[(c,a)]['ghz']:.2f}GHz/IPC {boot[(c,a)]['ipc']:.2f}" for c in range(1, R + 1) for a in arms if boot[(c, a)]["ghz"]))
rc = 0; med = {}; per = {}
for a in arms:
    print(f"\nbraccio {a} = {label.get(a, a)} — req/s [per ciclo] mediana spread · p99 ms · CPU(u+s)/req ms")
    for lev in levels:
        for u in urls:
            cells = [boot[(c, a)]["cell"][(lev, u)] for c in range(1, R + 1)]
            rps = [x["rps"] for x in cells if x.get("rps")]
            valid = all(x["ok"] for x in cells) and len(rps) == R
            spread = (max(rps) - min(rps)) / statistics.median(rps) if len(rps) >= 2 else None
            void = (not valid) or (R >= 3 and spread is not None and spread > 0.05)
            if not void:
                med[(a, lev, u)] = statistics.median(rps); per[(a, lev, u)] = [x["rps"] for x in cells]
            else: rc = rc or 1
            p9 = statistics.median([x["p99"] for x in cells if x.get("p99") is not None] or [0]); cpu = statistics.median([x["cpu"] for x in cells if x.get("cpu") is not None] or [0])
            sp = f"{spread*100:.1f}%" if spread is not None else "n/a"
            print(f"  C={lev:<2} {u:<6} rps=[{' '.join(f'{v:.1f}' for v in rps)}] med={statistics.median(rps) if rps else float('nan'):.1f} spread={sp} {'VOID' if void else 'ok'}  p99={p9:.0f} cpu/req={cpu:.1f} failed={sum(int(x['failed'] or 0) for x in cells)} non2xx={sum(int(x['non2xx']) for x in cells)}")
kills = []; voidc = []
print("\nAXnew/AXold req/s per cella [per ciclo] mediana banda esito:")
for lev in levels:
    lo, hi = BAND[lev]
    for u in urls:
        if ("AXnew", lev, u) not in med or ("AXold", lev, u) not in med:
            print(f"  C={lev:<2} {u:<6} VOID (cella senza mediana)"); voidc.append(f"C{lev}/{u}"); continue
        rs = [n / o for n, o in zip(per[("AXnew", lev, u)], per[("AXold", lev, u)]) if o]
        m = statistics.median(rs); inside = [lo <= r <= hi for r in rs]
        if lo <= m <= hi: esito = "ok"
        elif any(inside): esito = "VOID (rimisura: un ciclo dentro)"; voidc.append(f"C{lev}/{u}")
        else: esito = "KILL (3 cicli concordi fuori banda)" if m < lo else "REPERTO >1,05 (attribuire prima di promuovere)"; kills.append(f"C{lev}/{u}={m:.3f}")
        print(f"  C={lev:<2} {u:<6} [{' '.join(f'{r:.3f}' for r in rs)}] med={m:.3f} banda=[{lo:.2f};{hi:.2f}] {esito}")
def comp(key, name, thr, only_lev=None):
    # criterio p.4: il REPERTO p99 è definito SOLO a C=8 (revisione S-189 rilievo 4: a C=4 era un over-flag);
    # cpu/req ±2% su ogni cella. Le altre celle p99 restano stampate come advisory.
    rows = []
    for lev in levels:
        for u in urls:
            n = statistics.median([boot[(c, "AXnew")]["cell"][(lev, u)].get(key) or 0 for c in range(1, R + 1)])
            o = statistics.median([boot[(c, "AXold")]["cell"][(lev, u)].get(key) or 0 for c in range(1, R + 1)])
            flag = " REPERTO" if o and n / o > thr and (only_lev is None or lev == only_lev) else ""
            rows.append(f"C={lev} {u}={n/o:.3f}{flag}" if o else f"C={lev} {u}=n/a")
    scope = f" a C={only_lev}, advisory altrove" if only_lev else ""
    print(f"companion {name} nuovo/vecchio (REPERTO se > {thr}{scope}): " + " · ".join(rows))
comp("p99", "p99", 1.10, only_lev="8"); comp("cpu", "cpu(u+s)/req", 1.02)
print(f"celle VOID: {' '.join(voidc) or 'nessuna'} · KILL/REPERTO: {' '.join(kills) or 'nessuno'}")
if kills: rc = 2
print(f"rc={rc}  # 0 = tutte le celle in banda · 1 = celle VOID (rimisura) · 2 = KS-189-5 (cura NON promossa)"); sys.exit(rc)
PY
'''
s = s[:i0] + JUDGE + s[i1:]
emit(os.path.join(H, "s189-guardia-ax.sh"), s, SRC)

# ---------------------------------------------------------------- lanciatore
SRC2 = os.path.join(W188, "s188-lancio.sh")
l = open(SRC2, encoding="utf-8").read()
LH_OLD = """# s188-lancio.sh — COPPIA SERVER WP t3 @ pin s183 (criterio s188-criterio-server-pair.md p.6): COPIA DICHIARATA di
# ../wp184-harness/s184-lancio-pair-t25.sh (manifest s188-lancio-copia.diff, generatore gen-lancio-s188.py) coi SOLI
# adattamenti: nomi s188/t3/pair188, harness wp188, lock TOKEN s188, exec s188-wp-server-pair.sh t3, attesa post-.done del
# watchdog 10 min (nessun ORM). Il resto INVARIATO (testo S-184 conservato sotto).
"""
LH_NEW = """# s189-lancio.sh — FETTA 1 S-189 (guardia AX + tripwire W=1, criterio s189-criterio-guardia-ax.md p.7): COPIA
# DICHIARATA di ../wp188-harness/s188-lancio.sh (manifest s189-lancio-copia.diff, generatore gen-fetta1.py) coi SOLI
# adattamenti: harness wp189, nomi fetta1, lock TOKEN s189, watchdog su catena-fetta1.done, exec s189-catena-fetta1.sh.
# Il resto INVARIATO (gate di quiete CI, anti-flare, loadavg, E2 CPU, sentinella hog, watchdog Data, dichiarazione quieta).
"""
l = sub(l, LH_OLD, LH_NEW)
l = sub(l, 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp188-harness"', 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp189-harness"')
l = sub(l, 'LOG="$H/pair-out/lancio-t3.log"', 'LOG="$H/pair-out/lancio-fetta1.log"')
l = sub(l, 'echo "s188 coppia server WP t3 @ pin s183 $(date', 'echo "s189 FETTA 1 guardia AX + tripwire W=1 @ pin s189 $(date')
l = sub(l, "(TOKEN s188)", "(TOKEN s189)")
l = sub(l, 'WD="$H/pair-out/watchdog-t3.txt"; PD="$H/pair-out/pair188-t3.done"', 'WD="$H/pair-out/watchdog-fetta1.txt"; PD="$H/pair-out/catena-fetta1.done"')
l = sub(l, 'Q="$H/pair-out/quiet-decl-t3.txt"', 'Q="$H/pair-out/quiet-decl-fetta1.txt"')
l = sub(l, "== dichiarazione finestra quieta t3", "== dichiarazione finestra quieta fetta1")
l = sub(l, "dichiarazione in quiet-decl-t3.txt — lancio coppia server WP t3", "dichiarazione in quiet-decl-fetta1.txt — lancio FETTA 1 S-189")
l = sub(l, 'exec /bin/bash "$H/s188-catena-t3.sh"', 'exec /bin/bash "$H/s189-catena-fetta1.sh"')
emit(os.path.join(H, "s189-lancio.sh"), l, SRC2)
