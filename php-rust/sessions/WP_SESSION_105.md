# WP_SESSION_105 — S-105: la leva args SPEDITA (forma 2 direct-bind, +23 ns/iter); la forma letterale SmallVec misurata e caduta come controllo; PIN-106 saldo; coppia WP in volo

**In una frase**: abbiamo eliminato un'allocazione di memoria che il motore
faceva a ogni chiamata di funzione — la prima versione della modifica
(quella suggerita dal comitato) è risultata più lenta ed è stata scartata
con le misure alla mano, la seconda (passare gli argomenti direttamente,
senza contenitore) rende le chiamate di funzione il 14% più veloci ed è
stata adottata dopo tutti i collaudi; il confronto completo con WordPress
è partito in notturna.

**SCOREBOARD** (micro R=5 SUL PIN DI CHIUSURA d4d0fa52, N emessi):

| giudice | S-104 | S-105 | trend |
|---|---|---|---|
| aritmetica | 12,4 | 12,4 | = |
| proprietà | 11,5 | 11,5 | = |
| **chiamate** | **7,6** | **6,3** | **↓ −1,3 (leva args)** |
| stringhe | 6,7 | 6,7 | = |
| array | 4,5 | 4,5 | = |
| regex | 3,6 | 3,6 | = |

WordPress (riferimento WP-102): full CPU **1,89×** · media CPU **2,64×** ·
peak phpr ~1942-1990 MiB — **coppia bimodale RILANCIATA stasera** (chain
off→on detached 23:03, pin d4d0fa52 + server de67cb64; lettura = S-106).
**Leve perf spedite in questa sessione: 1** (H-D args forma 2).
Contatore sessioni-senza-Δ-rapporti: 3 → **0** (KS-GR-106-1 disinnescato).

**Data**: 2026-08-06 (21:5x–23:1x). **Modello verificato all'apertura**:
Fable 5. **Ordine eseguito**: WP-106 §S-105 punti 1 (intero, con coppia in
volo), 3 (parziale: fx20-banda+sigilli sì, mutazioni no), 4 (catalogo sì);
punto 2 → backlog per NOME. **Commit**: 11cc23a → cfa8d3a su main, pushati.

## Ordine eseguito

| # | Esito |
|---|---|
| **1a · Atto zero** | Criterio PRE-registrato e committato PRIMA di ogni misura (11cc23a): attesa Δ∈[6,14] su calls; co-primari timing E census→0,0000; caduta sotto max(rumore ~3, layout 0,67 N=1); admission-disasm + smoke R=2. |
| **1b · Gate d'apertura** | **G1 probe cap-bump 2→4**: massa census per-chiamata spostata (16,32]→(48,64] per **19.900.000 eventi ESATTI** su alloc E free ⇒ l'args-Vec di Op::Call è il canale PER MISURA. **G2 arità** (apparato GA_ARITY al choke-point bind_params, 60ceb68): calls a2=20M esatti; campione reale wptests (985.695 bind): **quota ≤2 = 73,1%** ⇒ inline-2 dimensionato. **G3 audit-fuga**: 12 chiamanti censiti, nessuna fuga; fixture fx21 7/8 righe oracle-identiche + 🔵 divergenza PRE-esistente scoperta (variadic by-ref diretto, §3.15). `hd-gate-apertura.out` |
| **1c · A/B a DUE FORME** | **Forma 1 (SmallVec letterale, mandato)**: R=5 **Δ=−14,00, 0/5** con census 0,0000 ⇒ CADUTA — il contenitore (aggregato 40 B per valore + IntoIter con check spilled) costa PIÙ della coppia alloc+free che elimina; REVERTITA. **Forma 2 (direct-bind nei frame slots, simple_call arità esatta)**: admission senza flip (bl 5408→5416, dropZval 1109→1111, alloc/dealloc IDENTICI, +324 B), smoke +24,5 (2/2), **R=5 Δ=+23,00 ns/iter, 5/5**, ns/iter 164→141, census **0,0000 alla quarta cifra** su alloc E free ⇒ **PROMOSSA** (d569a56). Il divario tra le forme (~37 ns/iter a parità di census) NOMINA il costo del contenitore. `hd-ab-verdetto.out` |
| **1d/1e · Trigger fedeltà + PIN-106** | Sequenza PIN-106 rispettata: hash₁ 5e8c84c9 → **batteria 1740/0** → re-hash₂ **d4d0fa52** (churn documentato) → STASH `phpr-s105` → fixture **13+5+19a/b+fx20(v2 due soglie)+fx21** tutte PASS (pin bilaterale, oracle 07b0df8d) → **corpus 1417 per NOME ×2 modi IDENTICO a wp82** (runner rilinkato verificato) → micro sul pin (scoreboard sopra). Server ricostruito @ HEAD **de67cb64** + stash (pin phpr INVARIATO dopo la build); grado PIENO rinviato (macchina occupata dalla coppia). **Coppia WP bimodale DETACHED in volo** (s105-pair-chain.sh, daemonizer, pin-check nel chain). `pin106-gate-verdetto.out` |
| **3 · Denti (parziale)** | fx20: cap fisso → **banda a due soglie** (guardia 75 = cap/2: [75,150) = EROSIONE, ≥150 = LEAK; portabilità macOS documentata) — rieseguita sul pin, clean 50, PASS. Sigillo **Copy** sui payload trivial (const-seal bool/i64/f64 accanto a size/align, A-HO-106-1) + **doc del verdetto S-104 nel predicato** is_trivial_drop (A-HO-106-2). ⏸ terza mutazione OBS-8 (sito nominato: mod.rs:4965 `strong_count−2>in_edges`, mutazione −2→−1) e mutante leak-parziale: RINVIATI (build vietate con la coppia in volo). |
| **4 · Fedeltà (timebox)** | Catalogo +2 voci rosse (c9df0af): **§3.14 memory_get_usage = stub costante** (KS-MA-106-1 recepito, cura a due gradini pre-approvata) · **§3.15 variadic by-ref diretto: solo il primo argomento del pack aliasa** (oracle `2 3`, phpr `2 2`; indiziato il push-side `param_by_ref.get(i)` oltre vslot; repro fx21 riga 5). Generator get_gc: non entrato. |
| **2 · Contatori L1I** | NON entrato → backlog per NOME (la sintesi WP-106 lo consente: l'ipotesi icache resta «non firmata»). |

## 🔵 Scoperte

1. **Il costo del contenitore args è ~37 ns/iter, l'alloc+free ~9**: le due
   forme, a parità di census (entrambe 0,0000), distano 37 ns/iter — la
   forma che elimina SOLO l'alloc (tenendo un contenitore più costoso)
   REGREDISCE. Terza conferma indipendente della tesi S-104: la valuta di
   run_loop è il volume di lavoro per op, non i micro-costi di
   alloc/chiamata (mimalloc TL è quasi gratis sul sentiero caldo).
2. **La banda pre-registrata [6,14] era sbagliata in ENTRAMBE le direzioni**
   (−14 la forma 1, +23 la forma 2): pre-registrare l'attesa non predice —
   VINCOLA la lettura; è il criterio co-primario (timing E census) che ha
   permesso di scartare la forma 1 con causa nominata invece di iterare.
3. **Variadic by-ref in chiamata diretta è rotto oltre il primo argomento**
   (§3.15) — scoperto dall'audit-fuga della leva, non da un test suite:
   una fixture di guardia scritta per un rischio ne ha trovato un altro.
4. **L'arità reale di WordPress è bassa**: 73,1% dei bind ≤2 argomenti
   (33% un solo argomento) — il fast path arità-esatta copre la
   maggioranza del carico reale, non solo il giudice sintetico.

## ⭐ Lezioni

- ⭐⭐ **La lettera di un mandato si misura, non si venera**: il concilio
  aveva prescritto SmallVec inline-2; la misura l'ha bocciato e la stessa
  finestra ha prodotto la forma promossa. Due forme + criterio co-primario
  = si può cambiare forma SENZA iterare alla cieca.
- ⭐⭐ **Il census co-primario trasforma una regressione in conoscenza**:
  senza il census 0,0000 la caduta della forma 1 sarebbe stata letta come
  «bersaglio mancato»; col census è «bersaglio centrato, contenitore
  troppo caro» — e indica ESATTAMENTE la forma 2.
- ⭐⭐ **Il probe cap-bump è un SiteTag a costo zero**: spostare la taglia
  dell'alloc indiziata e guardare la massa migrare nel census attribuisce
  il sito con certezza di misura, senza infrastruttura per-sito.
- ⭐ **`rtk grep` tronca in listing mode dentro gli script**: i conteggi
  sul disasm si fanno con awk sul file (i grep del hook hanno mangiato
  due istogrammi prima che me ne accorgessi).
- ⭐ **Un tail troppo corto perde il conteggio della batteria**: la prima
  run è uscita rc=0 ma senza il totale; la seconda run (cache calda) l'ha
  certificato — il conteggio è parte del verbale, non un ornamento.

## Stato binari e processi

- **phpr pin chiusura: d4d0fa5217515dd9** @ d569a56 (codice leva; HEAD di
  chiusura cfa8d3a ha SOLO commit doc/sigilli/harness parity-null — la
  prossima build churna, fa fede HEAD). Stash ADDITIVO `phpr-s105`.
  DEFAULT flag-ON. Batteria 1740/0 · fixture 13+5+19a/b+fx20v2+fx21 ·
  corpus 1417×2 IDENTICO · micro sul pin (scoreboard).
- **php-server: de67cb6466acb030** @ HEAD, stash `php-server-s105` —
  stesso HEAD del pin phpr ma grado PIENO NON ancora eseguito
  (KS-PE-106-1: cifre server solo dopo il grado).
- **IN VOLO**: coppia WP bimodale (chain off→on, daemonizer, partita
  23:03; marker `wp105-harness/pair-chain/pair-chain.done`; ratios in
  `wp105-harness/pair105-ratios-{off,on}.out` a fine gambe). MySQL wp8 su;
  uploads sotto guardia del pair (backup/restore interni).
- Census build (con GA_ARITY) in `phpr-census-target`.
- Harness di sessione: `wp105-harness/`.
