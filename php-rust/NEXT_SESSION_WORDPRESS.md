# Rotta WORDPRESS-FIRST — WP-track (dopo WP-40: GC marks in-object −2,5%, 2,68× — WP-41 = shim gc_note → attribuzione churn Zval)

> ⚡ **WP-40 (2026-07-23, gated `4c8de21`+`2f00d36`)** — demote churn GC
> chiuso con marks in-object (`GcMark` su Object: slot-index + bitfield;
> buffer unico `Vec<Option<Rc>>`; flag-guard sui set per-id, che restano
> autoritativi): **media −2,5% = 56,05/20,95 = 2,68×** (old stesso-giorno
> 2,75×), full run31 **~11:39**. Parità provata: sentinelle drop-order
> verdi prima/dopo, probe old==new byte-id, gc-census con contatori
> IDENTICI, gate22 tutto verde, **run31 = run30 per nome**, cargo 1636.
> Riprofilo: probe hashmap sparito (sweep 38→19); restano il WALK di
> gc_note (86 campioni; 177M chiamate) e il canale drop/clone Zval
> (132+116) + memmove (108) come colli phpr-only.
> ⭐ Lezione chiave: i flag specchio vanno azzerati a OGNI svuotamento del
> set autoritativo o è under-insert (= destructor perso); il clone del
> buffer deve droppare nello stesso istante della vecchia map remove.
> **Storia completa, meccanica, lezioni e verdetti Gemini 23/07:
> `sessions/WP_SESSION_40.md`.**

## 📁 Convenzioni (decisione utente 2026-07-23)

- Qui SOLO: sintesi ultima sessione · decisioni in vigore · stato gate ·
  prossimo lavoro · backlog. Storia: `sessions/WP_SESSION_<n>.md` (un file
  per sessione, con lezioni e verdetti; ≤WP-27: memoria + git history).
  Gap perf: `gaps/REPORT_GAP_<n>.md` (l'ultimo = tabella viva).
- Chiusura della sessione N: scrivere `sessions/WP_SESSION_N.md`; copiare
  l'ultimo REPORT_GAP in `gaps/REPORT_GAP_N.md` aggiungendo la riga N;
  sostituire la sintesi qui in testa e aggiornare stato gate / prossimo
  lavoro; commit+push.

## 🧭 Decisioni in vigore (fonte citabile: migration/RULEBOOK.md)

- **Zero `unsafe` nel value core** (RULEBOOK §0; NaN-boxing WP-32 e
  SSO-union WP-38 bocciati — non riproporre senza rotta esplicita utente).
- **Bytecode a registri = unica "leva lunga" approvata** (WP_SESSION_38);
  JIT fuori orizzonte; arena per-request collide con byte-parity dtor.
- Micro-bench solo advisory: verdetti SOLO su A/B interleaved stesso-giorno
  sul workload reale. Gate per NOME a ogni commit; refactor layout/GC =
  sentinelle drop-order pinnate PRIMA; oracle-probe con `-d log_errors=0`.
- Commit AND push a ogni step; deviazioni deliberate = marker
  `BUG(port):` / `PERF(port):` / `TODO(port):`.

## Stato gate per nome (WP-40)

- Gate22 verde (wp22-harness/gate-out): corpus **1447** · sess 28 ·
  date 351 · refl 290 IDENTICI · ORM 3E/13F · hk 0E/0F · cargo **1636** ·
  probe gd/mysqli/media byte-id · http DIFF-set 16 · option/restapi
  identici. (Se ORM/hk in /private/tmp spariscono: ri-estrarre i tarball
  da wp9-harness/gates/.)
- **Full-suite run31** (~/Claude/wpdev, trunk@5e3fced): 30.472 test,
  0E/2F/86W/73S, **identico per nome a run30**; baseline =
  `wp16-harness/full-out/run31-fails.txt` (88 righe); master-CPU ~11:39.
  Multisite (WP-28): 1 diff = minimo teorico.
- Suite phpt (misura): xsl 63/64 (da CWD root php-8.5.7) · tidy 44/45 ·
  asym 38/39. Suite phpt SEMPRE con path ASSOLUTO.

## Harness full-suite

```bash
"/Volumes/Extreme Pro/Claude/wp16-harness/run-full-detached.sh" phpr
# col daemonizer perl (double-fork+setsid) — il task-kill a 10' non deve
# raggiungere la run. MAI due gate22 insieme; uploads azzerati PRIMA di
# ogni full run; non ricompilare mentre una run/gate usa il binario.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr>
```

## 🎯 PROSSIMO LAVORO (riprofilo WP-40 ∩ verdetti Gemini 23/07 — dettaglio verdetti in WP_SESSION_40)

1. **Warm-up: frontend `gc_note` (Leva C)** — shim `#[inline(always)]` con
   guardia sul discriminante (`Object|Ref|Array|Closure` → slow path
   out-of-line), così i ~60 call-site (gc_note_frame su slots/stack di ogni
   frame che ritorna, overwrite di slot, displaced degli array) pagano un
   confronto inline invece di call+match per i 177M/run. ⭐ il hook
   `gc_census::note()` resta NEL shim (il contatore deve contare TUTTE le
   chiamate). **Tetto dichiarato ~1-1,5%** (gc_note self 86/10s ≈ 2,9%
   include walk vero — lezione WP-36). Census di parità prima/dopo.
   L'"elisione a compile-time" proposta da Gemini è ridondante col shim.
2. **Canale drop/clone Zval + memmove (Leva A)** — collo phpr-only #1 da
   WP-36 (132+116+108 campioni/10s): attribuzione per-chiamante PRIMA
   (metodo WP-26/39), poi la leva. (CoW già corretto: by-value = Rc bump,
   mai deep-clone passivo.)
3. **Arco bytecode-a-registri (Leva B)** — multi-sessione (compiler +
   run_loop + unit-cache + riscrittura fusioni stack-based WP-33/34);
   census WP-33 per il tetto PRIMA di aprire. SOLO ad A+C esaurite.
4. **Validazione Laravel** ([[php-rust-roadmap-wp-first]]) alla chiusura
   dell'arco perf. Il footprint (12,0×) resta il fronte non aggredito.

## Backlog aperto (non legato a una sessione)

- Residui strutturali: `ast_printing.phpt` (serve zend_ast_export
  sull'HIR) · xsl `bug69168` (nodi php:function devono aliasare il doc
  live) · tidy `010` (free-order var_dump-di-albero).
- Ret-hook usa ancora gc_cascade (non gc_release_cascade) per oggetti con
  `__destruct` nel subtree — nessun test lo copre oggi.
- Verbo "increment/decrement" per `$null->p++` (oggi "assign").
- Se si toccano date/prelude DateTime: gate ext/date OBBLIGATORIO (351).

## 📊 Report gap perf — ricorrente di fine sessione

Tabella cumulativa e metodo di misura: **`gaps/REPORT_GAP_40.md`** (ultimo
file = tabella viva). A ogni chiusura: misurare media (user CPU +
footprint) e full-suite master-CPU, copiare l'ultimo report in
`gaps/REPORT_GAP_<N>.md` con la riga nuova, riportare il gap all'utente.
Ultimo stato: **media 2,68× · full 2,06× · footprint 12,0×**.
