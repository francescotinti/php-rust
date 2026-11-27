#!/bin/bash
# s194f-mcalls-conto.sh — S-194 night b, inquiry step 1 (criterion s194-criterio-notte-b.md p.17): retired instructions and cycles per
# method call on the guard fixture wp185-harness/mcalls-dq.php, binaries s183 / s189 / s190 in rotation, R runs each, median per call.
# No build, no night window. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin
W="$(cd "$(dirname "$0")" && pwd)"; R="${R:-5}"; OUT="$W/s194f-mcalls-conto.out"; RAW="$W/out/s194f-mcalls-conto.raw"; mkdir -p "$W/out"; : > "$RAW"; rm -f "$W/out/s194f-mcalls-conto.done"
FX="$W/../wp185-harness/mcalls-dq.php"
B183="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s183"; B189="/private/tmp/phpr-s194c-bin/phpr-s189"; B190="$HOME/Claude/php-rust-output/release/phpr"
chk() { [ "$(shasum -a 256 "$2" | cut -c1-16)" = "$3" ] || { echo "hash $1 inatteso" > "$OUT"; echo 7 > "$W/out/s194f-mcalls-conto.done"; exit 7; }; }
chk s183 "$B183" 9a6ae11bf7e900ef; chk s189 "$B189" 2d42efd6606e8dd9; chk s190 "$B190" 97e000aa00fe802e
cp "$B183" /private/tmp/phpr-s194c-bin/phpr-s183; B183=/private/tmp/phpr-s194c-bin/phpr-s183
for r in $(seq 1 "$R"); do
  for p in "s183 $B183" "s189 $B189" "s190 $B190"; do
    set -- $p
    /usr/bin/time -l "$2" "$FX" > /dev/null 2> "$W/out/s194f-t.err"
    i=$(grep 'instructions retired' "$W/out/s194f-t.err" | tr -s ' ' | cut -d' ' -f2); c=$(grep 'cycles elapsed' "$W/out/s194f-t.err" | tr -s ' ' | cut -d' ' -f2)
    echo "$r $1 $i $c" >> "$RAW"
  done
done
python3 - "$RAW" "$R" > "$OUT" <<'PY'
import sys, statistics as st
N = 60_000_000; rows = [l.split() for l in open(sys.argv[1])]; R = int(sys.argv[2]); d = {}
for r, b, i, c in rows: d.setdefault(b, []).append((int(i) / N, int(c) / N))
def drop1(v): return max(abs(st.median(v[:k] + v[k+1:]) - st.median(v)) for k in range(len(v)))
print(f"== s194f: istruzioni e cicli per chiamata, mcalls-dq (N={N}), R={R}, mediana (criterio p.17) ==")
m = {}
for b in ("s183", "s189", "s190"):
    ins = [x[0] for x in d[b]]; cyc = [x[1] for x in d[b]]; m[b] = (st.median(ins), st.median(cyc), drop1(cyc))
    print(f"{b}: istr {m[b][0]:.2f} · cicli {m[b][1]:.2f} (min {min(cyc):.2f} max {max(cyc):.2f}, drop-1 {m[b][2]:.2f}) · cicli/istr {m[b][1]/m[b][0]:.4f}")
di = m['s190'][0] - m['s189'][0]; dc = m['s190'][1] - m['s189'][1]; exp = di * m['s189'][1] / m['s189'][0]
print(f"D s190-s189: istr {di:+.2f} · cicli {dc:+.2f} · cicli spiegati dalle istruzioni {exp:+.2f}")
print(f"D s189-s183: istr {m['s189'][0]-m['s183'][0]:+.2f} · cicli {m['s189'][1]-m['s183'][1]:+.2f}")
if max(v[2] for v in m.values()) > 4: g = "VOID: dispersione dei cicli oltre 4 per chiamata, da ripetere a macchina calma"
elif dc < 4: g = "il costo NON si riproduce in cicli di giorno: dipende dallo stato della finestra, apertura per nome"
elif dc >= 8 and exp >= dc / 2: g = "costo di ISTRUZIONI sul cammino della fixture: passo 2 = lettura di methodcall_fast"
elif dc >= 8: g = "costo MICRO-ARCHITETTURALE (IPC): passo 2 = bisezione sui cinque passi, in cicli"
else: g = "NON DECISO (fra +4 e +8): si ripete una volta con R=9"
print("GIUDIZIO:", g)
PY
echo 0 > "$W/out/s194f-mcalls-conto.done"
