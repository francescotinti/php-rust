#!/usr/bin/env python3
"""Generator of s192-direzione-zl.sh as a DECLARED COPY of wp190-harness/s190-direzione-k0.sh.

Adaptations (each asserted unique): arm A is the twin Z given by env (no pin-hash check), verdict/out paths in
wp192-harness, header, and the direction rule (lever L vs twin Z: D(K0) reported, rc=0 iff D(K0) < 0).
Generato con assistenza AI (Claude Fable 5.2).
"""
import pathlib, subprocess

P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
src = (P / "wp190-harness/s190-direzione-k0.sh").read_text()


def sub(text, old, new):
    assert text.count(old) == 1, (text.count(old), old)
    return text.replace(old, new)


t = src
t = sub(t, "# s190-direzione-k0.sh <tag> — S-190 C2, A/B di DIREZIONE",
        "# s192-direzione-zl.sh <tag> — S-192, COPIA DICHIARATA di s190-direzione-k0.sh (adattamenti: A = gemello Z da env senza\n"
        "# controllo dell'hash del pin; percorsi in wp192-harness; regola: rc=0 sse D(K0) = L−Z < 0). Testo originale: A/B di DIREZIONE")
t = sub(t, 'H="$P/wp190-harness"; H6=', 'H="$P/wp192-harness"; H6=')
t = sub(t, 'A="${PIN:-$HOME/Claude/php-rust-output/release/phpr}"; AEXP="9a6ae11bf7e900ef"', 'A="${A:?A = gemello Z}"')
t = sub(t, '[ "$AM" = "$AEXP" ] || { note "rc=9 A $AM != pin s183 $AEXP"; exit 9; }\n', '')
t = sub(t, 'VERD="$H/s190-direzione-k0-$TAG.out"; OUT="$H/ab-out/direzione-$TAG"',
        'VERD="$H/s192-direzione-zl-$TAG.out"; OUT="$H/out/direzione-zl-$TAG"')
t = sub(t, 'note "== s190 direzione K0 tag=$TAG A=pin $AM · B=tree $BM',
        'note "== s192 direzione L−Z tag=$TAG A=gemello Z $AM · B=leva L $BM')
t = sub(t, "ok0 = k0 < 4.0;", "ok0 = k0 < 0.0;")
t = sub(t, "'< 4 OK' if ok0 else '>= 4: K0 ancora caro'", "'< 0: la leva guadagna' if ok0 else '>= 0: nessun guadagno'")
dst = P / "wp192-harness/s192-direzione-zl.sh"
dst.write_text(t)
diff = subprocess.run(["diff", str(P / "wp190-harness/s190-direzione-k0.sh"), str(dst)], capture_output=True, text=True).stdout
(P / "wp192-harness/s192-direzione-zl-copia.diff").write_text(diff)
print("ok", len(diff.splitlines()), "diff lines")
