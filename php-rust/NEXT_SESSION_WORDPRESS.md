# NEXT_SESSION_WORDPRESS.md — S-94.0: IL FALSIFICATORE CHE HA MORSO CHI LO SCRIVEVA → WP-95(sessione)

**Ultima sessione**: S-94.0 (2026-08-04) — ordine del Concilio WP-95
eseguito. **Apparato A1-A4**: i tre canali di Klabnik RIPRODOTTI a macchina
(ognuno firmava un `PASS --all` rc=0 col judge_sha pristino), chiusi con un
re-exec sanificante sul self FISICO il cui marker anti-loop è **lo stato
sanificato stesso**; falsificatore A-SK-91 = denti T24/T25/T26, **selftest
PASS rc=0**. **OGGETTO**: la **coppia full stessa-sera** è tornata a
esistere e la **battery61** è riproducibile sul modo nativo (criterio 5
SODDISFATTO). Dettaglio: `sessions/WP_SESSION_94.md`,
`wp94-harness/MEASURE94_RESULTS.md`, `gaps/REPORT_GAP_94.md`.

**⏱ FONDAMENTALI (regola utente 2026-08-03, aggiornare a OGNI rotazione)**:
ultima misura full/media = **WP-94 (QUESTA sessione — il contatore è
AZZERATO dopo otto sessioni)** · ultima campagna sull'oggetto = m90 in
WP-90 (4 sessioni fa) · S-94.0 ha prodotto misure **verdict-grade** sulla
coppia full e un gate di accettazione WordPress riproducibile. **Restano
non misurati**: il probe slope v2 e l'attribuzione dello slope per NOME
(criterio 1 PARZIALE) — sono la prima voce d'oggetto di S-95.0.

## Stato gate

- **phpr (CLI, parità release)**: **d5ce86e3342f3926 INVARIATO** — la
  coppia full E battery61 girano su questo pin. Corpus Zend per NOME 1418
  + refl 290 (non rimisurato in S-94.0: nessuna ricompilazione).
- **php-server**: **f8f4295a1dcdb627** (stash additivo `php-server-wp94`).
  ⚠️ **Il pin dichiarato d45b57843eeb1375 NON è riproducibile a HEAD** e
  **A4 è esclusa per differenza misurata** (stesso sha con e senza la
  patch). Voce APERTA: o il pin nacque da un albero diverso da quello
  dichiarato, o la build non è riproducibile byte-a-byte. Finché non è
  deciso, ogni claim che vi poggia è ADVISORY.
- **Gate cifre v3+A1**: `--all` **PASS a HEAD**; il budget in vigore vive
  in `wp81-harness/gate-cifre-corpus.budget`, MAI citato nelle prose
  (A-SK-77 ha morso per la terza volta). **A-SK-82 non è più aggirabile
  dai tre canali WP-95**: T24/T25/T26 lo provano per comportamento a rc
  esatto, ognuno col morso sul giudice pre-cura.
- **Misure**: coppia full S-94.0 = **VERDICT** (raw `pair94.out`, rapporti
  macchina `pair94-ratios.out`). Slope per-worker: nessuna cifra nuova, lo
  SCREEN di S-93.0 resta tale.
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).

## Permanent Binding Rules (nuove da S-94.0)

**Un privilegio che vale per il processo non vale per la sua discendenza**:
se il giudice delega a sottoprocessi, si sanifica l'AMBIENTE CONSEGNATO,
non la shell (`bash -p` non rimuove `BASH_FUNC_*`). **Un dente che smette
di mordere non lo annuncia**: un range che si apre su un pattern presente
anche nella riga che lo usa tronca in silenzio, e il dente diventa vacuo,
non fallito — solo l'rc ESATTO lo rivela. **Un predicato non deve dipendere
da ciò che esso stesso introduce**. **Un confronto identico non è valido se
entrambi i lati stanno fallendo.** **Il rc del runner non è il giudice di
una coppia** (la suite WordPress fallisce anche sull'oracle).

## §WP-95(sessione) — la leva, finalmente con un giudice

**P0**: verificare `--all` PASS a HEAD e che il pin phpr sia ancora
d5ce86e3342f3926 (la coppia full di S-94.0 è la baseline della leva: se il
binario cambia prima della misura, la predizione perde il suo «prima»).

### L'OGGETTO (il corpo della sessione)

1. **LEVA arene PER-FILE del preludio** — è ora legittima: il «prima»
   fresco esiste ed è di S-94.0, sullo stesso pin. I 16 obblighi del
   team-leva (`wp95-harness/verbali/team-leva.md` §5): contatore per-unità
   parse-only col controllo positivo `Σ T_i ≈ 25795552 B` (touched reale
   ±10%), dente sulla semantica bumpalo, fixture F1-F8 oracle-morse,
   sentinella `b"prelude"` migrata in TUTTI i ~20 siti (mai a metà),
   **predizione ex-ante firmata 2,3-2,7× oracle** (falsificata se
   peak_post >40MB o <21MB), gate parità COMPLETI + ricertificazione
   baseline phpr NELLO STESSO commit.
2. **Probe slope v2 FUSO** (slittato da S-94.0, invariato nel disegno):
   MI_STAT=1 dichiarato nel banner (mai TRACE nello stesso run) + coppia
   alloc/free in-band nel GlobalAlloc (soglia ≥524288) + eco d'arm
   `fired==W` (raw senza ⇒ VOID) + R≥5 interleaved W∈{1,2,4} mediana±2se +
   doppia metrica peak+residency + `huge_note` simmetrico su realloc.
3. **Attribuzione slope ~18,8 MB/worker per NOME** — SOLO dal probe fisico
   on-thread (A-DL-55), MAI da committed (A-DL-66). Criterio 1 del fronte.
4. **Il pin php-server che non torna**: decidere fra le due ipotesi con una
   misura (rebuild ripetuto a parità di albero → riproducibilità; oppure
   ricostruzione dell'albero storico → provenienza).

### BACKLOG PER NOME (non «più avanti»)

Regresso del **media footprint** (nominato in S-94.0, NON attribuito:
serve un canale, non una congettura) · A-SK-92-PROBE (grado rc=65) ·
A-AH-70/74/75 (ancore ledger) · A-AH-73 (HIR plain-data, precond. leva
#2) · audit A-BG-72 (derivate m90 che consumarono malloc_huge come
retained) · debito WP-94 non-A (ancoraggi campo, perimetro root, sigilli
E1→E3, checker LSP D1→D4) · residui A-DS51 fasi 2-3 ·
`stream_get_wrappers` incompleto (divergenza confermata dalla full).

### Criteri di CHIUSURA del fronte Axum/php-server

1. Slope attribuito per NOME — **PARZIALE** (invariato). 2. Leva applicata
e misurata o refutazione motivata — **la leva per-file è ora eseguibile**.
3. Parità intatta + ricevuta pin — **APERTA** (il pin php-server non
torna). 4. Apparato CONGELATO fuori dalla quota. 5. Batteria WordPress
riproducibile — ✅ **SODDISFATTO** in S-94.0.

**NON riproporre**: tutti i NON-riproporre WP-83..95 restano; in più —
«sanificare la shell e credere sanificati i figli»; «un dente che non
fallisce sta mordendo» (può essere vacuo: pretendere l'rc esatto);
«un predicato che dipende da ciò che introduce»; «due lati identici =
confronto valido» (possono fallire entrambi); «il rc del runner come
giudice di una coppia»; «citare il budget corpus in una prosa».

---
**Chiusura**: 2026-08-04. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
