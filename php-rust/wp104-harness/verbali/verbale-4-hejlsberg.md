# Verbale sedia 4 — Hejlsberg (compilatori incrementali, emissione, dedup) — Concilio WP-104

Fonti: NEXT_SESSION §S-103; WP_SESSION_102 punti 5-6; miei A-HE-103-1..7;
`php-cli/tests/absent_eq_one.rs`; `compile/reg_lower.rs` 995-1112.

## VERDETTO: APPROVATO CON RISERVE — S-102 ha eseguito A-HE-103-1/3/4 con onestà (attese scritte prima, residuo confermato, metà tautologica eliminata), ma il dente sottoprocesso porta un difetto di copertura CAPITALE e il body-zoo lascia lasco il conteggio.

### (a) Dente sottoprocesso vs A-HE-103-3
Impianto giusto: due processi, env COSTRUITO, PHPR_REG_LOWER assente vs
`=1`, verdetto = dump-diff byte + stdout. Le tre obiezioni minori si
sciolgono da sole: il PATH nel dump è lo STESSO file in entrambi i bracci
(nessun differencer); i buchi di env_clear (TMPDIR/HOME assenti) sono
COMMON-MODE — identici nei due bracci, non possono fabbricare un verde
sul diff, e un crash sarebbe rumoroso (rc!=0 asserito fail-closed). MA:
1. **R-HE-104-1 (capitale, recidiva della classe R-HE-103-1)**: il
   commento dichiara «modulo intero» ma NESSUN controllo prova che
   `PHPR_DUMP_OPS` stampi i corpi FUORI funnel. Il controllo positivo
   (forme registro) è soddisfatto dal solo loop di `f` — se il dump
   omettesse prop_init, il dente certificherebbe l'identità di un dump
   PARZIALE proprio sull'angolo (RC-2) dove un sito ambientale residuo
   vivrebbe. Copertura dichiarata ≠ copertura controllata.
2. Il giudice non ha mai dimostrato di saper vedere ROSSO: manca il
   braccio discriminante. Se il dump stampasse una forma pre-lowering,
   due bracci sarebbero byte-identici per costruzione (il controllo forme
   lo mitiga solo in parte).
- **A-HE-104-1**: terzo braccio `=0` nel dente: `dump_zero != dump_one`
  asserito — prova che il diff DISCRIMINA i modi.
- **A-HE-104-2**: controllo positivo sul fuori-funnel: asserire che il
  dump del braccio assente contiene la sezione prop_init (sotto ON il suo
  `Binary(Add)` residuo è il marcatore perfetto, già pinnato dal
  body-zoo). Se il dump non lo stampa, PRIMA si estende il dump, POI il
  claim «modulo intero» diventa vero.

### (b) Body-zoo: `>= 1` non basta
L'attesa statica è ESATTA: un solo add in prop_init. `>= 1` resta verde
se l'emissione DUPLICA l'op (regressione di dedup — il mio perimetro) o
se un futuro const-fold ne lascia due per errore. Inoltre il vettore
`funnel` è costruito A MANO (main+functions+closures+methods): una
categoria di corpi nuova (hook, const-thunk) vi sfuggirebbe in silenzio
mentre `all_funcs` esaustivo la vedrebbe.
- **A-HE-104-3**: pinnare `== 1` (conteggio esatto); derivare il funnel
  come `all_funcs` MENO `prop_inits` per identità, mai lista a mano.
  Nota: lo zoo non esercita un const-thunk con add — pinnarlo fuori
  perimetro per NOME o estendere lo zoo.

### (c) Backlog A-HE-103-2 / A-HE-103-7
A-HE-103-2 (budget call-site `enabled()`): la classe ha morso DUE volte
(A-HO-102-1); la cura è una guardia statica da minuti. Backlog aperto è
troppo molle: entra in S-103 punto 5 (igiene) con timebox.
A-HE-103-7 NON è backlog neutro: S-103 punto 3 ESTENDE stackcensus e
mem-census a Call/Ret — lavoro sotto feature. La classe che guarda
(liveness.rs sfuggito in S-101) ricorre ESATTAMENTE lì.
- **A-HE-104-4**: `cargo check --release --features zval-census` (lista
  feature chiusa, KS-HE-103-3) si esegue PRIMA o CONTESTUALE al punto 3
  di S-103; A-HE-103-2 al punto 5 con timebox.

### (d) H-C2 e i miei KS
KS-HE-103-2 si applica per la regola 2 stessa (emissione O runtime ⇒
coppia): «se promossa» in §S-103 punto 2 è corretto — pin non toccato =
niente coppia. KS-HE-103-1 resta tripwire passivo (Op==48), ma il budget
che H-C2 tenta davvero è l'ALTRO:
- **KS-HE-104-1**: pinnare `size_of::<Zval>()` accanto a Op==48 PRIMA di
  aprire H-C2 — un drop fast-out tentato via tag/bit nello Zval è un
  widening D-cache che non si compra in silenzio; corpus è 1417×2, non
  1418.

**Refutazioni capitali: SÌ — R-HE-104-1** (il dente sottoprocesso
dichiara «modulo intero» senza controllare che il dump copra i corpi
fuori-funnel: copertura dichiarata, non provata).
