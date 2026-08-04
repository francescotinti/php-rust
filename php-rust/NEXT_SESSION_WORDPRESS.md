# NEXT_SESSION_WORDPRESS.md — S-96.0: A-ZV2 SI È CHIUSA DA SOLA → WP-97(sessione)

**Ultima sessione**: S-96.0 (2026-08-04, sera) — **l'ordine del Concilio WP-97
eseguito nei suoi passi, e chiuso da un verdetto invece che dal tempo**. Passo 0:
apparato A-SK-93..97 (`env -i` + lista CHIUSA, denti T27-T30, **SELFTEST PASS
rc=0**). Passo 1: fix di soundness A-TH-97-1 + match esaustivi + varianti
mancanti + contatore `would_take_safe_ref`, poi RICONTEGGIO — P2 soddisfatta,
bande invariate, e **i delta F1 esattamente ZERO**: il difetto è reale (provato
a macchina dalla fixture `t4-first-op-def.php`) ma la forma che lo espone non
ricorre in questo corpus. Passo 2: il confronto col piano B — **la strada lunga
NON vince sul perimetro fedele**, quindi `TakeSlot` non è stato scritto e F4 non
è applicabile. Dettaglio: `sessions/WP_SESSION_96.md`.

**⏱ FONDAMENTALI (regola utente 2026-08-03, aggiornare a OGNI rotazione)**:
ultima misura full/media = **WP-94 (2 sessioni fa)** · ultima campagna
sull'oggetto footprint = m90 in WP-90 (6 sessioni fa). S-96.0 non ha
cronometrato: ha contato (conteggi esatti) e ha DECISO di non costruire la leva.
**Il cronometro è fermo da due sessioni e la rotta CPU-VM ha appena perso il suo
prossimo passo: la prima voce del §WP-97 deve essere una leva sull'OGGETTO, non
apparato.**

## Stato gate

- **phpr (CLI, parità release)**: **d5ce86e3342f3926 INVARIATO** (tutto il
  lavoro A-ZV2 vive dietro la feature `zval-census`; nessun ri-stash). Corpus
  Zend per NOME 1418 + refl 290 (non rimisurato: nessun cambio al binario —
  valido per costruzione).
- **php-server**: **f8f4295a1dcdb627** (invariato, non toccato in S-96.0).
  ⚠️ Il pin storico d45b57843eeb1375 resta NON riproducibile — voce APERTA.
- **Gate cifre v3+A1+A-SK-93..97**: `--all` **PASS a HEAD** dopo ogni commit
  (budget alzato con delibera nello stesso commit del nuovo raw). **Il canale
  env di git è CHIUSO**: `SELFTEST PASS rc=0` con T27-T30, ciascuno col proprio
  morso sul giudice pre-cura. **KS-SK-97-1 è soddisfatta**: i PASS di parità
  futuri sono verdict-grade.
- Build di strumentazione: post-fix `3e0e861c5fdbcb9b`
  (`phpr-census-target/`), pre-fix `e318fbfc248a8e35` (`phpr-pre-target/`;
  ricetta per ricostruirlo nella testata di
  `wp96-harness/check-liveness-fixtures.sh`).
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM 3E/13F ·
  hk 1665).

## Permanent Binding Rules (invariate, più una)

**Un privilegio che vale per il processo non vale per la sua discendenza.**
**Un dente che smette di mordere non lo annuncia** (pretendere l'rc ESATTO).
**Un predicato non deve dipendere da ciò che esso stesso introduce.**
**Un confronto identico non è valido se entrambi i lati stanno fallendo.**
**Il rc del runner non è il giudice di una coppia.**
**NUOVA (S-96.0) — una cura ENUMERABILE contro un attacco NON enumerabile è
vacua per costruzione**: l'ambiente di un giudice si COSTRUISCE (lista chiusa),
non si sottrae (lista di negazione).

## §WP-97(sessione) — la rotta CPU-VM ha perso il suo passo: sceglierne uno, sull'OGGETTO

**P0**: pre-flight standard + `--all` PASS a HEAD + pin phpr invariato.
⚠️ `~/Claude/php-rust-output/debug/` si RIGENERA da rust-analyzer (rimossa DUE
volte in S-96.0, la seconda dopo poche ore) e il volume locale sta al limite dei
15G: rimuoverla è parte del pre-flight, non un'eccezione. La taglia si misura
sul momento con `du -sh`, non si cita a memoria.

### Il fatto da cui partire

A-ZV2 non è stata abbandonata: è stato **archiviato un PERIMETRO**. La strada
lunga sul solo nucleo stringhe, con un braccio nuovo, non ha un netto
difendibile (`wp96-harness/design96-confronto-piano-b.md` §5). Restano tre
strade, e la prima ha la precedenza perché è quella che potrebbe RIAPRIRE A-ZV2
a costo quasi nullo:

1. **La FORMA dell'emissione, che nessuno ha valutato.** Hejlsberg (RC-1): «o
   corpo handler nuovo (WP-43) o branch in un arm esistente (WP-38)». Le due
   forme hanno pedaggi DIVERSI: un branch in testa al `run_loop` si paga a OGNI
   opcode, un branch dentro il braccio `LoadSlot`/`LoadVar` si paga solo sulle
   letture di slot ed è per-sito, quindi ben predetto. **Un `LoadSlot` che porta
   un flag `take` deciso a compilazione non è un corpo caldo in più** — e se
   così fosse, il verdetto del passo 2 cambierebbe. Da istruire con la taglia
   `nm -S` PREDETTA prima (A-LB-97-1), su un binario adiacente.
2. **O1 di Bak — outlining dei bracci freddi** (i ~140 opcode rari
   `#[inline(never)]`, restano inline i ~40 caldi). È l'unica leva che ABBASSA
   il numero di corpi caldi, quindi è il prerequisito dichiarato di qualunque
   leva che ne aggiunga uno, ed è a rischio quasi nullo perché meccanica e
   parity-preserving. Il tetto A-LB-97-1 («Δ netto bracci caldi ≤ 0») non è
   soddisfacibile da costruzione finché O1 non è fatta.
3. **Il piano B vero (A-ZV1)**: fast path per riferimento in
   `binary_value_ab`, che NON aggiunge bracci. Ha già il suo design e le sue
   predizioni; ciò che gli manca è la sezione «Correzione» che non è mai stata
   scritta — **se lo si riprende, il primo atto è scriverla, non citarla**.

**Vincolo di grado**: il moltiplicatore del canale (§P1) viene da un profilo
R=1 ed è SCREEN. Qualunque banda derivata da lì eredita quel grado, comprese
quelle di `design96-confronto-piano-b.md`. Il cronometro fermo da due sessioni
è il problema, non il numero.

### Dopo, per NOME (non «più avanti»)

Denominatore omogeneo in GAP_TREND (KS-BG-96-3) · leva arene per-file del
preludio con α RI-DERIVATO (Leijen) · probe slope v2 fuso · attribuzione dello
slope · il pin php-server che non torna · **divergenza §3.10 (argomenti
`string` dei builtin: coercizione con warning invece di `TypeError`) — il
perimetro NON è misurato, e misurarlo è il primo passo**.

### BACKLOG PER NOME (invariato, più le voci nuove)

A-AH-78/79 · A-MS-65/66 · A-DS-96-1/2/3 (registry wrapper) · A-PP-83
(battery61 senza reset fra le gambe) · A-SK-92-PROBE · A-AH-70/74/75 · A-AH-73
· audit A-BG-72 · debito WP-94 non-A (ancoraggi campo, perimetro root, sigilli
E1→E3, checker LSP D1→D4) · residui A-DS51 fasi 2-3 · `stream_get_wrappers`
incompleto · gc_note_frame bitmask per-funzione (Stogov §3) · hash cachato in
PhpStr (Stogov §4).

### Criteri di CHIUSURA del fronte Axum/php-server (invariati)

1. Slope attribuito per NOME — PARZIALE. 2. Leva per-file eseguibile.
3. Parità + ricevuta pin — APERTA (pin php-server). 4. Apparato CONGELATO
fuori quota. 5. Batteria riproducibile — SODDISFATTO con riserva.

**NON riproporre**: tutti i NON-riproporre WP-83..95 restano; in più —
**«la strada lunga non aggiunge opcode al percorso caldo»** (falso: `TakeSlot`
è un braccio nuovo, Bak A-LB-97-1); **«il piano B è la superistruzione
`LoadSlot+Binary`»** (il riferimento a `design95-leva-zval.md` §Correzione è
PENDENTE: quella sezione non esiste, il piano B su disco è A-ZV1 e non è una
superistruzione); **«il perimetro F2 intero»** come base di un F3 fedele
(refutato da Stogov); **«sanificare l'ambiente togliendo le variabili che
conosciamo»** (lista di negazione = vacua per costruzione); **«una fixture che
non morde prova che il difetto non c'è»** (il controesempio di Hoare è vero e
non morde nella sua forma letterale).

---
**Chiusura**: 2026-08-04. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
