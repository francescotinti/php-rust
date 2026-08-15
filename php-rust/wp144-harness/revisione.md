# Revisione S-144 — lente SEMANTICA (revisore singolo adversariale)

## Reperto principale
**«churn_zval IN budget» non è stabilito da questa misura: per quel canale il giudice pre-registrato è quasi-vacuo.** Il test p.6 confronta quote di *simboli visibili*, ma Zend inlina dtor/assign negli handler: nel denominatore emendato v2 il vm_inline oracle attivo vale 1048/3736 = **28,0%** — basta che ~1/6 di esso sia churn nascosto per superare la soglia (50%×10,3% = 5,15pp) e flippare il verdetto. Il «ROBUSTO in entrambe le letture» copre solo v1/v2, non questa asimmetria; il tie-break S-129 citato è una tassa di *tempo* per-statement, metrica diversa dall'attribuzione di canale. Poiché il bersaglio VIVO di B è proprio churn, il budget che autorizza le fette B poggia oggi su S-129, non su questo profilo — va detto così, o chiuso per misura (i raw ci sono: aggregazione per *intero stack* invece che top-of-stack dà un maggiorante del churn oracle senza rifare le run).

## Reperti secondari
- **Emenda v2 = il vizio censurato da revisione S-143 az.4, ripetuto il giorno dopo averlo sanato sul census**: denominatore cambiato post-dati con awk in sessione, fuori dal parser committato (`s144-profilo-oracle.sh` non contiene l'esclusione idle), senza golden. Ed è asimmetrica: le quote phpr s140 non sono state riemendate — «phpr mono-thread, idle≈0» è plausibile ma non verificato sul raw s140. Per memops la scelta severa è difendibile; il metodo no.
- **«Enumerazione CHIUSA DAL COMPILATORE» è semanticamente invertita** (zval.rs:72-84): il tipo di `zcell` vieta payload non-Zval *nei siti che lo usano*; nulla impedisce a un sito mancato di chiamare `Rc::new(RefCell::new(zval))` direttamente. La chiusura resta diligenza testuale (61 occorrenze) — il maggiorante loose è condizionato ad essa. Mitigato: per flippare servono >106M eventi mancati.
- I growth-alloc dei props dinamici (alloc-nuovo+free-vecchio hashbrown) stanno in other e sono obj-attribuibili ma fuori dal loose; ordine ~banda dyn (≤0,69pp), non flippa — va nella tranche-3.

## Vagliate e respinte
- `zcell` nella build di parità: senza feature è testualmente `Rc::new(RefCell::new(v))`, `#[inline]` — nessun rischio semantico; parità per NOME rc=0.
- `rczval_prop_n=0`: `make_cell` chiama davvero `zcell` (vm/mod.rs:17433) → le promozioni `&$o->prop` sono nel TOTALE; il loose le copre, strict minorante dichiarato.
- Sentinelle BUSY e vecargs-minorante: census a conteggi deterministici (r1==r2 al singolo evento); vecargs non entra nel numeratore obj.

## Azioni S-145
1. Ri-parsare i raw oracle **e** phpr-s140 con parser emendato committato+golden, esclusione idle simmetrica; ripubblicare memops/churn dallo stesso giudice.
2. Sui raw esistenti: aggregazione whole-stack per churn oracle (maggiorante); <5,15pp ⇒ canale chiuso per misura.
3. Nella progettazione B, dichiarare il budget churn poggiato su S-129 finché il p.2 non chiude; nessuna fetta promossa «per budget di questa run».
4. Correggere la dicitura «chiusa dal compilatore» + dente CI che vieta `Rc::new(RefCell::new` fuori da zval.rs.
5. Congelare la baseline fail-names oracle (0 nomi).
