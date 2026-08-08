# Verbale sedia Stogov — S-116 → S-117 (lente: Zend engine/opcache)

## VERDETTO: CONCORDO CON EMENDAMENTI

La raccomandazione è giusta nell'ordine operativo, sbagliata nella tassonomia:
tratta C e D come rotte separate e relega C a riserva. Dalla mia lente è
insostenibile: il fatto S-103 (costo/op ~9-10 ns INVARIANTE tra categorie) è la
firma del ciclo di vita dei valori, non delle operazioni. Zend arriva a ~2-3
ns/op non con handler più furbi ma perché la maggioranza degli zval NON ha
ciclo di vita: scalari per valore in 16 byte senza allocazione né refcount;
stringhe interned refcount-free (nomi di proprietà, literal); COW sui soli
contenitori. La lezione S-113 («1 clone Rc/op ≈ 3 ns») lo conferma: 3-4
Rc-op/iterazione SONO il pavimento. Aritmetica: prop pin ~107 ns/iter, oracle
~14; per il 3× servono −65 ns. A (5-15%) + treno di leve da 3-30 ns non li
trova: solo togliere il refcount dal path caldo li trova. Quindi **D fatto
bene È C a rate** — e comincia in S-118, non «se dopo A+B resta >3×».

## ROTTA DALLA MIA LENTE (3 sessioni)

1. **S-117 = A**, per il METRO prima che per i ns: pipeline PGO+LTO fat+
   codegen-units=1 + **order-file ld64** — ⚠️ REFUTAZIONE PUNTUALE: **BOLT non
   esiste su Mach-O** (solo ELF); su darwin il layout deterministico si fa con
   `-order_file`/`-Wl,-order_file`, non con BOLT. Chi scrive «BOLT» nel piano
   S-117 sta pianificando su Linux.
2. **S-118 = D ordinato dal ciclo di vita**: census rc-op/alloc per iterazione
   e per categoria su ENTRAMBI i motori (non frequenze opcode). Ordine
   previsto da Zend: (1) scalari per valore rc-free, (2) stringhe
   interned/literal rc-free, (3) COW contenitori, (4) inline cache famiglia
   L-A, (5) specializzazione handler **ULTIMA** — S-103 la refuta come
   priorità: attacca il costo per-op che non è il collo.
3. **B = regime di promozione** del treno (L-A primo vagone: magnitudine
   +27-29 stabilita ×3; la tassa calls ~1-1,5 ns è esattamente ciò che un
   giudizio a somma assorbe). Parità output e admission restano PER VAGONE.

## EMENDAMENTI

- **R1 — profilo PGO mai sui giudici**: corpus di profiling = WP + held-out,
  MAI le sei micro (teaching-to-the-test). Misura: micro giudicate post-hoc
  col criterio solito.
- **R2 — le bande DECADONO con la pipeline**: dopo A, banda micro 
  (0,40…10,00) e held-out sono VOID. Prima di ogni verdetto: ≥2 leve nulle
  sulla pipeline nuova, banda ri-pre-registrata. Misura: file banda v2
  committato PRIMA del primo A/B.
- **R3 — D si seleziona con contatori di vita, non di dispatch**: harness che
  conta nascite/morti/rc-op per iter su phpr e (via Vexp/DTrace) su Zend.
  Misura: tabella per categoria, delta rc-op ↔ delta ns previsto.
- **R4 — trappole semantiche pre-registrate per ogni rata di C/D**: COW ×
  references (is_ref sospende la separazione); ordine di distruzione
  OSSERVABILE (lezione sweep EAGER); interned ≠ per-richiesta (mai liberate
  nel reset, mandato output-capture intatto); IS_UNDEF ≠ NULL. Gate: fail-set
  1415 per NOME ×2 + batteria, come sempre. NIENTE NaN-boxing: Zend non lo
  usa, perde i tipi-sentinella e complica il flag rc — la rata giusta è
  «tagged value 16B + rc solo sui tipi contati».

## KILL-SWITCH

- **A**: se la banda ri-misurata post-pipeline > 10,00 globale (metro
  peggiorato) o WP < −1% oltre spread A-A′ → revert pipeline, si tiene solo
  ciò che non degrada il metro.
- **B**: se la somma del treno sui giudici < ½ della somma delle magnitudini
  firmate → i vagoni si annichilano (layout): treno fermo, si torna a R2.
- **D/C a rate**: ogni rata che muove il fail-set per NOME o rompe la parità
  output si riverte al byte in sessione; due rate consecutive revertate →
  concilio.

## APPARATO minimo
Solo R3 (contatori rc-op): blocca la selezione dei vagoni; timebox ½ sessione.
