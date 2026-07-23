# Rotta WORDPRESS-FIRST — WP-track (dopo WP-42: warm-up by-borrow FLAT→keep, leva locale chiusa; Leva B APERTA con census+piano → WP-43 = stadio 1 registri)

> ⚡ **WP-42 (2026-07-23, `c6e82c2` warm-up + `19b4d27` piano)** —
> (1) **Warm-up `silent_walk` by-borrow ESEGUITO: FLAT su A/B 6 round →
> KEEP** (precedente WP-36; parità: probe ~70 casi oracle==new e old==new
> byte-id, gate22 tutto verde). Zero cloni su isset/empty nested; verdetto
> exists/truthy nei 4 consumer; `??` non passava di lì. Mini-leva CHIUSA —
> le leve locali sul canale churn sono ESAURITE (A WP-33/34, C WP-41,
> warm-up WP-42). (2) **Leva B aperta formalmente**: census op WP-33
> misurato (**743,9M op/run media, 30,77% data-movement puro**, Ret 8,4%
> con Ret→DerefTop 40,5M) + piano d'arco in **`REGISTER_BYTECODE_PLAN.md`**
> (tetto plausibile ~8-15% CPU; 5 stadi; parità a ogni commit).
> (3) ⚠️ **Incidente disco root 100%** durante il gate (fail spuri
> corpus/sess → rilanciati IDENTICI): liberati 6,3G (npm cache + cargo
> debug/), restano ~5G — il grosso è dati utente. **Storia:
> `sessions/WP_SESSION_42.md`.**

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
- **Bytecode a registri = unica "leva lunga" approvata**; l'arco è APERTO:
  piano e census in `REGISTER_BYTECODE_PLAN.md` (WP-42). JIT fuori
  orizzonte; arena per-request collide con byte-parity dtor.
- Micro-bench solo advisory: verdetti SOLO su A/B interleaved stesso-giorno
  sul workload reale. Gate per NOME a ogni commit; refactor layout/GC =
  sentinelle drop-order pinnate PRIMA; oracle-probe con `-d log_errors=0`.
- Commit AND push a ogni step; deviazioni deliberate = marker
  `BUG(port):` / `PERF(port):` / `TODO(port):`.

## Stato gate per nome (gate22 completo su `c6e82c2`, 2026-07-23)

- Gate22 verde (wp22-harness/gate-out): corpus **1447** · sess 28 ·
  date 351 · refl 290 IDENTICI · ORM 3E/13F · hk 0E/0F · cargo **1636** ·
  probe gd/mysqli/media byte-id · http DIFF-set 16 · option/restapi
  identici. ⚠️ Se un gate attraversa una finestra disco-pieno: RILANCIARE
  le suite che scrivono (corpus/sess) — i "nuovi fail" ENOSPC mentono.
  (Se ORM/hk in /private/tmp spariscono: ri-estrarre i tarball da
  wp9-harness/gates/.)
- **Full-suite run32** (~/Claude/wpdev, trunk@5e3fced): 30.472 test,
  0E/2F/86W/73S, **fail-set BYTE-IDENTICO a run31**; baseline =
  `wp16-harness/full-out/run32-fails.txt` (88 righe). Master-CPU ~12:50
  nominale su giornata rumorosa — riferimento resta ~11:39 (WP-40) con
  A/B same-day flat. Multisite (WP-28): 1 diff = minimo teorico.
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

## 🎯 PROSSIMO LAVORO (Leva B, stadio 1 — dal piano `REGISTER_BYTECODE_PLAN.md` §5)

1. **Stadio 1 — infrastruttura a parità zero-delta**: `max_temps` nel
   `Func` (=0 ovunque), estensione Frame, tipo `Operand`, pass di
   riscrittura vuoto dietro flag (`PHPR_REG_LOWER`), chiave unit-cache
   con modalità. Diff di bytecode atteso VUOTO a flag spento; gate22 +
   **A/B "infra presente ma spenta" vs old = rumore zero** (guardia
   contro il costo del solo layout — fisica WP-32 Frame). Se lo stadio 1
   non è a costo zero, fermarsi e ridisegnare PRIMA di scrivere op.
2. **Stadio 2 — Binary/CmpJmp a operandi diretti** (assorbe binary_fast/
   CmpJmpConst WP-33/34, sostituzione mai convivenza): bigrammi target
   dal census (ThisPropGet→CmpJmpConst 29,9M ecc.). A/B go/no-go.
3. **NON riproporre**: leve locali sul canale churn (esaurite: fusioni
   WP-33/34, shim gc_note WP-41, by-borrow WP-42); NaN-boxing; SSO union.
4. Fronte footprint (12×): NON aggredito; quando si apre → PRIMA una
   sessione di attribuzione memoria data-driven (metodo WP-26). I verdetti
   sul doc Gemini "vincoli safe-Rust" sono in WP_SESSION_42 (AST-leak
   falso; unit-cache=opcache deliberato; PhpArray già dual-repr).
5. **Validazione Laravel** ([[php-rust-roadmap-wp-first]]) alla chiusura
   dell'arco perf.

## Backlog aperto (non legato a una sessione)

- 🆕 **isset via prefisso `__get` con indici annidati** perde il walk sul
  risultato (`isset($mg->m['a']['b'])` → false, oracle true) — bug
  funzionale preesistente trovato dalla probe WP-42
  (wp42-harness/probe-isset-div.php §3). Candidato fix.
- 🆕 Deprecation PHP 8.5 (chiave null/float-frac) non emesse dentro
  isset/empty (`coerce_key_silent` muto); `isset($nonAA['k'])` non lancia
  Error — famiglia quiet-fetch, catalogare in PHPR_DIVERGENCES se si
  decide di non chiudere.
- Residui strutturali: `ast_printing.phpt` (serve zend_ast_export
  sull'HIR) · xsl `bug69168` (nodi php:function devono aliasare il doc
  live) · tidy `010` (free-order var_dump-di-albero).
- Ret-hook usa ancora gc_cascade (non gc_release_cascade) per oggetti con
  `__destruct` nel subtree — nessun test lo copre oggi.
- Verbo "increment/decrement" per `$null->p++` (oggi "assign").
- Se si toccano date/prelude DateTime: gate ext/date OBBLIGATORIO (351).
- ⚠️ Disco root della macchina a ~5G liberi: i consumatori grossi = dati utente
  (Application Support 33G, Parallels 4,9G, var/folders 6G) — serve
  decisione utente, non pulizia automatica.

## 📊 Report gap perf — ricorrente di fine sessione

Tabella cumulativa e metodo di misura: **`gaps/REPORT_GAP_42.md`** (ultimo
file = tabella viva). A ogni chiusura: misurare media (user CPU +
footprint) e full-suite master-CPU, copiare l'ultimo report in
`gaps/REPORT_GAP_<N>.md` con la riga nuova, riportare il gap all'utente.
Ultimo stato (WP-42): **media ~2,75× (flat, giornata rumorosa) ·
full [run32] · footprint ~12,7× raw maxrss (old==new)**.
