#!/usr/bin/env python3
"""s194-corpus-emenda-p4.py — S-194 P4, declared emend of the corpus gate: one frozen Zend failure passes now (a readonly property is
marked initialised only after the typed coercion succeeded, like PHP). DECLARED COPY of s194-corpus-emenda-c3a.py; only adaptations:
the name list (one test), the acceptance tag (s194P4), the output name. Removes the name from the frozen fail-sets and its row from
the golden digests, both modes; asserts the name is there exactly once and that the acceptance run saw exactly this one missing and
zero extra. Output: s194-corpus-emenda-2026-11-25.out. Generato con assistenza AI (Claude Fable 5.2)."""
import pathlib
P = pathlib.Path("/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust")
G = P / "wp109-harness/corpus-gate"; Z = "/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests/"
NOW = ["readonly_props/readonly_clone_success4.phpt"]
gate = (P / "wp190-harness/corpus-out/s194P4/corpus-gate.out").read_text()
for mode in ("off", "on"):
    assert f"gate CORPUS({mode}) NOMI: DIVERGE dal congelato (extra 0, mancanti {len(NOW)})" in gate, mode
    assert f"gate CORPUS({mode}) CONTENUTO: digest == golden" in gate, mode
for n in NOW:
    assert gate.count("-" + Z + n) == 2, n
out = ["S-194 P4 (criterio P p.13), emenda dichiarata del corpus-gate (segno di inizializzazione di una readonly dopo la coercizione riuscita; zero pass->fail, contenuto == golden)"]
for mode in ("off", "on"):
    f = G / f"corpus-s109-{mode}.fails"; names = f.read_text().splitlines(); n0 = len(names)
    for n in NOW:
        assert names.count(Z + n) == 1, (mode, n)
    names = [x for x in names if x[len(Z):] not in NOW or not x.startswith(Z)]
    f.write_text("\n".join(names) + "\n")
    g = G / f"golden-content-{mode}.tsv"; rows = g.read_text().splitlines(); r0 = len(rows)
    keep = [r for r in rows if r.split("\t", 1)[1] not in [Z + n for n in NOW]]
    assert len(keep) == r0 - len(NOW), (mode, r0, len(keep))
    g.write_text("\n".join(keep) + "\n")
    out.append(f"modo {mode}: congelato {n0} -> {len(names)} (passa ora: {', '.join(NOW)}); golden: {r0} -> {len(keep)} righe (tolta la riga, nessun digest cambiato)")
(P / "wp194-harness/s194-corpus-emenda-2026-11-25.out").write_text("\n".join(out) + "\n")
print("\n".join(out))
