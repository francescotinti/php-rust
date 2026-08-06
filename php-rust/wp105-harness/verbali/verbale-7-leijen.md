# Verbale 7 — LEIJEN (allocatore, footprint, censimenti) — Concilio WP-105

## VERDETTO

**S-103 APPROVATA CON RISERVE; bozza §S-104 EMENDATA.** La correzione di
denominatore REGGE (metodo giusto: sorgente del giudice, linearità 199:1
alla quarta cifra). La cifra «1 alloc × 32,0 B» regge sul lato ALLOC ma è
**sovra-firmata sul lato FREE**. L'audit VOID è corretto; il protocollo del
rerun R=7 ha un **difetto di costruzione** sulla misura di dispersione.

## Refutazioni

**R1 — «32,0 B ESATTI»: la conclusione è giusta, la dimostrazione nel .out
è incompleta.** Il bucket (16,32] da solo ammette 17–32 B. Ciò che prova
l'esattezza è l'argomento del SOFFITTO: media 31,999993 con massimo 32 ⇒
quasi tutti gli eventi sono esattamente 32 (deficit totale 130 B su 19,9M
eventi ≈ ≤20 eventi sotto-taglia). Va scritto così; e il Δ istogramma ha
**7 eventi orfani** (le32 +19.900.007 vs galloc_n +19.900.000 esatto):
«ogni altro bucket ≈ 0» non è una cifra — i delta per-bucket si pubblicano
ESATTI (fail-closed sulle cifre, non solo sui gate).

**R2 — «1 free/chiamata bilanciato» NON prova il LIFO stretto né che il
free liberi la STESSA popolazione.** I contatori sono cumulativi: il
bilancio aggregato è compatibile con free differiti alla chiamata
successiva o a lotti in regime stazionario. Peggio: `gfree_note` riceve i
byte ma **non ha istogramma** (solo `galloc_note` chiama `hist_note`,
memcensus.rs:1350-1360) — che i free siano da 32 B è ASSUNTO, non
misurato. «Nasce e muore per chiamata» oggi è inferenza, non censimento.

**R3 — Indiziati: l'aritmetica dei tipi discrimina già, ma va
PRE-REGISTRATA come attesa, non decisa.** Con `size_of::<Zval>()==16`
pinnato: args `Vec<Zval>` cap 2 = 32 B esatti; `Rc<Zval>`/`Rc<UnsafeCell<Zval>>`
= 16 header + 16 = 32; ma **`Rc<RefCell<Zval>>` = 40 B → bucket (32,48]**,
quindi se ret_cell è un RefCell l'indiziato cade PER LAYOUT prima ancora
del SiteTag. Scrivere l'attesa byte-per-tipo nel criterio; il SiteTag
resta il giudice.

**R4 — Il ~5 gc_note/chiamata è una DIVISIONE, non una misura**: dividere
×½ una cifra S-102 già fallata ne eredita ogni altro difetto. Ricontare.

**R5 — Rerun R=7: lo SPREAD (range) è monotono NON-decrescente in R.** A
parità di tetto 51,96 tarato in fase-1, più coppie ⇒ VOID più probabile
PER COSTRUZIONE. E la diagnosi «coppia 1 fredda» è a un lato solo: senza
coppia 1 lo spread A cala a 13,8 ma **B resta a 47,9**, a un soffio dal
tetto — il warmup cura A, non B. Se il peak di B è bimodale (due fasi che
si contendono il massimo; timing GC/purge mimalloc), R=7 non stringe
nulla. Il segno però è già eloquente: 5/5 B>A ⇒ p one-sided 0,031; 7/7
darebbe 0,0078 — il sign test sopravvive al rumore che uccide il test di
magnitudine.

## Emendamenti

- **A-LE-105-1**: istogramma anche sul FREE path (`gfree_note` ha già i
  byte) + delta per-bucket pubblicati esatti (i 7 orfani inclusi).
- **A-LE-105-2**: attesa byte-per-tipo pre-registrata nel criterio SiteTag
  (32=Vec 2×Zval; 32=RcBox+16; 40=Rc<RefCell<Zval>> ⇒ escluso per layout).
- **A-LE-105-3**: gc_note/chiamata RICONTATO a census, mai riscalato.
- **A-LE-105-4**: alla lettura del R=7, verdetto co-primario = sign test
  7/7; dispersione futura per IQR/percentili, mai range con R variabile.
- **A-LE-105-5**: se VOID: high-water event-level (GA_LIVE/GA_PEAK in
  memcensus) + peak PER-FASE (le finestre phys esistono già) come metrica
  bisecabile; il peak fisico resta cifra di riferimento, non giudice.

## Kill-switch

- **KS-LE-105-1**: nessuna cifra «N B esatti» firmata senza soffitto
  esplicito E lato free istogrammato — pena declassamento a «media».
- **KS-LE-105-2**: non citabile un verdetto A/B il cui tetto usa il range
  a R diverso dalla taratura; alla lettura R=7 l'audit lo verifica PRIMA.
- **KS-LE-105-3**: VOID a R=7 ⇒ ridisegno metrica (A-LE-105-5), MAI un
  terzo rerun cieco; nessun bisect finché banda < effetto atteso.

## Priorità S-104

1) Lettura A/B con KS-LE-105-2 + sign test (A-LE-105-4). 2) Leva H-C2
(non contestata da questa sedia). 3) SiteTag H-D SOLO dopo A-LE-105-1/2
(free-hist + attese di layout). 4) Se VOID: A-LE-105-5 prima di ogni
bisect. Igiene: A-LE-105-3 nel timebox.
