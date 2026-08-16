# COUNCIL_S146_REVIEWS — concilio a 9 su B3/filone conteggi (KS-B4)
# Convocazione: s146-concilio-fascicolo.md · Verbali INTEGRALI (fonte VINCOLANTE) + note di team · Sintesi: concilio/sintesi.md


════════════════════════════════════════
# Sedia HOARE — Concilio S-146 (B3/filone conteggi)
Lente: design linguaggio/runtime Rust, SAFE-ONLY, sigilli di tipo (VmGate ZST, assert 16B/niche, unsafe solo nel crate types — ZStr S-124).

## VERDETTO: CONCORDO CON EMENDAMENTI

Fatto aritmetico che il fascicolo NON trae dalla sua stessa sonda: **TakeSlot non rimuove il pavimento**. Un take è ancora un movimento — in safe Rust è `mem::replace(slot, Undef)`: copia 16 B e scrive Undef; elimina solo inc-dec (0,21 s) e nota (0,25 s), e solo sulla frazione take-abile. Il 69,5% memcpy resta intatto. Chi rimuove il pavimento è il **borrow-through** (precedenti SPEDITI HC1/L-FR1): il valore non si materializza affatto. Dai prezzi firmati: borrow ≈ −2,9…−3,9 ns per movimento EVITATO; take ≈ −0,5…−1,0 per movimento preso. «Muovere MENO» = borrow prima, take poi.

## Posizioni a–e

**a) forma d'emissione — CONCORDO CON EMENDAMENTI.** Il flag su LoadSlot è esprimibile senza corpo caldo nuovo: campo nel payload dell'op, branch dentro il braccio esistente, per-sito (BTB); sfugge al prerequisito O1 (che vale per corpi NUOVI). MA un campo aggiunto a una variante NON fa scattare il sigillo dei match esaustivi (una variante nuova sì): serve R1+R2. Restano dovuti nm -S predetta (A-LB-97-1) e disasm bl-count (criterio-B p.4).

**b) perimetro — CONCORDO.** Perimetro fedele di Stogov (nucleo-stringhe, niente identità). Dalla mia lente il guard di tipo è indipendente dal perimetro statico: match sulla variante, `Ref` ⇒ fallback clone. safe_ref 0,013% lo rende quasi gratis e MAI superfluo (recount S-96: la correttezza non si misura in frequenza). Il move safe non tocca la repr: le assert 16/8/niche restano intatte per costruzione, zero unsafe nuovo.

**c) censimento F1 su ORM — SÌ ma CONDIZIONATO:** serve solo se TakeSlot supera l'istruttoria borrow. Prima un census dei SITI borrow-abili su ORM (pattern-census classe FR1, zero liveness): più economico, rischio zero.

**d) alternative — CON EMENDAMENTO D'ORDINE (R4).** FR1-ext (chiave da slot, FieldRead/isset) è la leva PRIMA: semanticamente invisibile, zero liveness, e il borrow checker È il sigillo — un borrow tenuto attraverso una chiamata che può mutare il frame NON COMPILA; safe Rust fa da giudice statico gratis. Arena-conteggi: MAI definita; onere di definizione ≤1 pagina col costo sostitutivo, altrimenti ARCHIVIATA (come nominata collide coi veti alloc-removal/contenitori sul call path).

**e) cosa compra — CONCORDO, con cifra più severa.** Take modellato ≤ ~0,46 s × frazione take-abile (solo incdec+nota); borrow aggredisce anche la quota memcpy dei movimenti evitati, ma sempre DENTRO l'1,52 s modellato; ~4,4 s glue fuori modello: nessun claim oltre la risoluzione.

## Emendamenti

- **R1** — `SlotMode` enum {Copy, Take}, non bool: match esaustivi ⇒ un futuro Borrow non compila in silenzio. Misura: compile-fail test per nome.
- **R2** — `Take` costruibile SOLO con token ZST rilasciato dal modulo liveness (pub(crate); precedente VmGate): un'emissione take senza analisi non compila. Misura: compile-fail per nome.
- **R3** — mutation-check del guard: il mutante senza guard DEVE morire su fixture nominata Ref-in-slot-safe (lezione WP-104: il mutante si fa sull'arbitro del rischio).
- **R4** — ordine istruttoria: 1) FR1-ext borrow (census siti ORM + criterio ≤10 righe); 2) F1-liveness SU ORM solo se resta residuo che paga; 3) TakeSlot forma-flag con R1–R3; 4) arena archiviata salvo definizione.
- **R5** — sentinella dinamica read-after-take nel build census (verifica a macchina della liveness statica) PRIMA di ogni emissione.

## Kill-switch pre-registrabili

- **KS-H1**: read-after-take > 0 su corpus 1414×2 o ORM ⇒ STOP emissione take.
- **KS-H2**: mutante-guard sopravvive ⇒ giudice inesistente ⇒ STOP fetta.
- **KS-H3**: bl-count run_loop aumenta o nm -S oltre il predetto ⇒ forma respinta (Δ bracci caldi ≤ 0).
- **KS-H4**: census siti borrow su ORM sotto soglia REGOLE §3 (max(4 ns, rumore, banda)) ⇒ FR1-ext non si apre, si passa al punto 2 di R4.
- Vigente e citato: fail NUOVO per NOME in weakrefs/destructor ⇒ STOP.

## Mandato inverso (Gregg)

Oggi sappiamo che il collo è il pavimento move (2,88 ns × 367,6M) e che take NON lo rimuove — ieri «filone conteggi» era sinonimo di TakeSlot. E i would_take esistono solo su media-WP: su ORM sono IGNOTI.


════════════════════════════════════════
# Verbale sedia MATSAKIS — Concilio S-146 (bozza indipendente)
Lente: ownership/aliasing/borrow — soundness liveness, borrow-first, trappole di aliasing (Ref condivisi, generatori sospesi, closure by-ref, catch/finally).

**VERDETTO: CONCORDO CON EMENDAMENTI.** La leva prima del filone conteggi NON è TakeSlot: è l'estensione borrow-first. Motivo di lente: KS-B4 dice che il collo è il pavimento move (memcpy 69,5% = 1,06 s; 2,88 ns × 367,6M). TakeSlot NON muove di meno — esegue lo stesso memcpy e in più scrive Undef nello slot; ciò che elimina è la coppia inc/dec, cioè la fetta inc-dec 14,1% (0,21 s). Il borrow-first (HC1 S-140, L-FR1 S-145) elimina il MOVIMENTO: opera su `&Zval`, zero analisi di liveness, zero falsi-morti per costruzione, semanticamente invisibile. La leva coerente col verdetto della sonda è quella.

## Posizioni a–e

**a) CON EMENDAMENTI.** La forma-flag (`take` deciso a compilazione dentro LoadSlot) è l'UNICA ammissibile se TakeSlot mai si apre: nessun corpo caldo nuovo, tetto A-LB-97-1 con taglia `nm -S` predetta PRIMA. Ma resta subordinata (R1) e col guard di tipo su Ref a runtime non negoziabile: safe_ref 0,013% — quasi mai preso, MAI superfluo, la correttezza non si misura in frequenza.

**b) CONCORDO con Stogov, rafforzando.** I predicati di rinuncia di design95 (compact, eval, `$$x`, generatori, closure by-ref, catch/finally, distruttori…) sono una cura ENUMERABILE contro un attacco NON enumerabile: vacua per costruzione (S-96). L'inversione che la sana: enumerare il SAFE, non l'unsafe — take legale solo dove una grammatica chiusa lo prova (stesso basic block, nessuna call/eval/accesso dinamico allo scope tra lettura e kill, nessuna regione protetta attraversata); tutto il resto ⇒ clone, fail-safe. Perimetro fedele = nucleo senza identità: morte anticipata inosservabile per SEMANTICA del valore, non per fortuna dell'enumerazione. Nota di conto: sugli scalari take non compra nulla (niente rc); il comprabile è solo il nucleo str (18,7% media-group, banda MEDIA).

**c) CON EMENDAMENTO.** Un census F1 su ORM serve SOLO se si istruisce TakeSlot — i numeri media-group (47,1% / 90,2%) NON si trasferiscono. Ma il census giusto per la leva giusta è un altro: **borrow-census su ORM** (R2) — per sito consumatore, quanti dei 367,6M movimenti sono letture pure through-borrow-abili.

**d) CONCORDO.** Famiglia FR1-ext PRIMA (chiave da SLOT `$o->d[$k]`, FieldRead/isset — già aperture per NOME). «Arena-conteggi»: MAI definita ⇒ si ARCHIVIA per nome; qualunque cosa somigli a un'arena collide col veto «alloc-removal senza costo sostitutivo» finché nessuno ne nomina il meccanismo.

**e) CONCORDO** (il mio R4 S-143 resta): la scommessa compra la TAPPA, non la parità; perimetro modellato 1,52 s su 37,6 s; dentro quello, il comprabile di TakeSlot è ≤0,21 s (inc-dec) — nessun claim oltre la risoluzione.

## Emendamenti
- **R1 (ordine)**: istruttoria = 1) borrow-census ORM; 2) fette FR1-ext con criteri propri ≤10 righe; 3) TakeSlot solo dopo, e solo se inc-dec riemerge nella partizione post-fette. Perché: allineare la leva al meccanismo che KS-B4 ha firmato. Misura: ripartizione churn rieseguita a fette spedite.
- **R2 (borrow-census)**: monobinario census, ×2 repliche r1==r2 ≤1%, conteggi per sito consumatore; è il prerequisito della prima fetta, non F1-liveness.
- **R3 (allowlist)**: ogni futura analisi take = grammatica SAFE chiusa; una renounce-list come fondamento è vietata per nome (S-96).
- **R4 (poison-Undef)**: build di collaudo dove lo slot preso diventa variante Poison che PANICA alla lettura — il falso-morto silenzioso diventa fail rumoroso su batteria+corpus. Grado: gate di collaudo, MAI nel pin.

## Kill-switch pre-registrabili
- **KS-M1**: due fette borrow-first spedite senza calo famiglia churn+memops E coppia ORM in banda ±0,7% ⇒ filone ridimensionato (si applica il più severo tra KS-B1/KS-B2).
- **KS-M2**: borrow-census con quota through-borrow-abile <20% dei movimenti ⇒ FR1-ext non si estende oltre le aperture già nominate.
- **KS-M3**: TakeSlot NON si apre finché inc-dec ≤20% del churn ripartito (coerenza KS-B4).
- **KS-M4**: fail NUOVO per NOME in weakrefs/destructor/generatori ⇒ STOP fetta (invariato).

**Mandato inverso**: ieri non sapevamo che il prezzo per-movimento è pavimento-move-dominato; oggi sappiamo che l'unica mossa che lo aggredisce è NON muovere (borrow), non muovere-più-a-buon-mercato (take).


════════════════════════════════════════
# Verbale S-146 — sedia KLABNIK (lente: chiarezza, spec dei gate, testabilità)

## VERDETTO: CON EMENDAMENTI (quasi-opposizione su TakeSlot come default)

Il fatto che decide tutto è NELLA sonda stessa: TakeSlot **non riduce i
movimenti** — sposta invece di copiare, ma il memcpy (69,5%, 1,06 s) resta
pagato. Ciò che TakeSlot compra è la coppia inc-dec: **0,21 s a frazione
100%**. La banda del giudice della scommessa (coppia ORM ±0,7%) vale ~0,3 s.
Il tetto della leva è SOTTO la risoluzione del suo giudice **per costruzione,
con i numeri già firmati**: non serve alcun censimento per saperlo. Anche
nella lettura larga (inc-dec+nota = 0,46 s, ammesso che il take eviti la
nota — non provato) il margine è 1,5× la banda, sotto la mia soglia di
distinguibilità 2×. Si applica la lettura severa (precedente S-144 memops).

## Posizioni a–e

**a) Forma d'emissione — CONCORDO con riserva.** Vero che il flag `take`
compilato non è un corpo caldo in più (design96 #1) e sta sotto il tetto
A-LB-97-1; ma la forma resta ACCADEMICA finché il tetto 0,21 s non è
sfondato da un censimento ORM firmato. Non si istruisce una forma per una
leva pre-uccisa. Per le leve borrow-first la forma è quella già spedita
(peephole in place, fallback per costruzione — L-FR1); disasm bl-count
obbligatorio su ogni tocco a run_loop (criterio-B p.4, resta).

**b) Perimetro semantico — CONCORDO.** Il vincolo Stogov (CV non consumati,
morte anticipata osservabile) morde solo le leve che muovono la fine-vita.
Le borrow-first sono semanticamente invisibili per costruzione (zero
liveness): il perimetro fedele è gratis. Fixture weakrefs/destructor a gate
restano.

**c) Censimento F1 su ORM — MI OPPONGO, ora.** Un censimento si deve a una
regola che ne ha bisogno; la regola qui decide già sui numeri S-145. Ciò che
serve invece è un **census SITI-CONSUMATORI su ORM** (monobinario, r1==r2,
criterio ≤10 righe firmato PRIMA): movimenti per sito × prezzo per classe
(2,88–3,85 ns) = guadagno modellato per fetta, ordinati.

**d) Alternative — CONCORDO: la leva che si istruisce è FR1-ext /
borrow-first** (chiave da SLOT, FieldRead/isset, siti consumatori): è
l'unica che elimina il MOVIMENTO, cioè il pavimento 69,5%. Due precedenti
spediti, zero liveness. **Arena-conteggi: si ARCHIVIA** — una leva senza
definizione non può avere criterio né giudice; riapribile solo per NOME con
design ≤1 pagina (perimetro semantico + giudice nominato).

**e) CON EMENDAMENTI.** Il perimetro modellato è 1,52 s su 37,6 s: B3 al
massimo teorico compra ~4% del gap — **non compra neanche la tappa ≤3× da
solo**; è un addendo. La glue ~4,4 s resta fuori modello: nessun claim.

## Emendamenti

**R1 (regola pre-registrata, PRIMA del census siti):** una fetta si apre se
guadagno modellato ≥ max(4 ns/iter sul SUO micro nominato, banda); una fetta
suite-judged pretende tetto modellato ≥ 0,6 s (2× banda coppia). Sotto:
micro-judged con suite a sola guardia (keep-partial-wins).
**R2:** il moltiplicatore 4,5–6,5% (SCREEN R=1) è MORTO per B3: non si
promuove, si SOSTITUISCE coi prezzi sonda-B. Ogni banda derivata da lì
(design95 §P1, righe guadagno_* del recount) è NON CITABILE nei criteri B3.
**R3 (KS riscritti):** KS-B1 («−25% churn+memops») era tarato sul programma
B1/B2: per B3 la predizione è la somma dei guadagni modellati spediti.
KS-B2 (4 sessioni, più severo con Gregg-5) resta, ma l'esito è **stop
famiglia + riconvoca**, non revert delle leve che tengono le proprie guardie.

## Kill-switch pre-registrabili

- **KS-B3-K1:** tetto modellato < soglia R1 o micro non nominato ⇒ fetta
  non si apre.
- **KS-B3-K2:** TakeSlot chiuso per tetto (0,21 s < 0,3 s a frazione 1);
  riapribile solo se un censimento ORM firmato mostra componente
  acquistabile ≥ 0,6 s.
- **KS-B3-K3:** guadagni modellati spediti ≥ 0,5 s con coppia ORM in banda
  ±0,7% ⇒ modello falsificato ⇒ stop famiglia, riconvoca.
- **KS-B3-K4:** 4 sessioni B3 con ORM fermo ⇒ stop famiglia + riconvoca.
- Gate semantici invariati; fail NUOVO per NOME in weakrefs/destructor ⇒
  STOP fetta.


════════════════════════════════════════
# Verbale HEJLSBERG — Concilio S-146, quesito B3/filone conteggi

Sedia: ingegneria dei compilatori (forma d'emissione, dataflow a compilazione,
costo per-sito). Bozza INDIPENDENTE; fascicolo letto per intero; nessun .rs
aperto (finestra di misura attiva).

## VERDETTO

**CONCORDO CON EMENDAMENTI** sul quesito; l'ordine d'istruttoria che propongo
INVERTE la priorità implicita («TakeSlot S-140 prima»): la sonda stessa dice
che il collo è il memcpy (69,5%), e il take NON elide il memcpy — lo elide
solo il borrow.

## Posizioni a–e

**a) CON EMENDAMENTI.** La forma-flag (bit `take` su LoadSlot, deciso a
compilazione) è ammissibile sotto il tetto WP-39..44: nessun corpo caldo
nuovo, branch per-sito dentro il braccio esistente (RC-1), BTB-predetto. Non è
gratis: si istruisce con (1) taglia `nm -S` di run_loop PREDETTA prima;
(2) disasm bl-count prima/dopo (lezione H-C2); (3) flag=0 ⇒ path byte-identico
al corrente (fallback per costruzione, precedente L-FR1); (4) banda-layout
pre-registrata. O1-outlining NON è prerequisito di QUESTA forma (era condizione
sui corpi NUOVI); ma taglia oltre predizione ⇒ stop.

**b) CONCORDO** con Stogov: per ogni take il perimetro fedele è il nucleo
senza identità (str, scalari). Rilievo di lente: la classe
borrow/through-borrow non ha bisogno di ALCUN perimetro liveness — non muove,
non anticipa morti, semanticamente invisibile (HC1 S-140 e L-FR1 S-145,
entrambe spedite).

**c) MI OPPONGO com'è posta.** Il primo censimento dovuto su ORM non è
F1-liveness ma il censimento dei DIGRAMMI (coppie `LoadSlot;CallArg` /
`;StoreSlot` / `;Dim` / `;Binary`, conteggi monobinari ×2, r1==r2): le fette
peephole si prezzano lì. F1-liveness su ORM SOLO se la forma-flag sopravvive
al gate prezzo (R2) — i tassi del media group WP (would_take_rc 47,1%) non si
trasferiscono a ORM per fede.

**d) CONCORDO con inversione d'ordine**: borrow/through-borrow PRIMA di ogni
take, perché elide il movimento intero, memcpy compreso — l'unica classe
coerente con KS-B4. **Arena-conteggi: ARCHIVIARE** — mai definita; ogni
lettura sensata ricade nei veti confermati 9/9 (alloc-removal senza costo
sostitutivo, contenitori sul call path).

**e) CON EMENDAMENTI.** B3 compra al massimo 1,52 s modellati (~3–4% del
tempo phpr; ~0,3× degli 8,6×) — nessun claim su parità né sui ~4,4 s di glue
fuori modello; coerente col mio «residui ≈6,5× a B completa». KS-B1 (−25%
churn+memops) è dimensionato su B1/B2: ereditato tal quale, B3 nasce
falsificata anche in pieno successo. La scommessa va ri-registrata alla scala
del perimetro modellato.

## Emendamenti

- **R1 (refutazione centrale)**: take converte clone→move ma il memcpy dei
  16 B e la store di Undef RESTANO; compra solo inc-dec+nota+glue di drop
  (≤~31% del prezzo pair, dalla partizione). La sonda deve prezzare
  «move+undef» vs «clone+drop» per classe PRIMA di aprire il flag.
- **R2**: gate prezzo pre-registrato per la forma-flag: risparmio/mov ×
  conteggio indirizzabile (dal censimento c) ≥ max(4 ns-equiv., rumore,
  banda) — sotto soglia, il flag non si scrive.
- **R3**: l'ordine per moltiplicatore PER-TIPO (scalar 91,1M · str 104,1M ·
  arr 60,9M · rc 111,5M) è inattuabile a compilazione: i tipi sono dinamici,
  l'emissione vede solo PATTERN. Ordine = contributo assoluto per pattern
  (conteggio×prezzo); il per-tipo resta tie-break osservativo
  (str ≈0,40 s ≈ rc 0,37 > scalar 0,26 ≈ arr 0,23).
- **R4**: ogni fetta peephole eredita il protocollo L-FR1 per nome: criterio
  ≤10 righe, R=5 ABAB, disasm agli atti, guardie con giudice DENTRO lo
  script (az.rev. S-145 #3).

## Kill-switch pre-registrabili

- **KS-H1**: bl-count run_loop post>pre, o taglia `nm -S` oltre predizione ⇒
  stop fetta.
- **KS-H2**: censimento digrammi ORM senza alcun pattern sopra il gate R2 ⇒
  B3 chiusa SENZA codice.
- **KS-H3**: fail NUOVO per NOME in weakrefs/destructor ⇒ STOP (invariato).
- **KS-H4**: scommessa B3 riscalata; micro churn 5/5 ma ORM fermo oltre
  l'orizzonte più severo (Klabnik/Gregg) ⇒ ridimensionare, non estendere.

## Mandato inverso

Oggi sappiamo che il prezzo per-movimento è PAVIMENTO (2,88 ns) e non
contatori; ieri non sapevamo che il 69,5% sta nel move stesso. Questo
retrocede il take (ieri candidato naturale) e promuove il borrow — che il
take non sostituisce.


════════════════════════════════════════
# Sedia BAK — Concilio S-146 — quesito B3/filone conteggi

Lente: VM hot-path (alloc-rate, icache/BTB, code-cache); miei il tetto WP-39..44, O1, A-LB-97-1. Bozza indipendente; mandato: refutare.

## VERDETTO: CON EMENDAMENTI

L'aritmetica della sonda REFUTA TakeSlot come prima leva del filone: KS-B4 dice che il collo è il pavimento (memcpy 1,06 s), e TakeSlot NON muove meno — muove più a buon mercato. Un take è ancora un movimento (dispatch + copia 16 B + scrittura Undef): elide inc/dec e al più la nota, cioè pesca SOLO nelle fette incdec 0,21 s + nota 0,25 s = 0,46 s massimi teorici (al 100% di eleggibilità, irrealistico: would_take_rc 47,1% su media-WP). Realisticamente ~0,2 s, cioè SOTTO la banda del giudice della scommessa (±0,7% coppia ORM ≈ ~0,3 s, derivata). Una leva il cui massimo modellato sta sotto la risoluzione del proprio giudice non è istruibile come scommessa suite.

## Posizione a–e

**a) CON EMENDAMENTI.** Due sub-forme, pedaggi diversi, il criterio deve nominarne UNA: (i) bit `take` dentro il braccio LoadSlot esistente = Δ corpi caldi 0 ⇒ A-LB-97-1 soddisfatto per costruzione e **O1 NON è prerequisito** (rispondo al mandato: O1 era prerequisito per «ogni corpo caldo IN PIÙ»; qui non ce n'è); il vincolo però NON evapora — si trasforma in vincolo di TAGLIA: il flag allarga il braccio (branch+Undef-write+skip-inc, stimo +16–48 B) e può spingerlo oltre la soglia d'inlining, che è il modo esatto in cui è caduta H-C2 (inliner flippato, bl 1101→0, WP-104). (ii) opcode-variante `LoadSlotTake` = braccio nuovo, target BTB nuovo ⇒ tetto pieno + O1 prerequisito. Nessuna delle due si istruisce senza nm -S PREDETTA prima.

**b) CONCORDO** (Stogov S-96): perimetro fedele = solo nucleo senza identità; guard di tipo su Ref obbligatorio — ramo quasi mai preso (safe_ref 0,013%), ben predetto dal BTB, quindi economico, MAI superfluo.

**c) CONCORDO, anzi lo esigo:** i conteggi S-95/96 sono media-WP; il giudice è ORM. Senza censimento F1-ORM con classi ALLINEATE alla partizione sonda (per derivare il guadagno in SECONDI = conteggio × prezzo firmato, non in %CPU screen) nessuna forma si apre.

**d) CON EMENDAMENTI — è il cuore:** la famiglia borrow-first/through-borrow ai siti consumatori è l'UNICA che comprime il CONTEGGIO e quindi l'unica coerente con KS-B4: attacca il pavimento 1,06 s più incdec+nota dei movimenti eliminati; due precedenti SPEDITI (HC1, L-FR1 −28%) con zero liveness. Va istruita PRIMA; TakeSlot retrocede (R2). Arena-conteggi: mai definita, non riduce movimenti per alcuna definizione nota, e i veti alloc (quota_obj 2,4%, costo sostitutivo) mordono ⇒ ARCHIVIARE salvo definizione scritta che riduca movimenti.

**e) CONCORDO** (Matsakis R4): perimetro modellato 1,52 s su 37,6 s ≈ 4% del gap — B3 compra una frazione della tappa, mai la parità; nessun claim oltre la risoluzione (glue ~4,4 s fuori modello).

## Posizione secca sul pavimento
**2,88 ns/movimento è comprimibile SOLO per-conteggio.** Il dispatch ~9–10 ns è invariante dichiarato (S-103, non-bersaglio); il residuo è copia 16 B + smistamento al pavimento macchina; taglia e niche già incassate, rappresentazioni alternative vietate. Chi promette di comprimerlo per-movimento deve prima nominare il meccanismo.

## Emendamenti
- **R1**: prima mossa = censimento consumatori su ORM (ripartizione dei 367,6M per sito/opcode, monobinario ×2, r1==r2); guadagno atteso in secondi.
- **R2**: TakeSlot ammesso solo se il censimento mostra movimenti take-eligible NON borrowable con (incdec+nota)×conteggio ≥ banda giudice; altrimenti chiuso per aritmetica.
- **R3**: forma-flag: nm -S di run_loop E del braccio LoadSlot predette prima/misurate dopo; sub-forma nominata nel criterio.
- **R4**: bl-count (s144-criterio-B p.4) NECESSARIO ma NON sufficiente per H-C2: aggiungere delta-taglia run_loop PREDETTO; scostamento ⇒ STOP prima dell'A/B.

## Kill-switch pre-registrabili
- **KS-BAK-146-1**: bl-count o taglia run_loop fuori predizione ⇒ STOP fetta, niente A/B.
- **KS-BAK-146-2**: censimento ORM: (movimenti eliminabili × prezzo) < banda coppia ⇒ famiglia ridimensionata a micro-only, niente scommessa suite.
- **KS-BAK-146-3**: micro in segno ma pair zcell/arr0 fuori gate 5% ⇒ leva in istruttoria (guardia layout).

Mandato inverso (Gregg): oggi sappiamo che le leve «muovere più a buon mercato» hanno un tetto aritmetico di 0,46 s — ieri TakeSlot sembrava il filone; e che la famiglia borrow spedisce senza liveness (HC1, L-FR1 lo provano).


════════════════════════════════════════
# PEDERSEN — bozza indipendente S-146 (lente: confini per-richiesta, lifecycle, ordine __destruct, output-capture, RetainSet, §3.22)

**VERDETTO: CONCORDO CON EMENDAMENTI** sul quesito. Binding output-capture INTATTO e non emendabile: qualunque variante di B3 che differisca una morte oltre il punto refcount-driven è fuori dal tavolo per costruzione.

## Posizioni a–e

**a) CON EMENDAMENTI.** Il flag `take` deciso a compilazione dentro `LoadSlot` è l'unica forma sotto il tetto WP-39..44 — ma la forma è ORTOGONALE alla semantica: non compra un grammo di fedeltà. Nessuna emissione senza R1–R3 sotto; disasm bl-count resta dovuto (lezione H-C2).

**b) CON EMENDAMENTI.** Il nucleo-stringhe **elimina** il rischio identità/lifecycle — le stringhe PHP non hanno `spl_object_id`, WeakReference, `__destruct`, risorse — **solo se** il guard è runtime-esatto: match `Zval::Str` puro, MAI `Ref` (slot condiviso: lo spostamento è osservabile altrove; safe_ref 0,013% rende il fallback economico, non superfluo — la correttezza non si misura in frequenza). Ciò che il perimetro NON elimina: (i) il rischio LIVENESS — uno slot vivo svuotato si rilegge `Undef` via canali dinamici e il fallimento è SILENZIOSO; la lezione S-96 («corretto per fortuna del corpus») vale per l'analisi, non per il valore; (ii) `memory_get_usage`/`debug_zval_refcount` osservano il momento del free — la seconda è già in rinuncia, la prima va a catalogo divergenze PRIMA, non dopo. Quindi: identità eliminata per costruzione del guard; morte-anticipata ridotta a valore senza effetti; residuo = soundness dell'analisi.

**c) CONCORDO: SÌ, obbligatorio.** would_take 47,1% è del media group WP; il giudice della scommessa è ORM. F1-ORM (sola misura, zero rischio) più tetto aritmetico pre-registrato: massimo teorico = mv_str 3,85 ns × 104,1M clone str ≈ 0,40 s; con frazione takeable tipo-WP ~0,2 s su 37,6 s di gap. Le tre bande di design95 §P1 vanno RI-derivate sui numeri ORM prima di decidere.

**d) CON EMENDAMENTI.** Borrow-first/through-borrow ai siti consumatori PRIMA di TakeSlot: non muove, non lascia Undef, non anticipa NESSUNA morte — zero rischio lifecycle per costruzione (precedenti HC1/L-FR1 spediti). «Arena-conteggi»: definire o archiviare; **pre-registro il veto**: qualunque definizione che porti la morte di un valore a un confine (sweep, fine-op, fine-request) invece che al DELREF→0 collide col binding output-capture e allarga la §3.22 — se non è per-valore con morte refcount-driven, è archiviata.

**e) CONCORDO col limite Matsakis, EMENDO col tetto**: B3-stringhe compra al più ~0,4 s del churn modellato 1,52 s; nessun claim su glue 4,4 s né sulla parità. Se il tetto F1-ORM non raggiunge la banda del giudice, KS-B1 è irraggiungibile per aritmetica: si dichiara PRIMA, non si scopre dopo.

## Emendamenti R1–R4
- **R1 (fixture bilaterali per NOME, PRIMA di ogni riga)**: `fx-destructor-order` (ordine di stampa a fine funzione) · `fx-generator-suspend` (locale letto via `get_defined_vars` dopo resume) · `fx-a-append-a` (`$a .= $a`) · `fx-compact-after-last-use` · `fx-weakref-slot` · in più dalla mia lente: `fx-ref-to-str` (`$r=&$s`) e `fx-resource-close-order` (chiusura file osservabile). Byte-id vs oracle; divergenza non sanabile ⇒ a catalogo, mai saltata.
- **R2 (sonda-verità, probe mai pinnabile)**: build che al take lascia SENTINELLA invece di Undef; ogni rilettura aborta col site-id. Corpus 1414×2 + ORM + batteria: conteggio riletture DEVE essere 0 — converte il fallimento silenzioso in rumoroso.
- **R3 (gate STOP allargato)**: fail NUOVO per NOME in `weakrefs/`, `destructor`, **`generators/`, `references/`** ⇒ STOP fetta. Il gate attuale è necessario, NON sufficiente (fortuna del corpus).
- **R4 (ordine d'istruttoria)**: 1· F1-ORM + tetto; 2· famiglia borrow-first (FR1-ext); 3· TakeSlot solo se tetto ≥ banda, con R1–R3 verdi; 4· arena-conteggi definire-o-archiviare sotto il veto in (d).

## Kill-switch pre-registrabili
**KS-P1** = R3 (STOP + revert). **KS-P2**: sentinelle R2 >0 dopo una riparazione ⇒ perimetro falsificato. **KS-P3**: tetto F1-ORM < banda giudice ⇒ TakeSlot NON si scrive. **KS-P4**: qualunque morte differita a confine ⇒ veto immediato, senza misura.

**Mandato inverso**: oggi sappiamo che il collo è il pavimento per-movimento (69,5%) e che il tetto di B3-stringhe è CALCOLABILE (prezzi firmati × conteggi ORM) — in S-96 non lo era.


════════════════════════════════════════
# Verbale LEIJEN — Concilio S-146 (B3/filone conteggi) — bozza indipendente
Lente: allocatore (mimalloc) / footprint fisico — prezzi alloc/free, bilancio bytes, località.

## VERDETTO: CONCORDO CON EMENDAMENTI

Refutazione centrale (R1): **KS-B4 dice che il collo è il memcpy (69,5%), ma TakeSlot NON compra il memcpy**. Un take converte clone→move: la copia dei 16 byte RESTA; si elidono solo inc/dec (14,1%) e nota (16,4%), cioè la MINORANZA (30,5% = 0,46 s del perimetro modellato 1,52 s), scalata poi da would_take e dal perimetro fedele. L'unica mossa che compra il pavimento «sposta e smista» è NON generare il movimento: borrow-first/through-borrow ai siti consumatori (HC1, L-FR1: zero liveness, zero identità, spedite). Il filone conteggi va quindi ORDINATO con borrow-first come braccio primario e TakeSlot come braccio residuale.

## Posizioni a–e
- **a) CON EMENDAMENTI**: la forma-flag su LoadSlot è l'unica ammissibile (tetto WP-39..44, nm -S predetta, disasm bl-count invariante su run_loop); ma la forma non sana R1 — il flag decide take vs clone, non elimina la copia.
- **b) CONCORDO**: perimetro Stogov (CV non consumati, morte mai anticipata); nucleo-stringhe = unico perimetro senza identità. Dalla mia lente: il take è alloc-neutro anche su str (clone ZStr = inc, non malloc) — nessun acquisto sul canale alloc, vedi R2.
- **c) CONCORDO — serve**: i conteggi 47,1%/90,2% sono del media group WP; il mix ORM è diverso e il moltiplicatore 4,5–6,5% è SCREEN R=1. F1 su ORM è census a rischio zero e oggi ha finalmente un prezzo per-movimento firmato per moltiplicarlo — ma col prezzo GIUSTO (R3).
- **d) CON EMENDAMENTI**: borrow-first PRIMA di TakeSlot (R1). «Arena-conteggi»: **ARCHIVIARE**. Dalla mia lente non è definibile coerentemente: se significa drop-a-blocco è una leva di PREZZO travestita da conteggi (e KS-B4 ha appena mostrato che il prezzo non è il collo), viola il veto alloc-removal-senza-costo-sostitutivo, il binding output-capture, e il mio reperto S-143 (l'arena non batte mimalloc sul ns/coppia, ~1–3 s diretti). Se qualcuno la rivuole, rientra solo come A-pool (pool+refcount+handle-gen) con gli oneri S-143 — fuori dal filone conteggi.
- **e) CON EMENDAMENTI**: B3 compra al più fette del perimetro modellato 1,52 s (tappa, mai parità; nessun claim sui ~4,4 s glue). In più, dalla mia lente: **B3 compra ZERO del canale alloc** — i 471,3M pair (str 27,6%, other 57,9%) restano INTERI. Tranche-3 growth-alloc è quindi **complementare** in istruttoria (census a conteggi, niente codice, non blocca B3) e **concorrente** per la leva successiva: il residuo 57,9% senza nome è più grande di tutto ciò che B3 può comprare.

## Emendamenti
- **R1** (cosa/perché/misura): ordine istruttoria = FR1-ext borrow-first (chiave da SLOT, FieldRead/isset) → F1-ORM census → TakeSlot flag-form solo se (inc-dec+nota)×would_take_ORM supera la soglia REGOLE §3. Misura: criterio per-sito ≤10 righe, micro dedicata, ABAB R=5.
- **R2**: ogni criterio B3 dichiara «galloc/gfree invarianti per costruzione»; census di guardia a conteggi. Un delta galloc sotto una fetta B3 = effetto non capito ⇒ STOP e nominare.
- **R3**: i prezzi pair zcell 8,71–8,80 / arr0 11,57–11,79 sono sotto gate MAI ricollaudato (az.rev. #5): grado **INDIZIO**, e comunque **peso NULLO nel budget B3** — prezzano un canale (nascite/morti) che B3 non tocca; servono solo a un'eventuale A-pool, dove pretenderebbero rerun sotto gate 5%.
- **R4**: gate footprint vmmap resta su ogni fetta (Undef negli slot e borrow non devono muovere il fisico).

## Kill-switch pre-registrabili
- **KS-L1**: fetta B3 con delta galloc_n fuori parità ⇒ STOP fetta.
- **KS-L2**: criterio che usa i prezzi pair come budget di una fetta conteggi ⇒ criterio invalido (giudice sotto-risoluto).

## Mandato inverso (Gregg)
Oggi sappiamo che il pavimento per-movimento è dispatch+move e che nessuna leva della famiglia «take» lo tocca: ieri il filone conteggi sembrava un'alternativa al prezzo, oggi sappiamo che metà della famiglia (take) paga lo stesso pavimento e solo il borrow lo evita.


════════════════════════════════════════
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


════════════════════════════════════════
# Verbale GREGG — Concilio S-146 (quesito B3/filone conteggi) — bozza indipendente

## §MANDATO-INVERSO — che cosa sappiamo OGGI di phpr che ieri non sapevamo
1. Il ciclo per-movimento è PAVIMENTO-dominato: memcpy 69,5% (1,06 s) vs inc-dec 0,21 s vs nota 0,25 s (SONDA, regola p.3 pre-registrata). Rendere il movimento più economico (B1/B2) è morto per misura; può pagare solo muovere MENO.
2. Abbiamo PREZZI per-movimento firmati (2,88–3,85 ns/coppia, per tipo) e CONTEGGI ORM (367,6M): qualunque conteggio futuro si converte in SECONDI — il moltiplicatore SCREEN 4,5–6,5% è pensionabile.
3. Il perimetro modellato ha un TETTO assoluto: 1,52 s su 37,6 s di gap (memcpy 1,06 s). Coerenza esterna: gcnote 238,6M == dossier S-141 ESATTO.
4. churn_zval è IN budget vs oracle (whole-stack 0,24–0,26pp ≪ 5,15pp); memops FUORI (62–66% ≥50%): metà del bersaglio storico di B è fuori budget in attesa di attribuzione Zval-move.
Che cosa NON sappiamo né oggi né ieri: la liveness su ORM (tutti i numeri liveness sono media group WP) e il PONTE tra le due convenzioni di conteggio (slot_reads vs movimenti).

## VERDETTO
**CONCORDO CON EMENDAMENTI** sul quesito. Ordine d'istruttoria dalla mia lente: (1) census F1-su-ORM + provenienza per sito, con gate aritmetico pre-registrato (R3); (2) solo se la banda sopravvive si istruiscono forma (a) e perimetro (b); le estensioni borrow-first (d) procedono in parallelo: non richiedono liveness.

- **a) CON EMENDAMENTI.** «Il flag take non è un corpo caldo in più» è un'ASSERZIONE, non una misura: si istruisce solo con taglia `nm -S` PREDETTA + disasm bl-count prima/dopo pre-registrati (criterio-B p.4, A-LB-97-1). Non valutabile prima del census.
- **b) CON EMENDAMENTI.** Il nucleo-stringhe è un perimetro derivato dai numeri WP (str 18,7% di slot_reads_rc); trasferirlo a ORM è un denominatore a memoria (veto vigente). Il perimetro si sceglie DOPO i conteggi ORM — la composizione differisce (str = 27,6% del galloc ORM; mv_str 104,1M vs mv_scalar 91,1M nella sonda).
- **c) CONCORDO: è il numero che manca.** Con tre obblighi (R1–R3).
- **d) CON EMENDAMENTI:** borrow-first di pari grado, ordinata dal census di provenienza (R5); **arena-conteggi si ARCHIVIA** se nessuno la definisce con criterio proprio ≤10 righe — una leva senza giudice nominabile non si istruisce (az.rev. S-145 #4).
- **e) CONCORDO** (Matsakis R4 confermato): tetto modellato 1,52 s = contributo di tappa, mai parità; anche al tetto (irraggiungibile) B3 compra ≤2,8% del gap. Nessun claim oltre la risoluzione; i ~4,4 s di glue restano fuori modello.

## Emendamenti R1..R5 (cosa/perché/misura)
**R1 — Ponte di convenzione.** slot_reads (census: letture di slot) e «movimenti» (sonda: coppie clone+drop) NON sono la stessa grandezza — i 367,6M includono nascite/args/prop fuori dagli slot. Il census F1-ORM emette NELLA STESSA RUN slot_reads_rc(ORM), would_take_safe(ORM) E un contatore-ponte (movimenti con origine slot-read / totale confrontabile col 367,6M), denominatori dal sorgente; VIETATO dividere would_take (convenzione census) per 367,6M (convenzione sonda). Monobinario census, ×2 repliche, r1==r2 per chiave.
**R2 — Pensionare lo SCREEN.** Banda d'attesa B3 = would_take_safe per-tipo × prezzo pair per-tipo della sonda = SECONDI (grade derivato CENSUS×SONDA, dichiarato). Mai più il 4,5–6,5% R=1 del media group WP su ORM.
**R3 — Kill aritmetico di visibilità (pre-registrato PRIMA del census).** Se l'estremo ALTO della banda R2 < risoluzione del giudice della scommessa (±0,7% della coppia ORM net ≈ 0,26–0,30 s), B3-TakeSlot NON si apre: ucciso a tavolino, zero codice.
**R4 — Memops resta VOCE PROPRIA.** B3 non chiude l'attribuzione Zval-move: un calo della famiglia memops post-B3 tra binari diversi è solo DIREZIONE (veto «differenze tra A/B distinti come cifra»). Giudici di B3 = micro churn + coppia ORM, MAI la quota memops.
**R5 — Census di provenienza per SITO dei 367,6M**: ranking misurato dei bersagli borrow-first (famiglia FR1-ext), stesso principio del «moltiplicatore» interno di B1.

## Kill-switch
**KS-G1** = R3 (banda sotto risoluzione ⇒ B3 chiuso senza codice). **KS-G2**: contatore-ponte non definibile dal sorgente, o r1≠r2 >1% ⇒ nessuna banda derivata, riconvoca. **KS-G3**: qualunque fetta B3 giudicata sulla quota memops ⇒ STOP (giudice sbagliato).


════════ NOTE DI TEAM (fase 2) ════════

────────────────────────────────
# Team FORMA-MOTORE (Hoare · Hejlsberg · Bak) — nota di fase 2, S-146

Fonte VINCOLANTE: i verbali individuali. Questa nota registra, non sostituisce.

## 1) CONVERGENZE
- **3/3 (refutazione centrale)**: TakeSlot NON rimuove il pavimento memcpy (69,5%) — un take è ancora un movimento (copia 16 B + store Undef); elide solo inc-dec+nota (≤0,46 s teorici). Il borrow/through-borrow è l'unica classe coerente con KS-B4: elimina il movimento intero. **Ordine invertito: borrow PRIMA, take poi/retrocesso** (Hoare R4, Hejlsberg d, Bak R2).
- **3/3 forma-flag ammissibile**: bit/campo dentro il braccio LoadSlot esistente = zero corpi caldi nuovi ⇒ **O1 NON è prerequisito** di questa forma; restano dovuti nm -S PREDETTA prima e disasm bl-count (lezione H-C2). Bak: il vincolo si trasforma in vincolo di TAGLIA del braccio (+16–48 B stimati, rischio inliner).
- **3/3 perimetro (b)**: nucleo senza identità (Stogov); guard di tipo su Ref obbligatorio e MAI superfluo (safe_ref 0,013%, economico via BTB).
- **3/3 arena-conteggi**: ARCHIVIARE salvo definizione scritta (≤1 pagina per Hoare) che riduca movimenti e superi i veti alloc.
- **3/3 (e)**: B3 compra ≤1,52 s modellati; nessun claim su parità né sui ~4,4 s di glue.

## 2) CONFLITTI NON LEVIGATI
- **Primo censimento ORM**: Hoare = siti borrow-abili (classe FR1, zero liveness); Hejlsberg = **MI OPPONGO a F1** com'è posta: prima i DIGRAMMI (LoadSlot;CallArg/…); Bak = **esige F1-ORM** con classi allineate alla partizione sonda prima di aprire QUALSIASI forma. Tre oggetti diversi.
- **Bak, posizione secca non condivisa dagli altri**: TakeSlot realistico ~0,2 s < banda giudice (~0,3 s) ⇒ non istruibile come scommessa suite; e «2,88 ns/movimento comprimibile SOLO per-conteggio».
- **Sigilli di tipo (solo Hoare)**: SlotMode enum non-bool, token ZST per Take, mutation-check del guard, sentinella read-after-take (R1–R3, R5).
- **Solo Hejlsberg**: ordine per-tipo inattuabile a compilazione (R3); scommessa KS-B1 da RI-REGISTRARE alla scala del perimetro.

## 3) PRIORITÀ per S-147
1. Censimento consumatori su ORM (conciliare i tre oggetti: per sito/opcode/digramma, guadagno in SECONDI = conteggio×prezzo firmato).
2. FR1-ext borrow ai siti consumatori, protocollo L-FR1 per nome (criterio ≤10 righe, R=5 ABAB, giudici dentro lo script).
3. TakeSlot forma-flag SOLO se residuo take-eligible non-borrowable ≥ banda, coi sigilli Hoare R1–R3.
4. Arena: archiviata salvo definizione.

## 4) KILL-SWITCH
- **Unificato 3/3**: bl-count run_loop o taglia nm -S fuori predizione ⇒ STOP fetta (KS-H3/H1/BAK-1).
- **Coincidenti nello spirito, esiti diversi**: censimento sotto soglia ⇒ Hoare: FR1-ext non si apre; Hejlsberg: B3 chiusa SENZA codice; Bak: micro-only, niente scommessa suite.
- **Vigente citato**: fail NUOVO per NOME weakrefs/destructor ⇒ STOP.
- **Solo Hoare**: read-after-take>0 ⇒ STOP take; mutante-guard sopravvive ⇒ STOP fetta. **Solo Bak**: pair zcell/arr0 fuori gate 5% ⇒ leva in istruttoria.


────────────────────────────────
# Team SEMANTICA-CONFINI — nota di relazione S-146 (Stogov · Pedersen · Matsakis)
I verbali individuali restano la fonte VINCOLANTE.

## 1) CONVERGENZE
- **3/3 — Borrow prima di take**: l'ordine fedele è through-borrow ai siti consumatori (precedenti HC1/L-FR1), TakeSlot subordinato (Stogov d; Pedersen R4; Matsakis verdetto+R1). Matsakis dà il perché di conto: il take NON elimina il memcpy (69,5%), solo l'inc-dec (14,1%).
- **3/3 — Perimetro fedele = nucleo senza identità**: scalari/stringhe consumabili (stringhe solo con analisi sound); array/oggetti MAI con drop anticipato — `__destruct`, spl_object_id, WeakReference, risorse osservabili anche senza destructor (Stogov b vincolante; Pedersen b; Matsakis b).
- **3/3 — Arena-conteggi si ARCHIVIA** salvo definizione su carta che conservi morte refcount-driven (veto Pedersen su morte-a-confine; veto costo-sostitutivo Stogov/Matsakis).
- **3/3 — Nessun claim oltre risoluzione** (e): perimetro modellato 1,52 s su 37,6 s; niente parità.
- **2/3 — Fail rumoroso obbligatorio**: sentinella/Poison-Undef come build di collaudo, MAI nel pin (Pedersen R2; Matsakis R4).
- **2/3 — Guard `Ref` runtime mai superfluo**: safe_ref 0,013%, la correttezza non si misura in frequenza (Pedersen b; Matsakis a).

## 2) CONFLITTI NON LEVIGATI
- **Quale censimento**: Stogov R1 = movimenti per SITO D'ORIGINE×categoria; Matsakis R2 = borrow-census per sito CONSUMATORE (F1-liveness serve SOLO se si istruisce TakeSlot); Pedersen c = F1-ORM obbligatorio con tetto aritmetico. Tre oggetti diversi, non fusi.
- **Quanto compra il take su str**: Pedersen ~0,4 s (clone str evitato); Matsakis ≤0,21 s (solo inc-dec). Divergenza di MODELLO su cosa il take evita — va sciolta col censimento, non a tavolino.
- **Fondamento dell'analisi**: Matsakis R3 = allowlist SAFE chiusa, renounce-list vietata per nome; Stogov accetta le rinunce S-96 intere. Pedersen intermedio (soundness residua).
- **Soglie fetta**: Stogov R2 ≥100M movimenti per giudice-coppia vs Matsakis KS-M2 quota <20% — soglie diverse, non riconciliate.

## 3) PRIORITÀ S-147 — fixture/gate per NOME = PRE-condizione di ogni riga
1. **Fixture bilaterali Pedersen R1** (byte-id vs oracle): fx-destructor-order, fx-generator-suspend, fx-a-append-a, fx-compact-after-last-use, fx-weakref-slot, fx-ref-to-str, fx-resource-close-order.
2. **Gate STOP allargato**: fail NUOVO per NOME in weakrefs/destructor + generators/references (Pedersen R3; Matsakis KS-M4).
3. **Censimento ORM monobinario** (×2, r1==r2, parità per NOME) sciogliendo il conflitto 2a.
4. Solo poi fette FR1-ext; TakeSlot dietro KS-M3/KS-P3/R3-Stogov.

## 4) KILL-SWITCH DEL TEMA
- KS-ST-146-3 / KS-P4: take su container senza deferral, o morte a confine ⇒ veto immediato senza misura.
- KS-P2: sentinelle >0 dopo riparazione ⇒ perimetro falsificato.
- KS-M3: TakeSlot chiuso finché inc-dec ≤20% del churn ripartito.
- KS-P1/KS-M4/KS-ST-146-1: fail NUOVO per NOME ⇒ STOP fetta + revert.
- KS-ST-146-2: siti aggredibili <100M ⇒ niente giudice-coppia, o si chiude B3.


────────────────────────────────
# Team MISURA-GIUDICI (Gregg, Klabnik, Leijen) — nota di sintesi S-146
I verbali individuali restano la fonte VINCOLANTE.

## 1) CONVERGENZE
- **3/3 — Lo SCREEN 4,5–6,5% è morto**: ogni banda B3 si esprime in SECONDI = conteggi × prezzi per-movimento della sonda (Gregg R2; Klabnik R2 con non-citabilità di design95 §P1 e righe guadagno_*; Leijen c).
- **3/3 — Arena-conteggi: ARCHIVIARE** senza definizione ≤1 pagina con giudice nominato (Gregg d; Klabnik d; Leijen d: se è drop-a-blocco è leva di prezzo, rientra solo come A-pool con oneri S-143).
- **3/3 — Borrow-first/FR1-ext si istruisce comunque** (zero liveness, non attende censimenti): Klabnik e Leijen la ordinano PRIMARIA (unica che elimina il movimento, cioè il pavimento 69,5%); Gregg pari grado in parallelo.
- **3/3 — e)**: tetto modellato 1,52 s = tappa, mai parità; nessun claim sui ~4,4 s glue.
- **3/3 — kill aritmetico pre-registrato PRIMA del census** (Gregg R3; Klabnik R1/K1; Leijen R1 via soglia REGOLE §3).
- **2/3 (Klabnik, Leijen) — TakeSlot non compra il memcpy**: elide solo inc-dec (0,21 s) + eventualmente nota (0,46 s, non provato) ⇒ per Klabnik pre-ucciso a tavolino (KS-B3-K2); Gregg rinvia il verdetto a dopo il census.

## 2) CONFLITTI non levigati
- **Quale censimento**: Gregg = F1-su-ORM con contatore-ponte slot_reads↔movimenti (R1: VIETATO mescolare le due convenzioni) + provenienza per sito (R5); Klabnik = MI OPPONGO a F1-ORM ora, solo census SITI-CONSUMATORI; Leijen = F1-ORM sì ma dopo FR1-ext. R5-Gregg e c-Klabnik convergono di fatto sui siti; F1 resta 2/3 contro 1.
- **Soglie**: Gregg 1× banda giudice (~0,26–0,30 s); Klabnik 2× (0,6 s suite-judged, oppure 4 ns/iter micro-judged); Leijen rinvia a REGOLE §3. Non levigato.
- **Solo Leijen**: canale alloc — B3 compra ZERO dei 471,3M pair; guardia «galloc invariante per costruzione» (R2, KS-L1); prezzi pair zcell/arr0 = INDIZIO, peso nullo (gate 5% mai ricollaudato, R3); tranche-3 growth-alloc concorrente per la leva dopo (residuo 57,9%).

## 3) PRIORITÀ S-147
Un SOLO census ORM monobinario (r1==r2, criterio ≤10 righe firmato prima) che emetta nella stessa run: movimenti per SITO × prezzo per classe (soddisfa Klabnik-c e Gregg-R5) + contatore-ponte e F1 (Gregg R1); kill aritmetico registrato PRIMA — soglia da arbitrare in sintesi (1× vs 2× banda). FR1-ext procede in parallelo senza attendere.

## 4) Kill-switch del tema
KS-G1/K1 banda<soglia ⇒ zero codice; KS-G2 ponte indefinibile o r1≠r2>1% ⇒ riconvoca; KS-G3 e KS-L2 giudice sbagliato (quota memops / prezzi pair come budget) ⇒ criterio invalido; KS-L1 delta galloc ⇒ STOP fetta; KS-B3-K3 modello falsificato ⇒ stop famiglia; KS-B3-K4 4 sessioni ORM fermo ⇒ riconvoca.

