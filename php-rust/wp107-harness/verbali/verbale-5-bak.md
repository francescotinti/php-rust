# Verbale sedia 5 — Bak (VM engineering: alloc-rate, path caldi, dispatch)
## Concilio WP-107 su S-105 e programma S-106

**VERDETTO: la leva forma 2 è promossa a norma di criterio e non la tocco;
REFUTO la Scoperta 4 del report (claim di copertura) e il perimetro del
prossimo bersaglio come oggi argomentato. Una refutazione capitale.**

### R-BA-107-1 (CAPITALE) — «il fast path copre la maggioranza del carico reale» è NON DIMOSTRATO
Il 73,1% ≤2 è misurato al choke-point `bind_params` (G2), ma il predicato
del fast path promosso è un ALTRO insieme: `Op::Call` ∧ `simple_call` ∧
`n == n_params` ESATTO. Tre esclusioni mai censite: (1) i BUILTIN non
passano MAI da bind_params (`pop_keys` → Vec, run.rs:2705 e affini) —
volume assente da OGNI census; (2) metodi/closure/CallValue/CallNsFallback
arrivano a bind_params ma NON al braccio fast (MethodCall run.rs:4482 fa
`pop_keys` → Vec; CallValue run.rs:1854 idem); (3) `simple_call` esclude
hint, by-ref, variadic, generator (func.rs:119) e l'arità ESATTA esclude
ogni chiamata che usa un default — in WordPress i parametri opzionali sono
la norma, quindi una fetta ignota di a1=33,1% cade sul sentiero Vec.
In più: la forma 2 promossa è ARITÀ-INDIPENDENTE (il loop `(0..n).rev()`
serve n=7 come n=1) — il censimento arità e il «inline-2 ben dimensionato»
appartenevano alla forma 1, che è MORTA. Citarli a sostegno della forma 2
è un trapianto di evidenza. **Copertura = contatore hit/miss, non
istogramma d'arità.**

### R-BA-107-2 — a4=15,6% > a3=7,7%: anomalia non investigata
Una distribuzione con la gobba a 4 non è rumore: indizia pochi siti caldi
4-ari (candidato ovvio: la filiera hook di WP). Se il prossimo sito si
nomina senza spiegare quella gobba, si nomina alla cieca.

### R-BA-107-3 — «mai contenitori nel call path» generalizza oltre la misura
Il fatto misurato è: SmallVec 40 B passato PER VALORE attraverso il
confine non-inlined di bind_params + IntoIter con check spilled costa ~37
ns/iter più del Vec heap; la coppia alloc+free del Vec vale ~9. Il Vec di
CallValue/CallNsFallback NON è quindi imputato: toccarlo rende al massimo
~9 ns/iter × un volume MAI misurato. Verdetti sui contenitori = per forma
E per confine, non per categoria.

### R-BA-107-4 — prop-RMW al 2° slot con prerequisito INSOLUTO
La baseline stessa scrive «contatori = prerequisito di tesi» per H-C3
prop; il braccio contatori (punto 2, WP-106) NON è entrato in S-105.
Aprire prop-RMW in S-106 senza contatori è aprire su premessa non firmata.

### A-BA-107-1 — contatore di copertura PRIMA di nominare il prossimo sito
Apparato ~30′ census-gated: (a) hit/miss al branch fast di Op::Call;
(b) contatore per-chiamante sui 12 chiamanti di bind_params; (c) volume
`pop_keys` builtin — sul campione wptests G2. Il prossimo sito
(method-call fast path vs builtin args) si nomina DAI NUMERI; la mia
attesa pre-registrabile: dominano MethodCall+builtin, non CallValue.

### A-BA-107-2 — srotolamento per-arità n=0,1,2: RESPINTO
Il corpo del loop è pop+decay; srotolare risparmia il controllo di loop e
paga in volume di codice — esattamente la classe che S-104 ha refutato
(valuta = volume, non micro-costi). Stessa famiglia del threaded-dispatch
(A-BA-106-3). Ammissibile SOLO con run_loop non-crescente all'admission,
cioè mai.

### A-BA-107-3 — S-106: prop SOLO col braccio contatori eseguito PRIMA
(~30′, checkout 4ea2cff); se il braccio non entra, il 2° slot passa ad
arith (fusioni registro H-C3, stessa famiglia, senza quel prerequisito).
Il rapporto più alto non alloca da sé: l'obiettivo X≤3× esige TUTTE le
categorie, l'ordine è solo resa/sessione — e la resa si stima col
contatore di A-BA-107-1, non col desiderio.

### KS-BA-107-1
Nessun claim di copertura di un fast path senza contatore hit/miss al
branch: un istogramma d'arità non è un census di copertura.

### KS-BA-107-2
Specializzazioni per-arità (srotolamenti) inammissibili salvo taglia di
run_loop non-crescente all'admission-disasm.

### KS-BA-107-3
I verdetti sui contenitori si registrano per forma-e-confine; vietato
estenderli per categoria («mai Vec») senza misura del sito.
