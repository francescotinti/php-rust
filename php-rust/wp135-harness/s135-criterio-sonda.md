# Criterio S-135 p.1 — sonda a CONTEGGI s134 vs s133 (az.rev. S-134 #1+#5): commit PRIMA del run

Scopo: (a) verificare **resolve objalloc 2→0/iter a regime** sul pin s134;
(b) **ripartire per NOME l'eccedenza +101,3** dell'A/B S-134 sui componenti
dichiarati-senza-prezzo del criterio icnp p.2.3 — a conteggi (eventi/iter che
spariscono), NON a prezzi per componente (vietato, REGOLE §3/§4);
(c) escludere il canale layout con disasm bl-count su `prop_set_entry`
(metodo WP-104, completa l'az.rev. #5).

## Metodo (ricetta s133-sonda, estesa)

Due build EMENDATE (mai pinnabili), target separati: gamba A = sorgente s133
@ `59f1cb0`, gamba B = sorgente s134 @ `70f078a` (worktree; il tree canonico
non si tocca). Modulo `wp135-harness/s135probes.rs` (NSEG=12, atomici Relaxed,
dump atexit su `PHPR_S135_PROBES`), innesto `#[path]` fuori-crates (ricetta a
catalogo). Conteggi deterministici ⇒ niente quiescenza (dichiarato, come s133).
Giudici: `wp127-harness/micro-orm/{objalloc,objdatains}.php`, 2 run/gamba,
parità stdout vs oracle, determinismo tra run. rc autoritativo =
`sonda-out/sonda.rc`.

Segmenti (siti nel testo di `prop_set_entry`/`resolve_prop_access`; su A i
siti 2/3 non esistono = 0 strutturale):
0 entrate prop_set_entry · 1 entrate resolve_prop_access (fn, TUTTE) ·
2 IC-hit con bit NP · 3 IC-fill NP · 4 magic-probe (chiamata
magic_applies[_resolved]) · 5 hook-lookup eseguito (blocco !hook_guarded) ·
6 enum-check borrow · 7 asym check (declared) · 8 readonly prop_readonly_decl
(declared) · 9 deprecation props.contains (solo !declared) · 10
coerce_typed_prop_write · 11 IC-hit plain (NP clear).

## Predizioni REGISTRATE (per iter, N=3e6; scostamento = reperto, non si aggiusta a valle)

| seg | objalloc A(s133) | objalloc B(s134) | objdatains A | objdatains B |
|---|---|---|---|---|
| 0 entrate | 2 | 2 | 2 | 2 |
| 1 resolve | 2 | ~0 (fill 1ª passata/sito) | 4 | 2 (solo dim-write) |
| 2 NP-hit | 0 | 2 | 0 | 2 |
| 3 NP-fill | 0 | ~0 (totale ~2) | 0 | ~0 |
| 4 magic-probe | 2 | ~0 | 2 | ~0 |
| 5 hook-lookup | 2 | ~0 | 2 | ~0 |
| 6 enum-borrow | 2 | ~0 | 2 | ~0 |
| 7 asym | 2 | ~0 | 2 | ~0 |
| 8 readonly | 2 | ~0 | 2 | ~0 |
| 9 depr-contains | 0 | 0 | 0 | 0 |
| 10 coerce typed | 2 | 2 (canale pagato) | 2 | 2 |
| 11 plain-hit | 0 | 0 | 0 | 0 |

Lettura pre-dichiarata: l'eccedenza +101,3 è attribuita — per NOME e a
conteggi — ai canali 4·5·6·7·8 che vanno 2→0/iter più il costo di sito della
resolve (canale 1, parte modellata 35,4); il canale 9 era nominato nel
criterio icnp ma NON contribuiva (0→0 atteso: falsificazione misurata di un
componente dichiarato). Se su B un canale tra 4–8 resta >0/iter, il claim
meccanicistico della leva S-134 è FALSIFICATO in quella parte. La magnitudine
per canale resta NON ripartita (nessun prezzo per componente).

## Disasm (parte c)

`objdump --disassemble-symbols` sui PIN STASHATI s133 (c87439a9) e s134
(61896da1): bl-count per NOME dentro il perimetro `prop_set_entry`/run_loop
già agli atti (+681 istr, resolve 21=21); qui: conteggio bl del cammino hit
NP (atteso: nessun bl verso magic/hook/asym/readonly nel ramo hit) →
`s135-eccedenza-chiusura.md`. Il canale layout resta escluso se: conteggi B
confermano i salti E il ramo hit non contiene bl verso i canali contati.
