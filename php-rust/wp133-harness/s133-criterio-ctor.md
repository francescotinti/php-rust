# Criterio S-133 — forma ctor (70,8 ns, modello s131-propstep p.3): sonda conteggi per-sito + leva resolve-once su PropSet non-plain (commit PRIMA del codice)

## p.1 — Sonda CONTEGGI per-sito (PRIMA della forma, ricetta propstep riusabile)

1. **Oggetto**: validare lo split dei 4 resolve/iter del cammino ctor (objalloc,
   k=4 esatto da S-130) tra i DUE siti enumerati per NOME: sito A =
   `magic_applies` (oop.rs, resolve interna), sito B = fallback di
   `prop_set_entry` (run.rs, `let access = resolve_prop_access(...)`).
   **Predizione registrata (objalloc, per iter)**: entrate `prop_set_entry` = 4
   (2 assegnamenti ctor + 2 default `prop_init` INIT_PROPS che escono prima) ·
   resolve sito A = 2 · sito B = 2 · resolve TOTALI = 4. Su objdatains: totali
   = 9 (4 ctor + 5 statement, k=9 di S-130), A = 2, B = 2 (lo statement
   FieldAssign non passa da PropSet).
2. **Sonda**: SOLO CONTEGGI (AtomicU64, niente timer: i conteggi sono
   deterministici e insensibili al rumore). Modulo `wp133-harness/ctorprobes.rs`
   via `#[path]` (patch `count-probes-ctor.patch` scritta col tree e
   RIPRISTINATA; nessuna strumentazione nei sorgenti del pin); target separato
   `phpr-ctor-probe-target` (a freddo: il TTL S-131 è stato potato); dump
   atexit su `PHPR_CTOR_PROBES`. R=2 per categoria, conteggi/N identici tra R
   pena indagine; parità stdout vs oracle attesa.
3. **Lettura**: se lo split NON è 2+2 (sito nascosto), la forma si RIDISEGNA
   dal reperto prima di ogni A/B; l'UB della leva si rifà dai conteggi veri.

## p.2 — LEVA «ctor resolve-once» (forma dal modello + sonda)

1. **Forma**: in `prop_set_entry` (cammino non-IC, non-fast: classi non
   plain_set_props — il ctor di `En` con prop tipizzate vi cade sempre) UNA
   `resolve_prop_access` calcolata DOPO i check hook (il cammino hooked resta a
   zero resolve) e PRIMA di `magic_applies`; `magic_applies` si scinde in
   wrapper (per gli altri call-site, invariati) + `magic_applies_resolved`
   che riceve la resolve pronta; il blocco chiave/slot/IC-fill/Denied riusa la
   STESSA resolve. Equivalenza per costruzione: stessi input (classes, cid
   post-lazy, name, cur) nei due siti oggi; nessuna mutazione tra i due punti
   (i rami hook/magic escono prima); ordine errori invariato (Denied si decide
   dopo il magic-check, come oggi). Delta atteso: 4→2 resolve/iter su objalloc.
2. **UB modello**: 2 × 17,7 = **35,4 ns/iter** (70,8/4 per resolve, cammino
   Denied/Dynamic — s131-propstep p.3); la riconciliazione D vs UB si dichiara.
3. **Giudice**: **objalloc** (cammino ctor puro; phpr s132 = 983,3 ns/iter) +
   co-giudice **objdatains** (1200,0): entrambi sopra soglia per promuovere.
   **Soglia = max(4 ns, rumore drop-1 del run stesso per gamba, spread-batch
   s132 gambe B = 6,7 ns)** (objalloc B: 2,96–2,98 s; objdatains B: 3,61–3,63 s
   @ N=3e6 dal verdetto s132-ab-lo1). Smoke R=2 con early-stop a segno opposto;
   riconciliazione smoke↔R5 dichiarata con banda = spread-batch.
4. **A/B**: `s133-ab.sh` = COPIA DICHIARATA di `s132-ab.sh` (soli cambi: A=pin
   s132 stash `phpr-s132` 6af6e497, B=candidato, giudici/soglie di questo
   criterio, OUT in wp133-harness). R=5 ABAB alternato, user CPU
   netto-pavimento per-binario, N dal sorgente, quiescenza gate SEPARATO con
   rc nell'header, argv senza pattern del gate. Guardie SOLO-REGRESSIONE:
   objchurn (banda fondata gambe B s132 = 13,3) · objmap (10,0) · le sei micro
   con SL come in s132 (soglia_reg = −max(4, SL)). Churn dichiarato nel
   verdetto: il tree porta anche i commenti/debug_assert az.rev. S-132 (commit
   c1f8f54, zero codegen release ma shift di riga nei metadati panic).
5. **Costo dichiarato**: il cammino hooked-set NON paga resolve aggiuntive
   (resolve calcolata dopo i check hook); il cammino magic-set (`__set`
   dispatch) paga la resolve UNA volta come oggi (era il sito A); nessun
   contenitore nuovo, nessuna cache.
6. **Promozione**: catena s132 riusata (`s133-promozione.sh` copia dichiarata):
   batteria `cargo test --release` (rc dal comando) · corpus-gate ×2 modi
   (fail-set 1414 per NOME + golden) · fixture-chain s109 con inventario NUOVO
   `hc1 move recv fx20 fx21 w9 preg teardown` · micro R=5 · gate ORM per NOME
   (16) · hk 0E/0F · pin SOLO via scripts/pin-phpr.sh + pin-server.sh (dopo la
   build canonica ricontrollare l'hash del server). Fallimento di un gate ⇒
   niente pin, revert al byte verificato.
