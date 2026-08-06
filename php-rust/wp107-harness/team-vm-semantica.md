# Team VM-SEMANTICA — Fase 2 Concilio WP-107

**Team**: VM-semantica e fedeltà · **Membri**: Stogov (relatore), Hoare, Matsakis
**Data**: 2026-08-07 · **Fonti**: verbale-8-stogov, verbale-1-hoare, verbale-2-matsakis
(+ indice COUNCIL_WP107_REVIEWS per il quadro)

## 1. Composizione delle posizioni

### Convergenze (ID canonico dichiarato)

- **Forma 2 sana, promozione regge**: le tre sedie concordano — Hoare (predicato
  identico al braccio fast, nessuno slot Undef), Matsakis (ownership pulita per
  costruzione, D-R11 preservato, canale (a) CHIUSO), Stogov (coerente con Zend,
  ma «a metà» — vedi debito). Nessuna refutazione capitale sulla LEVA.
- **Doppia copia della semantica di bind**: R-HO-107-4 ≡ **KS-ST-107-2** (canonico).
  Predicato in tre siti (run.rs:2593, calls.rs:324, calls.rs:1135), `simple_call`
  duplicato testualmente (func.rs:119/:281), corpo decay duplicato
  (run.rs:2606-2609 ↔ braccio fast di bind_params). Cura durevole = A-HO-107-4.
- **Ordine di decay**: R-ST-107-2 concorda esplicitamente con **A-MA-107-1**
  (canonico); la concessione di Hoare è CONDIZIONATA (KS-HO-107-2). Lo standard
  del dente è «mai user code nel decay» (formulazione Stogov), non genericamente
  «pure-read».
- **Sigillo Copy**: R-HO-107-1 (capitale) con cura **A-HO-107-1 ≡ A-KL-107-2**
  (canonico A-HO-107-1). Il team fissa il CRITERIO di adeguatezza (T-VM-107-4).
- **§3.15**: R-ST-107-3 + A-ST-107-2 ≡ **A-MA-107-4** sullo stesso sito
  (expr.rs:1615-16, stessa riga citata con off-by-one — nessun conflitto reale).
  Le due cure sono COMPLEMENTARI: Matsakis dà il veicolo (specchiare il binder
  dinamico, mod.rs:11582-84, in push_call_args + ramo named expr.rs:1417),
  Stogov la semantica Zend-esatta (Error/MakeRef ≥ vslot, gamba dinamica fx21
  senza biforcare) e il dovere di citare i fail (KS-ST-107-1, attesa 1417→1415).
- **Debito «metà-Zend»**: R-ST-107-1 + A-ST-107-1, nessuna obiezione altrui:
  il doppio transito push→pop resta (2 move/arg vs 1 write/arg Zend); ritirarlo
  = frame-in-costruzione alla Zend = leva futura con protocollo pieno.

### Conflitti risolti

- **Hoare vs Matsakis sull'osservabilità dell'ordine inverso**. Hoare: «inosservabile
  anche per f($a,$a)». Matsakis: l'ordine dei decrementi Rc è esattamente ciò che
  recycle_frame chiama "the only PHP-observable effect" (riuso handle-id, cascate
  GC) — asserito, non provato. **Risoluzione: vince Matsakis.** La concessione di
  Hoare era già condizionata; per KS-MA-107-1 un claim d'ordine senza prova o
  dente è VOID come premessa. Serve prova per NOME (operando di Op::Call mai
  last-ref) O dente, O declassamento del commento (T-VM-107-2).
- **Collocazione §3.15**: nessun conflitto sostanziale — Matsakis la vuole «fuori
  finestra leva: coda fedeltà», Stogov la mette PRIMA dentro la coda fedeltà per
  costo/impatto. Le due tesi compongono: resta al punto 7, ma in TESTA al
  timebox e non negoziabile (T-VM-107-8).

### Resta aperto

- Prova per NOME che un operando di Op::Call non può essere un Ref last-ref
  (chiude T-VM-107-2 senza dente).
- Quota di bind ELEGGIBILI alla forma 2 (A-HO-107-5; converge col capitale di
  Bak sul 73,1% — misura di competenza team-metodo/contatori, qui solo il
  vincolo: nessuna estensione senza quella cifra).
- Leva frame-in-costruzione (metà-Zend): nominata, NON schedulata.

## 2. Direttive composte

### (a) VINCOLANTI per l'ordine S-106

1. **T-VM-107-1** — Finché il predicato/decay non è unificato (A-HO-107-4:
   helper unico `Func::takes_fast_bind(n)`, `simple_call` calcolato in UN punto),
   ogni modifica a UNA sola copia ⇒ fx21 VOID; ogni allargamento di `simple_call`
   senza dente differenziale + fx21 = leva VOID. [R-HO-107-4, KS-ST-107-2, KS-HO-107-1]
2. **T-VM-107-2** — Ordine-di-drop: o prova per NOME, o dente (due temp
   Ref/oggetto last-ref, arbitro = riuso handle-id/cascata), o il commento
   run.rs:2599-2600 si declassa a «ordine non provato osservabile». Standard
   permanente: mai user code nel decay. [A-MA-107-1, R-MA-107-1, R-ST-107-2,
   KS-MA-107-1, KS-HO-107-2]
3. **T-VM-107-3** — Dente VM sul braccio direct-bind: fixture negative — chiamate
   ad arità esatta verso hinted/by-ref/variadic/generator DEVONO prendere il
   sentiero generico — più `f($a,$a)` con ref sul fast. [A-HO-107-3, KS-HO-107-1]
4. **T-VM-107-4** — Criterio di adeguatezza del sigillo Copy riscritto: pinna il
   payload AL costruttore di variante (`const fn s<T: Copy>(_: fn(T)->Zval)`;
   `s(Zval::Bool)` …) e la mutazione nominata «Bool → boxed» NON deve compilare.
   Un sigillo che il proprio controesempio nominato attraversa è VOID.
   [R-HO-107-1 capitale, A-HO-107-1 ≡ A-KL-107-2]
5. **T-VM-107-5** — Backstop ArgPlace rumoroso (debug_assert/contatore census
   «funnel mancato» al posto del Null silenzioso, calls.rs:303); nessun
   direct-bind su CallValue/CallNsFallback senza check di materializzazione
   PRIMA del bind e senza la quota di eleggibilità misurata. [A-MA-107-2,
   KS-MA-107-2, R-MA-107-2, A-HO-107-5]
6. **T-VM-107-6** — Cura §3.15 composta: compiler-side, specchia la logica del
   binder dinamico in push_call_args E ramo named; semantica Zend-esatta ≥ vslot
   (letterale ⇒ Error runtime, arbitro by_ref_error.phpt; place ⇒ MakeRef);
   fx21 gamba dinamica senza biforcare i sentieri; gate ORM/hk; il fix CITA i
   fail da flippare, attesa 1417→1415. [R-ST-107-3, A-ST-107-2, A-MA-107-4,
   KS-ST-107-1]
7. **T-VM-107-7** — Il debito «metà-Zend» (doppio transito push→pop) è leva
   futura NOMINATA: frame-in-costruzione alla Zend con cleanup su eccezione a
   metà SEND e rientranza; MAI eseguita come «rifinitura» — protocollo pieno
   + fx21. [A-ST-107-1, R-ST-107-1]

### (b) Raccomandazioni

8. **T-VM-107-8** — Se la leva del punto 5 cade con early-stop (KS-GR-107-1),
   la finestra liberata va a §3.15: è la voce fedeltà a costo minimo con
   dividendo nominato (−2 fail). [A-ST-107-3, A-MA-107-4]
9. **T-VM-107-9** — Riscrivere la doc zval.rs:258-264: «canale refutato PER
   MISURA (Δ negativo 5/5); icache-bound = ipotesi N=1 NON firmata,
   KS-BA-106-1» — da coordinare con team-metodo. [R-HO-107-2, A-HO-107-2]
10. **T-VM-107-10** — Generalizzare KS-ST-107-1 oltre §3.15: ogni voce nuova a
    catalogo si cerca PRIMA nel fail-set congelato; già recepito in
    NEXT_SESSION, la SYNTHESIS lo confermi come regola permanente. [R-ST-107-4]

## 3. Posizione sull'ordine provvisorio §S-106 (punti 1-7)

**Il team NON chiede stravolgimenti. La fedeltà §3.15 al punto 7 è collocata
BENE** (Matsakis: fuori finestra leva; Stogov: prima DENTRO la coda fedeltà —
già recepito nel testo del punto 7). Modifiche puntuali chieste:

- **Punto 4 (igiene del pin)**: aggiungere il backstop ArgPlace rumoroso
  (T-VM-107-5, parte assert/census — poche righe) e, SE il timebox regge,
  l'unificazione A-HO-107-4; altrimenti l'unificazione resta aperta e vige
  T-VM-107-1. La voce «dente ordine-di-drop (A-MA-107-1)» va letta come
  «dente O declassamento del commento» (T-VM-107-2).
- **Punto 6 (denti nella finestra)**: aggiungere il dente VM del braccio
  direct-bind (T-VM-107-3) accanto a OBS-8, fx20 e hit/miss.
- **Punto 7**: confermato con l'ordine Stogov (§3.15 > get_gc > §3.13 >
  §3.12-i > §3.14); §3.15 è la TESTA non negoziabile del timebox e assorbe
  l'eventuale finestra liberata da early-stop della leva (T-VM-107-8).
