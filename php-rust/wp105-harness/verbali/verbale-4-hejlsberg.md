# Verbale 4 — Sedia HEJLSBERG (Concilio WP-105) — giudizio su S-103 e bozza §S-104

## VERDETTO

S-103 APPROVATA CON RISERVE sul mio perimetro. Il dente sottoprocesso emendato è
oggi il miglior giudice della coppia assente↔`=1` che abbiamo avuto; ma il
`==1` globale è un tripwire SMUSSATO, il braccio `=0` è senza controllo
positivo proprio, il «modulo intero» resta un claim su uno zoo incompleto, e la
banda-layout 0,67 poggia su UNA perturbazione accidentale. Bozza §S-104:
ordine approvato, con gli emendamenti sotto.

## Refutazioni

**R1 — Il `==1` globale conflaziona tre cause distinte.** Il conteggio di
`Binary(Add)` è sul dump INTERO: un prelude che cambia, un funnel che si
allarga, o un residuo nuovo in prop-init producono lo STESSO fallimento con lo
STESSO messaggio. È un tripwire voluto (giusto che morda su qualunque deriva),
ma un tripwire che non discrimina la causa costa una sessione di diagnosi a
ogni scatto. In più il match è substring su output Debug: un ipotetico
`XBinary(Add)` conterebbe. Non rumore, ma dente da affilare (A-HE-105-1).

**R2 — Il braccio `=0` non ha controllo positivo.** Si asserisce
`dump_zero != dump_absent` e assenza di forme registro: un env che sotto `=0`
UCCIDESSE il dumping intero passerebbe verde (dump vuoto ≠ dump_absent, zero
forme registro banalmente). Recidiva esatta di A-PE-102-1 sul terzo braccio:
il modo OFF si PROVA, non si deduce dall'assenza (A-HE-105-2).

**R3 — «Modulo intero» provato su uno zoo che non è uno zoo.** `Z::{prop-init}`
prova UN corpo fuori-funnel. Restano non provati: corpi di hook get/set,
espressioni di default dei parametri (incluse closure), closure/arrow fn,
inizializzatori static, const-expr di enum. Se una di queste specie ha un
emettitore proprio che ignora `PHPR_REG_LOWER`, il dente attuale è cieco. La
copertura va PINNATA all'enumerazione dei body-kind dell'emettitore, non alla
fantasia della fixture (A-HE-105-3).

**R4 — KS-HE-104-1: la taglia non è il layout.** `size_of::<Zval>()==16` a
compile-time chiude il rischio NOMINATO (crescita), non il repack entro 16 B:
riordino varianti, spostamento del tag, cambio di niche passano a taglia
costante e alterano cache/branch del fast-out. NON chiedo un pin di variant a
runtime: chiedo `align_of` const-asserito e il fingerprint della DEFINIZIONE
di Zval nel criterio (KS-HE-105-1). Il match esaustivo esistente copre la
semantica, non il costo.

**R5 — Banda-layout da perturbazione accidentale = campione N=1.** Il pad è
stato strippato eppure `__text` è cambiato di −308 B: il meccanismo della
perturbazione misurata NON è quello progettato. 0,67 ns/iter è UNA estrazione
dalla distribuzione del layout, non la banda. Mitigazione già in casa: il
rumore run-to-run ~3 ns e il pavimento 4 ns dominano entrambi — quindi NON
blocco la leva H-C2; ma ogni futuro claim di effetto nell'intervallo
(0,67; 3] ns è VOID finché la banda non è ricampionata (KS-HE-105-2).

## Emendamenti

- **A-HE-105-1**: il conteggio `Binary(Add)` si fa PER CORPO (parse delle
  sezioni del dump): `==1` DENTRO `Z::{prop-init}`, `==0` nei corpi funnel,
  e lista dei corpi con residuo nel messaggio di fallimento. Match ancorato
  (inizio-token), non substring nudo.
- **A-HE-105-2**: controllo positivo sul braccio `=0`: `dz` deve contenere
  `Z::{prop-init}` E `Binary(Add)` con conteggio > 1 (il loop in modalità pila
  li emette generici).
- **A-HE-105-3**: BODY_ZOO esteso a hook, default-param con closure, arrow fn,
  static-init; per ciascuno un marcatore asserito PRESENTE nel dump. Tripwire
  di enumerazione: l'insieme delle intestazioni-corpo nel dump == insieme
  atteso per NOME.
- **A-HE-105-4**: prossima leva-nulla NON strippabile: `#[no_mangle]` +
  `#[used]`/black_box sul puntatore, pad a K≥3 taglie (256/1024/4096 B),
  banda = max|Δ|; la si esegue DENTRO la campagna A/B di H-C2, non come
  prefisso nuovo.

## Kill-switch

- **KS-HE-105-1**: `hc2-criterio.out` è VOID se cambia `size_of`, `align_of`,
  O il fingerprint testuale della definizione di `Zval` registrato nel
  criterio.
- **KS-HE-105-2**: nessun Δ micro dichiarato «effetto» se |Δ| ≤ max(banda-
  layout ricampionata, rumore run-to-run misurato quella sera); verdetto
  ammesso: INDETERMINATO.
- **KS-HE-105-3**: il dente absent_eq_one che scatta NON si «aggiorna
  l'attesa» nello stesso commit della causa: prima la causa a registro CON
  NOME, poi l'attesa.

## Priorità S-104 (dal mio seggio)

1. Verdetto A/B R=7 (invariato, primo atto). 2. **LEVA H-C2** — non
aggiungere prefissi: A-HE-105-1/2/3 sono emendamenti al dente eseguibili
nella finestra del gate pieno, A-HE-105-4 dentro la campagna A/B. 3. H-D
SiteTag. 4. Generator-in-cycle. KS-HE-105-1 va scritto nel criterio PRIMA di
aprire la leva (un'ora, non una sessione).
