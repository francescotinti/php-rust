# S-164 p.4 — L-AL3 CADUTA A VERDETTO: leva NON PAGANTE (criterio p.3b) — STOP e revert al byte

## Catena degli arbitri (tutti pre-registrati, tutti rc a file)
1. **Smoke al3sm R=2** (`s164-al3sm-verdetto.out`, rc=5): GIUDICE missload
   **D=+0,0** ns/miss (A=282,0 B=282,0; soglia 6,0) — SOTTO SOGLIA e FUORI
   banda [4;10] SOTTO; guardie arrload −8,5 e objdropdef −6,7 mordono a R=2
   ⇒ arbitrato census DOVUTO (emenda rev. S-161 #4).
2. **Census al3** (`s164-census-al3-verdetto.out`, rc=5 FUORI ATTESA di 1):
   Δ class_exists = **199998** vs attesa 199999 (già EMENDATA pre-run per il
   cold-start). A=200006 (1 Box/iter) · B=**8**. REPERTO post-hoc (dichiarato
   come tale, NON assoluzione): il secondo cold-alloc di B è il **buffer del
   Vec `exts` del pool stesso al primo put_ext** — col quale il modello torna
   esatto al singolo alloc. Repliche r1==r2 su entrambi i probe; altri nomi 0.
3. **Verdetto leva**: la leva RIMUOVE davvero il Box per-iter (199998/200000)
   ma **non paga in ns** — p.3b del criterio, pre-registrato: «D_smoke < 4
   pur con census esatto ⇒ leva NON PAGANTE ⇒ STOP senza promo». Vale sotto
   ENTRAMBI gli esiti census, quindi NESSUN rerun census per una leva morta
   (economia dichiarata; il rerun avrebbe solo lucidato il verbale).

## Meccanismo NOMINATO (ciò che la caduta insegna)
- **mimalloc rende il pool ridondante**: alloc+dealloc di un Box ~200B è già
  un pop/push di freelist thread-local; il mio pool replica lo stesso lavoro
  spostandolo in `recycle_frame`, e l'**init del box (memset dei ~10 campi)
  resta identico nei due bracci** (`Box::default()` vs `*b = default()`).
  Prezzo netto dell'alloc su questo sito ≈ 0 ns visibili.
- Il coeff per-sito «autoload 7,0±3,0 ns/alloc» (tabella S-162) NON è il
  prezzo di UNA alloc mimalloc: nelle leve promosse (AL2/AM2/AU1) la rimozione
  dell'alloc veniva INSIEME al dispatch semplificato — il coeff impacchetta
  entrambi. **La classe «Box/Vec-pooling puro senza cambio di dispatch» esce
  RIDIMENSIONATA dalla coda leve.**
- Le guardie morse (arrload −8,5, objdropdef −6,7) muoiono col revert:
  erano il prezzo del branch aggiunto in `recycle_frame` (tocca TUTTI i
  frame) — coerente col verdetto «non paga»: il branch costa più del Box.

## Revert
patch -R -p2 di `s164-al3-edit.patch` (2 file, 0 rej, inverso esatto
−53/+14) + build ricetta ⇒ hash atteso **fea4a2d040a0d8d0 == pin s163**
(verifica al byte nel commit di questo verbale). Gli `unreachable!` ×2
(az.rev. S-163 #4) cadono col revert e TORNANO IN APERTURE, da montare sul
prossimo edit spedito. NOTA operativa: `git apply -R` dalla sottodir ha
fatto un NO-OP silenzioso (path risolti contro la root esterna del repo) —
il revert vero è passato per `patch -R`; verificare SEMPRE lo stato col
diff dopo un apply.

## Bracci a registro
phpr-s164-gemelloA = fea4a2d040a0d8d0 (==pin, build ricetta al 1° colpo a
freddo; il 1° build SENZA ricetta era 1492be21 — il gate identità ha morso,
lezione: la ricetta è parte del gemello) · phpr-s164-al3-B = b0f0f766401f93d7
(smoke parità 2 modi ok) · disasm bl run_loop 6033==6033 Δ=0.
