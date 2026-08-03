# NEXT_SESSION_WORDPRESS.md — S-93.0: L'ARENA-CHE-MORIVA-IN-SILENZIO → WP-94(sessione)

**Ultima sessione**: S-93.0 (2026-08-03, commit 2859c81 → 070fabf →
d011a86 + chiusura sanatorie) — DIRETTIVA UTENTE A/B/C eseguita. **A**:
riparazione di autorità (A-SK-82 tether su BASH_SOURCE[0] + A-AH68
identità dal contenuto, dente T23 col morso della copia). **B**, la
questione dei worker: i sei blocchi huge per worker NOMINATI dal vivo =
sei chunk di raddoppio di UNA arena bumpalo del parse del PRELUDIO stdlib
(per-thread, prima richiesta), TUTTI liberati alla prima richiesta;
`malloc_huge` di m90 è cumulativo per costruzione (MI_STAT=0); LEVER-2
(mi_collect) = EFFETTO NON RILEVATO (probe R=1 = SCREEN); leva vera
nominata = arene PER-FILE del preludio (CLI 4,42× oracle su hello).
**C**: giudizio di senso WordPress-su-Axum (no portage oggi). Dettaglio:
`sessions/WP_SESSION_93.md` + `wp93-harness/huge-sites.out` (gradi
post-Concilio WP-95).

**⏱ FONDAMENTALI (regola utente 2026-08-03, aggiornare a OGNI
rotazione)**: ultima misura full/media = **WP-85 (8 sessioni fa)** ·
ultima campagna sull'oggetto = m90 in WP-90 (3 sessioni fa) · S-91.0…S-93.0
= zero campagne verdict-grade (S-93.0 ha prodotto SOLO probe R=1). **Il
Concilio WP-95 ha eletto la COPPIA FULL a prima misura di S-94.0**: la
leva footprint non ha giudice senza un «prima» fresco (WP-48). Il concilio
apre la sintesi con §FONDAMENTALI; apparato in ordine SOLO se blocca
l'oggetto (timebox mezza sessione, permanente).

## Stato gate

- **phpr (CLI, parità release)**: **d5ce86e3342f3926 INVARIATO** (mai
  ricompilato in S-91.0…S-93.0; stash `phpr-wp90` resta la baseline;
  corpus Zend per NOME 1418 + refl 290). ⚠️ **Concilio WP-95/Pedersen**:
  il RIPRISTINO del pin php-server dopo i probe S-93.0 è DICHIARATO, non
  ricevutato in-band (nessun ri-run del gate lever-pins DOPO il probe):
  ogni claim di parità che vi poggia è ADVISORY finché S-94.0 non lo
  ri-certifica.
- **php-server**: release/php-server = build axum-server
  **d45b57843eeb1375 INVARIATO** (ripristinato dal pin dopo i probe; i
  build strumentati mem-census di S-93.0 sono identità dichiarate in
  huge-sites.out). La riga testimone unificata (S-92.0 p4) entra nel
  binario alla RICOMPILA in-campagna. **battery-91pre.sh MAI girata**.
- **Gate cifre v3**: --all PASS a HEAD sul perimetro per COMPLEMENTO;
  budget in vigore **24371**. **⚠️⚠️ A-SK-82 è AGGIRATA (Concilio WP-95,
  Klabnik 7/9 forge RIPRODOTTI a macchina)**: tre canali (env BASH_SOURCE,
  symlink, BASH_ENV) producono un `PASS --all` rc=0 firmato col judge_sha
  pristino su codice NON pristino — **prima voce di apparato di S-94.0**,
  §WP-94 A1.
- **Misure (gradi post-Concilio WP-95)**: TUTTE le cifre S-93.0 sono
  ADVISORY/SCREEN, nessuna verdict-grade (salvo la catena di raddoppio
  come identità aritmetica). Slope fisico probe 18814309 B/worker =
  SCREEN. Contatore full/media fermo a WP-85.
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).

## Permanent Binding Rules (emendate WP-95; attuazioni S-93.0)

Le regole della rotazione restano; NUOVE dalle lezioni S-93.0 (Concilio
WP-95): **un contatore stat non prova retention senza leggere il #if di
build** (MI_STAT, Leijen); **un probe R=1 SCREENA, non refuta** (delta
sotto lo spread inter-build = effetto non rilevato, mai refutato);
**«committed invariata ⇒ nessun decommit» è REFUTATA** (in release
committed non scende su purge); **il tether deve legare l'artefatto che il
kernel legge, mai la stringa che il chiamante sceglie** ($0/BASH_SOURCE da
soli non bastano); **nessuna leva footprint prima di una coppia full
fresca**.

## ⚖️ Concilio WP-95 ESEGUITO (2026-08-03, verbali VINCOLANTI): `wp95-harness/COUNCIL_WP95_REVIEWS.md`

**9 sedie (8 CON EMENDAMENTI + Klabnik REFUTATO), due fasi (9 verbali + 3
team: cifre/misura/leva); ~25 KS nuovi. DUE refutazioni capitali**: (1)
A-SK-82 aggirata da tre canali (PASS forgiati verificati a macchina); (2)
`malloc_huge` cumulativa PER COSTRUZIONE a MI_STAT=0 (Leijen, meccanismo
per NOME free.c:612/page.c:935), con «committed⇒decommit» refutata.
**Retrocessione 4/4**: il probe B3 è SCREEN, non refutazione. **Sanatorie
APPLICATE in chiusura S-93.0** (13 atti team-misura: somma 39423200→
39223200, residuo→688224, B3→SCREEN, huge-worker.out→SUPERSEDED-IN-PART).
Sintesi §FONDAMENTALI + ordine in `SYNTHESIS_WP95.md`.

## §WP-94(sessione) — ordine dal Concilio WP-95 (FONDAMENTALI-first)

**P0 (già fatto in chiusura S-93.0)**: sanatorie applicate. Verificare a
inizio S-94.0 che `--all` PASS a HEAD col budget 24371.

### Mezza sessione d'apparato (tetto duro, ordine di taglio A3→A2, A1 mai)

Solo l'apparato che BLOCCA l'oggetto (una cifra futura nasce da questo rc):
- **A1** (indivisibile, un commit): **A-SK-89** path fisici (`cd -P`/`pwd
  -P`) + **A-SK-90** re-exec sanificante `exec env -u BASH_ENV -u ENV -u
  SHELLOPTS bash -p "$SELF_PHYS" "$@"` come PRIMO atto, marker anti-loop
  VALIDATO (mai una env che salta il re-exec) + **A-SK-88** `declare -a`
  come asserzione del marker + **A-SK-91** falsificatore rc-esatto sui tre
  canali (assorbe la sotto-portata di T23: arm-b deve asserire l'assenza
  dell'escalation a rc=0 firmato). KS-SK-95-1..4.
- **A2** = **A-AH-71** writer= autenticato contro lo sha del battery a HEAD.
- **A3** = **A-AH-69** `.done` per-RIGA (4 campi dalla riga con rev=$BREV).
- **A4** = **A-TH-73 + A-TH-74** env-read fuori dall'allocatore, nessun
  panic-path nel GlobalAlloc (UB latente nel canale di misura, non apparato).

### L'OGGETTO (corpo della sessione, non conta nel timebox)

1. **COPPIA FULL stessa-sera** (media + peak footprint + CPU): PRIMA
   misura, verdict-grade, precondizione della leva (contatore fermo a
   WP-85). Ri-certifica anche il pin php-server (ricevuta in-band).
2. **battery61 riproducibile in modo nativo** (criterio 5, debito 31
   sessioni). *Dissenso ordinale registrato (Bak/Pedersen la vogliono al
   posto 1): se il tempo basta per una sola, decide il plenario/utente.*
3. **Probe slope v2 FUSO** = canale unico di m91 (i 4 emendamenti in UN
   strumento): MI_STAT=1 dichiarato nel banner (mai TRACE nello stesso
   run) + coppia alloc/free in-band nel GlobalAlloc (soglia ≥524288) + eco
   d'arm `fired==W` (raw senza ⇒ VOID) + R≥5 interleaved W∈{1,2,4}
   mediana±2se (NULLO solo come test di equivalenza) + doppia metrica
   peak+residency post-warmup + `huge_note` simmetrico su realloc.
4. **Attribuzione slope ~18,8 MB/worker per NOME**: SOLO dal probe fisico
   on-thread (A-DL-55), MAI da committed (A-DL-66).

### S-95.0 (NON S-94.0) — la leva

Leva **arene PER-FILE del preludio** con i 16 obblighi del team-leva
(`wp95-harness/verbali/team-leva.md` §5): contatore per-unità parse-only
col controllo positivo `Σ T_i ≈ 25795552 B` (touched reale ±10%), dente
sulla semantica bumpalo (touched 25795552, non 39534144), fixture F1-F8
oracle-morse, sentinella `b"prelude"` migrata in TUTTI i ~20 siti (mai a
metà), predizione ex-ante firmata (rapporto ben sotto il 4.42× attuale,
banda in `team-leva.md`), gate parità COMPLETI +
ricert. baseline phpr nello STESSO commit. Rank leve: 1 per-file, 2
precompilato embedded, 3 condiviso Arc, 4 lazy (tie-break 2↔3 da misura).

### BACKLOG PER NOME (non «più avanti»)

A-SK-92-PROBE (grado rc=65) · A-AH-70/74/75 (ancore ledger) · A-AH-73 (HIR
plain-data, precond. leva #2) · audit A-BG-72 (derivate m90 che consumarono
malloc_huge come retained: repair90-estimators, VCOV 0,778, decomposizione
b) · debito WP-94 non-A ereditato (ancoraggi campo, perimetro root, sigilli
E1→E3, checker LSP D1→D4) · residui A-DS51 fasi 2-3 · canale iter-3 restante.

### Criteri di CHIUSURA del fronte Axum/php-server (invariati, delibera utente)

1. Slope ~18,8 MB/worker attribuito per NOME (probe fisico on-thread) —
   PARZIALE. 2. Leva applicata e misurata o refutazione motivata — B3 è
   SCREEN, non ancora chiuso. 3. Parità intatta (+ ricevuta pin). 4.
   Apparato CONGELATO fuori dalla quota. 5. Batteria WordPress riproducibile
   (modo nativo obbligatorio).

**Kill-switch di rotta (WP-95, ~25 nuovi)**: KS-TH-95-1/2/3 · KS-MS-95-1/2/3
· KS-SK-95-1..4 · KS-AH-95-1/2/3 · KS-BB-95-1/2 · KS-PP-95-1/2/3 ·
KS-DL-95-1/2 · KS-DS-95-1/2/3 · KS-BG-95-1/2 — tabelle nei verbali.
**Ereditati ATTIVI**: WP-94 (22) + WP-93 (23) + WP-92 (22) + WP-91 (27) +
WP-88..90. **KS-SK-91-1 NON sollevabile.**

**NON riproporre**: tutti i NON-riproporre WP-83..94 restano; in più —
«tether su una stringa che il chiamante sceglie» ($0/BASH_SOURCE da soli,
KS-SK-95-1); «un contatore stat come prova di retention senza leggere il
#if di build» (MI_STAT, KS-DL-95-1); «committed invariata ⇒ nessun
decommit» (REFUTATA); «refutare una leva con un probe R=1» (SCREEN,
KS-BB-95-1); «peak arena = taglia sorgente» (confonde sorgente e arena);
«leva footprint prima di una coppia full fresca».

---
**Chiusura**: 2026-08-03. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
