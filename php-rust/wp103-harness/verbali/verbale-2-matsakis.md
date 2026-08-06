# Verbale sedia 2 — Matsakis (ownership/aliasing/borrow) — Concilio WP-103

**Oggetto**: S-101 (H-C1b: MOVE dell'handle ricevitore su PropGet/PropSet) + bozza §S-102.
**VERDETTO: CON EMENDAMENTI.**

## 1. Il MOVE è la forma giusta — ma il criterio lo difende con un argomento FALSO

La leva è sana: l'handle poppato è owned, il move non crea alias, il braccio
Ref conserva `deref_clone` (il wrapper non viaggia), l'ultimo drop non si
sposta. CONCORDO sulla forma. Ma il criterio (`hc1b-criterio.out` righe
18-21) asserisce: «il valore assoluto [dello strong_count] non è osservabile
da PHP». **FALSO come enunciato — refutazione capitale RC-MA-103-1**: il
motore STESSO osserva conteggi assoluti con aritmetica di handle cablata in
≥6 siti — `oop.rs:1084` (`==2+extra`), `mod.rs:4126` e `4458` (`==2`,
exclusivity in-place), `mod.rs:4913-4918` (collector: `strong_count−2 >
in_edges`), `mod.rs:5251` (`==1`), `run.rs:742` (`==1`). Ciò che salva la
leva NON è l'inosservabilità: è un invariante diverso, mai nominato —
**INV-RECV-1: in ogni punto dell'arm in cui può correre PHP sincrono
(lazy-init) o un observer di conteggio (gc_note→collezione), vive ≥1 handle
owned del ricevitore nell'arm** (`target`). Sotto INV-RECV-1 gli exact-check
(`==1/==2`) non possono scattare (il conteggio resta sopra la soglia in
entrambe le forme) e il trial del collector `−2 > in_edges` resta vero
(1+E>E). La leva sopravvive su un argomento corretto; l'argomento registrato
va sostituito.

Fixture: 04/12/13 NON coprono la finestra che davvero distingue old/new —
durante il handler differito (hook/__get via `continue`) i conteggi sono
IDENTICI (gli handle d'arm sono già morti); la finestra −1 è la
**re-entrancy SINCRONA intra-arm** (initializer lazy che droppa l'ultima ref
esterna) e il **gc_note di PropSet** che attraversa la soglia adattiva
mid-arm con old-value ciclico. Nessuna fixture le combina.

## 2. Perimetro (Silent/Dynamic a clone): asimmetria perf-only SOTTO INV-RECV-1

Le forme a clone tengono 2 handle d'arm, il move ne tiene 1: ogni observer
sopra guadagna solo slack dai handle in più, quindi l'asimmetria non morde
la semantica — ma solo finché INV-RECV-1 vale. Il rischio reale è
l'estensione per copia-incolla SENZA la guardia `matches!(obj, Zval::Ref(_))`:
un wrapper Ref mosso nei path prop violerebbe «il wrapper non viaggia».

## 3. Bozza S-102 punto 3 — refutazione capitale RC-MA-103-2 (controfattuale doppio-contato)

I ~2 ns/coppia MISURATI da H-C1b sono il costo della coppia **clone+drop**.
La forma slot-operando che «legge il ricevitore dallo slot» NON elimina
quella coppia: o CLONA dallo slot (la coppia resta — risparmia solo il
round-trip di pila, che è il ledger del punto 2 → doppio conteggio), o
BORROWA lo slot (VIETATO, unanimità WP-102, ribadito in NON-riproporre — e
sul path fallback la prova «nessun PHP nel borrow-window» FALLISCE per
costruzione: lazy-init è sincrono), o TAKE (slot vuoto visibile alla
re-entrancy; A-ZV2 sospesa con giudizio). Il punto 3 com'è scritto riapre
il canale che il move aveva chiuso, con un atteso preso a prestito da un
meccanismo che non usa.

## Emendamenti

- **A-MA-103-1**: nominare INV-RECV-1 nel codice (commento sui due siti) +
  tabella di audit dei ≥6 observer assoluti × raggiungibilità mid-arm;
  correggere la motivazione registrata del criterio.
- **A-MA-103-2**: debug_assert/dente: nessun `Zval::Ref` raggiunge
  `prop_get_fallback`/write come `target`.
- **A-MA-103-3**: fixture 14 (initializer lazy sincrono che droppa l'ultima
  ref esterna + `gc_collect_cycles()` + ordine `__destruct`, 2 modi, diff
  oracle esatto) + variante PropSet con soglia gc attraversata mid-arm.
- **A-MA-103-4**: riscrivere il punto 3: dichiarare il MECCANISMO; se
  clone-from-slot, controfattuale = SOLO traffico pila, ring-fenced dal
  punto 2 (mai le stesse ns due volte).

## Kill-switch

- **KS-MA-103-1**: forma slot-operando che borrowa lo slot ricevitore =
  reject, salvo sigillo A-MA-102-3 + prova per-path che né PHP né observer
  corrono nel borrow-window (il fallback fallisce per costruzione).
- **KS-MA-103-2**: estensione del move a Silent/Dynamic/OpSet/IncDec senza
  occorrenza contata per-forma E senza la guardia Ref = reject.
- **KS-MA-103-3**: qualunque cambio che ribalti il verdetto di un sito
  exact-count (`==1/==2/−2`) nelle fixture = reject senza appello
  (estende KS-MA-102-3 dall'ordine distruttori ai verdetti degli observer).
