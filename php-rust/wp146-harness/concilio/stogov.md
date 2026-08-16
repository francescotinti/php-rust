# CONCILIO S-146 — bozza indipendente — sedia STOGOV (semantica Zend/opcache)

## VERDETTO
**CONCORDO CON EMENDAMENTI.** Il fatto nuovo (memcpy 69,5%) dice che il collo
è il pavimento «sposta e smista», e la lente Zend dice PERCHÉ: **Zend quei
367,6M movimenti in gran parte non li esegue affatto**. Gli handler Zend
leggono gli operandi via `zval*` (Borrow): nessuna copia, nessun inc/dec.
Copiano solo ASSIGN/SEND/RETURN, e lì i TMP/VAR si CONSUMANO; i CV pagano al
più un ADDREF. phpr invece clona a ogni lettura: scalar 91,1M + str 104,1M
sono in maggioranza cloni che in Zend sono borrow, non take. Quindi la mossa
fedele è **prima non-muovere (borrow), poi consumare (take)** — non il
contrario.

## Posizioni a–e
**a) Forma — CON EMENDAMENTI.** Il flag `take` compilato su LoadSlot non è un
corpo caldo in più: forma ammissibile, va istruita con taglia `nm -S` predetta
e disasm bl-count (A-LB-97-1, lezione H-C2). Ma la forma PRIMA in ordine è il
**through-borrow ai siti consumatori** (precedenti SPEDITI HC1, L-FR1): zero
liveness, zero flag, zero rischio semantico. Il flag-take si istruisce solo
dove il borrow non arriva (il valore deve davvero migrare: store, send, ret).
**b) Perimetro fedele — CON EMENDAMENTI (vincolante per me).**
— *Scalari*: consumabili SEMPRE (nessuna identità, nessuna morte osservabile).
— *Stringhe*: consumabili quando l'analisi è SOUND e con le rinunce S-96
(debug_zval_refcount, compact/extract, ref): la morte anticipata di una ZStr
non è osservabile dal programma PHP (niente destructor/weakref/id). Caveat
t4-first-op-def: «corretto per fortuna del corpus» ≠ corretto — l'analisi si
ricollauda su ORM, non si trasporta.
— *Array/oggetti*: **MAI** come take che anticipa il drop. Il drop transitivo
di un array libera oggetti/risorse: `__destruct` deterministico a DELREF→0,
riuso spl_object_id, WeakReference, chiusura risorse — osservabile ANCHE senza
`__destruct`. Take lecito su container SOLO se il drop dell'ultimo ref resta
nel punto esatto in cui Zend l'avrebbe eseguito (deferral) — e allora conviene
il borrow. In Zend i CV non si consumano: qualunque TakeSlot su slot è già
PIÙ aggressivo di Zend, sta in piedi solo con le rinunce intere.
**c) Censimento ORM — CONCORDO, rafforzo.** I conteggi liveness sono sul media
WP (53,6M slot_reads_rc); i 367,6M sono ORM: nessuna trasportabilità. Ma il
censimento nuovo deve ripartire i movimenti **PER SITO D'ORIGINE**
(slot-read / args / return / prop-get / dim-read) × categoria: il borrow
aggredisce SITI, non slot, e l'ordine delle fette esce da lì.
**d) Alternative — CONCORDO: borrow-first è la fedeltà, non l'alternativa.**
Ordine: (1) estensione through-borrow ai siti consumatori più moltiplicati dal
censimento c; (2) flag-take su scalar+str sound. «Arena-conteggi»: mai
definita ⇒ **si archivia**, salvo definizione su carta che conservi refcount e
destruct refcount-driven (veto costo-sostitutivo, rifondazione A S-143).
**e) Cosa compra — CONCORDO.** Perimetro modellato 1,52 s su 37,6 s: anche
azzerato, ~4% del gap. Compra risoluzione e metodo per il glue fuori modello
(~4,4 s), NON la parità. Nessun claim oltre la risoluzione.

## Emendamenti
**R1** (cosa: censimento per-sito; perché: c; misura: monobinario census, ×2,
r1==r2, quote sito×categoria, parità per NOME rc=0).
**R2** (cosa: fette giudicabili dalla coppia solo se mirano ≥~100M movimenti
evitati — 0,7% di ~42,5 s a 2,88 ns/mov ≈ 104M; sotto: giudice = micro churn +
famiglia, composizione dichiarata).
**R3** (cosa: TakeSlot solo dopo ri-derivazione P1/P2 su ORM con bande firmate
PRIMA dei dati; il moltiplicatore 4,5–6,5% resta SCREEN e non si eredita).

## Kill-switch
**KS-ST-146-1**: fail NUOVO per NOME in weakrefs/destructor/spl_object_id nel
corpus 1414×2 o nelle fixture bilaterali ⇒ STOP fetta (riaffermato).
**KS-ST-146-2**: censimento c con siti aggredibili <100M movimenti ⇒ nessuna
fetta a giudice-coppia; si scala al giudice micro o si chiude B3.
**KS-ST-146-3**: qualunque take su container senza deferral del drop ⇒ veto di
sedia, non negoziabile.

## Mandato inverso
Oggi sappiamo: la ripartizione (memcpy 69,5%) e i prezzi per-movimento firmati;
quota_obj 2,4% — il mio kill-A (<15%) è scattato a fortiori: del mio B-poi-A
resta SOLO B, e B com'era (B1/B2) è chiusa da KS-B4. Resta ciò che Zend fa:
muovere meno, nell'ordine borrow→take, coi confini di b).
