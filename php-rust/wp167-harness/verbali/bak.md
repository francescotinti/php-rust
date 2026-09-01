# Verbale BAK (V8/HotSpot: hot path, code layout, IC) — S-167, bozza indipendente

## VERDETTO: GO-CONDIZIONATO

**Rotta in una riga**: R1 (dispatch) SOLO dopo una sonda a tre bracci che discrimini misprediction / traffico operandi / semantica Zval su arith — che oggi NON ha decomposizione — con tetto di fattibilità misurato prima di qualunque leva.

## Refutazioni dal mio angolo

1. **R2 è già mezza refutata dai vostri stessi verdetti.** `size_of::<Zval>()==16` pinnato a compile-time (array.rs:93): il move è 2 registri, gratis. A3c NO-GO, borrow ≤1 ns (L-TD1), alloc fast-path ≈0 (L-AL3). Il canale (c) residuo è la SEMANTICA clone/drop, non la taglia — e il drop-census S-103 (11 DropS scalari/iter) sa già prezzarlo. Non riaprite NaN-boxing senza un contatore che lo incrimini.
2. **L'esperienza V8 pre-JIT**: ciò che pagò fu Ignition = macchina a REGISTRI (meno dispatch per statement) + handler OUTLINED piccoli con dispatch in coda (l'indirect branch unico si spezza in coppie op→successore che il predittore impara) + handler specializzati per tipo alimentati da IC (AddSmi). Vicolo cieco: full-codegen (compilare tutto baseline) morì di icache/memoria — la vostra H-C2 è la stessa lezione; le superistruzioni generiche resero POCO rispetto alla specializzazione.
3. **In safe Rust senza computed-goto né tail-call garantite, la leva pagabile è MENO dispatch per iterazione, non dispatch più economico.** Il driver arith è ~8-10 op/iter ≈ 5-6 ns/op contro ~1 dell'oracle: fusione cmp+jmp e load+op+store specializzata per tipo è la forma coerente con le vittorie già firmate (MC1d/MCk = salto d'imbuto).
4. **Vincolo di forma dalle cadute icache (H-C2, MC1 +45 bl ⇒ ±5-8 ns su path non toccati)**: ogni superistruzione è un handler `#[inline(never)]` OUTLINED, mai carne nuova dentro `run_loop`; bl-count del run_loop prima/dopo nel criterio, banda-layout fondata obbligatoria.

## Emendamenti

- **E1**: nessuna leva R1 prima della sonda a tre bracci (PRIMA FETTA).
- **E2**: rispondere al quesito del dossier «quanto del dispatch sopravvive col reg-lowering?» con A/B `PHPR_REG_LOWER` on/off su arith — è il braccio (b) gratis, il flag esiste.
- **E3**: contatore branch-miss/iter (CPU counters Apple Silicon, phpr E oracle — feedback-one-sided-profile) per il braccio (a); mai attribuire misprediction per sottrazione.
- **E4**: veto BOLT/PGO confermato; NaN-boxing resta a veto (E1+refutazioni sopra).

## Kill-switch

**KS-BAK-167-1 (tetto)**: handler fuso straight-line dell'intero corpo del driver dietro flag, semantica identica, corpus verde. Se nemmeno il fuso scende ≤3× oracle su arith, il gap NON è dispatch: R1 muore, si va a R4.

## PRIMA FETTA

Sonda 3 bracci su arith, R=5, criterio pre-registrato: (a) branch-miss/iter due motori; (b) A/B reg-lowering; (c) drop-census dcn!/iter. **Giudice**: attribuzione nominata di ≥60% del gap 38,2 ns/iter ai tre canali. **Soglia GO leva**: canale dominante ≥15 ns/iter; sotto, il concilio ridiscute R4.
