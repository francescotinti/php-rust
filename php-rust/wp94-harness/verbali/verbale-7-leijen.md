# Verbale SEDIA 7 — Leijen — Concilio WP-94 (mimalloc, contatori, canale iter-3)

**VERDETTO: ACCETTO la refutazione della MIA ipotesi WP-93 (page-slack). E refuto un'assunzione di design92/A-DL-57.**

## Q1 — Refutazione accettata

Sì, senza riserve tecniche. dl59-join.out regge per NOME: thrSumMatch W/W su tutti i 20 raw, d1mispl/d2mispl=0, Σ_thr committed (4.049.975.808 su w16.r1) >> commit di processo, identità decisiva vis_model≡bin-committed <0,5%. La mia formula letterale dava 1,887 con intercetta NEGATIVA: forma e segno sbagliati — moriva anche senza il dedup. Lo slack fisico 0,090 chiude: la pendenza invisibile vive ad arena/chunk.

**Scomposizione CANDIDATA per NOME** (verificata sui raw m90, ckpt=peak_inreq_pwork):
1. **T-HUGE**: `malloc_huge` current = **39.911.424 B × W ESATTO** (159.645.696@w4, 638.582.784@w16), `malloc_huge_count` = **6×W esatto** (24/96). Sestetto huge per-worker. Ma total==peak==current e current@w4 (159MB) > commit (151MB): o il current non decrementa sul free (stat rotta ⇒ i blocchi sono transienti, il residuo committed post-purge è il candidato della pendenza) o vivono nel reserved (1.078.263.808 B fisso) parzialmente decommitted. **È il primo indiziato.**
2. **T-CHUNK**: chunk_bins S current 2→10 (w4→w16), X≈1: granularità chunk per-worker; servono i BYTE per classe dal header vendored (v3, libmimalloc-sys 0.1.49) — non sono in banda.
3. **T-EAGER**: arena_eager_commit (ord 4) — residuo = committed − (bin-committed + huge-committed); nota: pwork_commit ≡ `mi_arena key=committed` AL BYTE (151.781.376 w4.r1) ⇒ l'invisibile è interamente contabilità d'arena.
4. **T-ABAND**: abandoned — atteso ~0 (i worker non muoiono); si misura per REFUTARE.

**Ordine di misura**: (1) controllo positivo decremento huge-current (one-off, binario esistente, zero campagne); (2) promozione malloc_huge* a righe nominate; (3) barriera A-DL-57/58; (4) join INV(W)=T-HUGE+T-CHUNK+T-EAGER+residuo sui W∈{4,8,12,16}, grammar v2.

## Q2 — A-DL-55 va ridisegnato: SÌ

Simboli DISPONIBILI verificati nel tree (memcensus.rs r.631-721): mi_stats_get_json, mi_heap_of, mi_subproc_visit_heaps, mi_heap_visit_blocks/abandoned_blocks, mi_heap_main, mi_option_get, mi_collect. **mi_heap_get_default resta NON esportato** (r.633-637 e r.1425-1429): la lezione vale ancora. Il canale oggi estrae committed/reserved/pages/page_committed (count3) + arena_count/purged/reset/purge_calls/mmap_calls + chunk_bins; **malloc_huge/malloc_huge_count NON sono righe nominate** (solo dentro mi_arena_json) — e sono la leva della pendenza. Vincoli: parse SECTION-AWARE (precedente A-DL46: commit_calls bandito per parse section-blind); `page_committed` legge 0 a pwork ⇒ controllo positivo prima dell'uso (contatore tutto-zero = controllo fallito, lezione WP-72).

## Q3 — Contaminazione dello heap condiviso

- **CONTAMINATO**: ogni claim m89/m90 formulato per-worker/per-theap dai dump tls (thr=k era heap-by-visit-index replicato); la promessa di A-DL46-census («WHERE the committed lives — per heap») mai realizzata.
- **NON contaminato**: slope_vis/vis_model (l'identità <0,5% lo RIABILITA a livello processo); marginal 0,778 (b_peak e commit sono di processo); b_peak mediana, robustezza 0,972. **VSELF/label census-self**: già ritirata; il finding la spiega meglio (una vista replicata non separa il self), non la resuscita.
- **DA RIVERIFICARE per NOME**: VDL24 per-thread 7.349.977×2 (WP-84) e canary per-thread WP-85 — solo se la sorgente era il piano tls-dump; se era VmGate/contatore on-thread, salvi.

## Q4 — Fedeltà A-DL-57/58

A-DL-57 fedele sui simboli (mi_heap_of esiste). **Ma assume l'esito**: heaps_total=1 in m90 è evidenza prima-facie che in QUESTO tree i thread possano condividere l'heap; se il probe on-thread restituisce ptr IDENTICI, il dente REFUSE scopato rende il canale always-VOID. Serve il pre-test di pluralità PRIMA della barriera. A-DL-58: il jbuf 64KiB di mi_stats_get_json (r.1214) è un'allocazione DENTRO lo snapshot — non nominata fra i buffer pre-allocati.

## Emendamenti

- **A-DL-61**: PT-1 pluralità heap (2 thread spawn, probe, mi_heap_of: ptr distinti?) PRIMA di costruire la barriera; se identici ⇒ split per-worker al piano contatore on-thread, non al piano heap.
- **A-DL-62**: A-DL-55 ridisegno = malloc_huge/malloc_huge_count righe nominate section-aware + mappa byte chunk-bin dal header vendored come riga header + controllo decremento huge.
- **A-DL-63**: controllo positivo page_committed (0 = morto o vero-zero: decidere col morso).
- **A-DL-64**: jbuf pre-allocato fuori fase 2 (o emissione arena-json post-fase-3).

**Kill-switch**: **KS-DL-94-1** lettura di malloc_huge current come massa viva senza controllo decremento ⇒ VOID. **KS-DL-94-2** split per-worker pubblicato senza PT-1 superato in-band ⇒ VOID. **KS-DL-94-3** chiave nuova estratta dal json con parse section-blind ⇒ riga VOID.

**Refutazioni capitali: SÌ (2)** — (a) la mia ipotesi page-slack WP-93 è morta (accettata); (b) l'assunzione implicita di A-DL-57 «il probe produrrà W ptr distinti» è non provata e contraria all'evidenza m90: senza PT-1 il design eredita il difetto che dice di curare.
