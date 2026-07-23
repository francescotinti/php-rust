# Rotta WORDPRESS-FIRST — WP-track (dopo WP-41: shim gc_note BOCCIATO + attribuzione churn Zval = strutturale → WP-42 = registri, opzionale warm-up silent_get_path)

> 🚫 **WP-41 (2026-07-23, zero delta codice: `a24539f` shim + `17bfbcd`
> revert)** — (1) **Leva C eseguita e BOCCIATA**: shim `#[inline(always)]`
> su gc_note con parità PROVATA (gc-census contatori IDENTICI a WP-40,
> probe byte-id, simbolo inlinato) ma **A/B 4 round = new +0,62%
> consistente** → revert secco. ⭐⭐ Lezione: i 86 self di gc_note erano il
> WALK dei container, non call-overhead scalari; inline a ~60 siti = bloat
> I-cache nel run_loop (fisica WP-33). Su un canale il cui self è WORK,
> l'inlining del frontend è leva morta. (2) **Attribuzione churn
> drop/clone Zval** (2 finestre sample): self drop+clone ~7% della
> finestra GC-heavy MA senza chiamante dominante — churn operandi inline
> nel run_loop (strutturale → registri) + recycle_frame (teardown vero) +
> **dim_is_walk→silent_get_path ~1,5%** (unica inefficienza locale: clone
> di ogni intermedio + leaf anche per isset/empty); memmove frammentato.
> Gate22/run32 NON servite: tree post-revert = `0a03772` gated (diff
> vuoto), run31 resta baseline. **Storia: `sessions/WP_SESSION_41.md`.**

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

## Stato gate per nome (WP-40, ancora validi: WP-41 chiude a zero delta codice)

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

## 🎯 PROSSIMO LAVORO (dopo WP-41: A e C esaurite — dettaglio in WP_SESSION_41)

1. **Arco bytecode-a-registri (Leva B)** — ora è LA strada: le leve locali
   sul canale churn sono esaurite (Leva C bocciata su A/B; attribuzione
   WP-41: churn = traffico strutturale del modello a stack, nessun
   chiamante dominante). Multi-sessione (compiler + run_loop + unit-cache
   + riscrittura fusioni stack-based WP-33/34); PRIMA di aprire: census
   WP-33 per il tetto. Unica leva lunga approvata (WP_SESSION_38).
2. **(Opzionale, warm-up di una sessione registri) `silent_get_path`
   by-borrow** — unica inefficienza locale residua (WP-41: ~1,5% della
   finestra GC-heavy): walk iterativo per riferimento, clone del SOLO
   leaf e SOLO quando il valore serve (`??`); mai per isset/empty. Tetto
   dichiarato ~0,5-1%, A/B obbligatorio, aspettative basse — abbandonare
   subito se flat.
3. **NON riproporre**: shim/inlining frontend gc_note (bocciato WP-41),
   NaN-boxing (WP-32), SSO union (WP-38).
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

Tabella cumulativa e metodo di misura: **`gaps/REPORT_GAP_41.md`** (ultimo
file = tabella viva). A ogni chiusura: misurare media (user CPU +
footprint) e full-suite master-CPU, copiare l'ultimo report in
`gaps/REPORT_GAP_<N>.md` con la riga nuova, riportare il gap all'utente.
Ultimo stato (WP-41, invariato): **media 2,68× · full 2,06× · footprint
12,0×**.
