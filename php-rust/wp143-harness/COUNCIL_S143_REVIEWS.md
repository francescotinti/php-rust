# COUNCIL_S143_REVIEWS — fascicolo del concilio a 9 (S-143, rotta utente 2026-08-15)

Assemblaggio = cat (token-lean). Ordine: sintesi · 4 note di team · 9 verbali (fonte VINCOLANTE).

════════ concilio/sintesi.md ════════
# CONCILIO a 9 — S-143 — sintesi di convergenza (dossier budget-di-parità ORM)

Convocato su rotta utente 2026-08-15. Fascicolo: `COUNCIL_S143_REVIEWS.md`
(verbali integrali VINCOLANTI + 4 note di team). Protocollo a due fasi
(bozze indipendenti → team sigilli/motore-costo/semantica-confini/metodo-gate).

## §FONDAMENTALI

- **Oggetto**: divario ORM 37,6 s (8,59–8,71×); canali ciclo-di-vita ~26–28 s
  (grade INDIZIO, un lato solo). Micro-leve sospese su 4 falsificazioni.
- **Mandato inverso (Gregg)** — cosa sappiamo oggi che ieri non sapevamo: la
  DIREZIONE «ciclo di vita del valore» è firmata bilateralmente (tassa 10×
  per-statement S-129, costo/op ~9–10 ns S-103, 4 falsificazioni micro);
  la RIPARTIZIONE A-vs-B non è firmata da niente: la quota oggetti dei
  471M alloc/free è IGNOTA e l'unico profilo è a un lato solo.
- **Sessioni-senza-misura**: 0 (S-142 ha misurato); S-143 = concilio su rotta
  utente + istruttoria deliberata sotto (leve spedite: 0, dichiarato).
- Rischio d'oggetto più trascurato: «other» 26,6% = 11,3 s senza nome (Gregg
  R3: chiuderla o dichiararla fuori-budget della scommessa).

## Deliberato

**9/9 CONCORDO CON EMENDAMENTI, 0 opposizioni.** Voto sulla scommessa:
**ISTRUTTORIA-PRIMA 7/9** (Hoare, Matsakis, Klabnik, Bak, Leijen, Pedersen,
Gregg) · Stogov **B-poi-A** (con istruttoria comunque per riprezzare A e
profilo oracle prima di promuovere B) · Hejlsberg **A+B ora** (istruttoria
come gate dentro la sessione 1). **Operativamente unanime 9/9: la prossima
sessione di lavoro è l'ISTRUTTORIA, con regola di decisione PRE-REGISTRATA
prima di leggere i dati.** Nessuna sedia vota l'opzione A come scritta.

### Rifondazione dell'opzione A (team semantica-confini, non contestata)
1. **Falso semantico nel dossier** (Stogov): Zend NON azzera il refcount
   sugli oggetti — `GC_DELREF→0` = `__destruct` deterministico, weakref;
   togliere inc/dec renderebbe la §3.22 SISTEMICA. A conserva il refcount;
   l'acquisto onesto è solo alloc/free + località.
2. **Arena → pool** (Pedersen): 29,4 GB/run refutano l'arena senza riuso
   intra-request; il `__destruct` allo sweep viola il binding
   output-capture (che NON si emenda: sweep residui DENTRO request_end()
   DOPO la cattura). A = «pool/slab a classi con handle+generazione,
   refcount conservato, destruct refcount-driven, RetainSet fuori pool».
3. **Costo sostitutivo non prezzato** (Hoare/Matsakis/Bak/Leijen): il veto
   «alloc-removal senza modello del costo SOSTITUTIVO» MORDE A per nome —
   handle-deref (index+gen-check) × (propget 29,9M + recv_clone 14,8M) da
   modellare e sondare PRIMA di ogni riga d'arena; tabella handle =
   slab/indice, MAI HashMap (veto contenitori sul call path).

### Su B e sull'aritmetica di rotta
- B è semanticamente invisibile (niente RetainSet/destruct/identità) e
  apribile con criterio pre-registrato; ma da sola non è scommessa di
  parità (residui ≈ 6,5× per Hejlsberg). Leijen: l'arena NON batte
  mimalloc sul ns/coppia (~1–3 s diretti): A vive dei canali adiacenti.
- **A verbale (Matsakis R4, Klabnik)**: anche A+B al massimo teorico
  (26–28 s) lasciano ~15 s ⇒ ~2,9–3×: la scommessa compra la TAPPA ≤3×,
  NON la parità; il secondo atto (other 11,3 s + dispatch) va dichiarato.

### Veti Q3: tutti e 6 CONFERMATI 9/9
Applicazioni nuove: alloc-removal → morde A per nome · contenitori sul call
path → tabella handle · NaN-boxing resta vietato, la niche di B ne compra la
parte lecita in safe · gc note-time confermato (il grosso è sweep, non nota).

## Istruttoria ordinata (contenuto armonizzato dai 4 team)

a) **Census CH_* per classe E per taglia** su ORM (quota oggetti/props vs
   array vs stringhe vs Vec-args dei 471M pair e dei 29,4 GB; monobinario
   census, ×2 repliche, r1==r2 al singolo evento).
b) **Profilo per famiglia lato ORACLE** (budget = phpr−oracle canale per
   canale) + dichiarare `size_of::<Zval>()` + attribuzione memcpy.
c) **Sonda monobinaria prezzi** alloc/free e gc_note (classe S-138): mai
   più 8–15 ns «plausibili» come budget.
d) **Chiudere il bilancio bytes** (free 33,8 GB > alloc 29,4 GB: incoerenza
   da sanare prima di ogni prezzo sui GB — Leijen R3).
e) **«other» 26,6%**: chiudere la riquantificazione S-141 o dichiararla
   fuori-budget (Gregg R3).

## Regola di decisione (da PRE-REGISTRARE prima dei dati — conflitti a verbale)

Proposta di sintesi (base: Bak a 3 esiti + clausola terza-via Klabnik):
**quota oggetti(+props Rc) ≥40% delle coppie ⇒ A-poi-B (A ricondizionata
pool+refcount) · <25% ⇒ B sola/B-poi-A · 25–40% ⇒ riconvoca su terza via ·
in aggiunta (Klabnik): churn memcpy-dominato ≥60% ⇒ B-prima.**
DISSENSI NON levigati: Stogov kill-A a <15% (solo oggetti) · Pedersen 30%
(A perde il titolo di headline) · Gregg regola binaria ≥35% o ≥10 s ·
orizzonti kill divergenti (Gregg Δ≥5% in 5 sessioni; Klabnik fuori banda
±0,7% entro 4, con revert). La regola firmata vive in
`s143-criterio-istruttoria.md`; i verbali restano la fonte vincolante.

## Oneri pre-prototipo della via A (se l'istruttoria la apre)
Giudici NUOVI prima del primo commit A (Klabnik R4, Stogov R2, Pedersen
R1–R2): fixture identità/weakref/spl_object_id-riuso/§3.22 bilaterali +
gate RetainSet/output-capture (2ª richiesta byte-id) + gate footprint vmmap
(Leijen R4: lo shrink −70 MB non è negoziabile) + modello borrow su carta
(Matsakis R3: doppio-oggetto, re-entrancy, foreach-durante-mutazione,
sopravvivenza oltre request) + spike safe handle brandizzati con micro
deref-arena vs deref-Rc (Hoare R3).

════════ concilio/team-sigilli.md ════════
# Team «sigilli» — sintesi fase 2 (Hoare, Matsakis) · Concilio S-143

Fonte VINCOLANTE: i verbali individuali (`verbale-hoare.md`, `verbale-matsakis.md`).

## §Convergenze
- **Entrambe le sedie: ISTRUTTORIA-PRIMA.** La variabile che decide A (quota per-classe dei 471M alloc/free, §7.1) non è nel dossier: è la grandezza deliberanda, non un limite. Votare A oggi = magnitudine ripartita senza A/B proprio.
- **R1 identico**: census CH_* per classe su ORM, giudice monobinario census, 1 sessione — decide quanto compra A.
- **R2 convergente**: profilo per famiglia lato ORACLE (feedback-one-sided-profile) + attribuzione memcpy + audit `size_of::<Zval>`/niche — il bersaglio è il DIFFERENZIALE, non la famiglia; prezza B e depura il tetto 9,8 s.
- **R3 convergente**: prima di scrivere l'arena, modello dei borrow (doppio-oggetto, re-entrancy con payload vivo, foreach-durante-mutazione, sopravvivenza oltre request) + sonda/micro pre-registrata del costo handle-deref (index+gen-check). Il costo SOSTITUTIVO di A non è prezzato: il veto alloc-removal morde A per nome.
- **Tutti e 6 i veti CONFERMATI** da entrambe (NaN-boxing: la niche di B ne compra la parte lecita in safe; contenitori sul call path: il deref d'arena vi sottostà).
- **RetainSet falsifica l'arena per-request pura**: serve promozione fuori arena; §3.22 (__destruct timing) diventa perimetro di soundness, non nota.

## §Conflitti
- **Sequenza attesa post-istruttoria — NON levigato**: **Hoare** pre-registra **B-poi-A** (B safe-banale e composabile, A carica di mine semantiche); **Matsakis** pre-registra il default **A-poi-B** se la quota oggetti+props supera la soglia R1 (B «non è una scommessa di parità», tetto ≥7× anche perfetta).
- **Soglia kill-switch divergente**: Hoare KS-1 = quota oggetti **<15%** ⇒ A cade; Matsakis = oggetti+props **<25%** ⇒ si ri-delibera. Basi diverse (oggetti soli vs oggetti+props), non riconciliate.
- **Budget di parità**: Matsakis mette a verbale il BUCO — anche A+B al massimo teorico restano ~15 s ⇒ ~3×: la scommessa compra la TAPPA, non la parità; esige la dichiarazione di un secondo atto (coda «other» 11,3 s + dispatch). Hoare non lo contesta ma non lo eleva a condizione del deliberato.

## §Delibera di team
**ISTRUTTORIA-PRIMA** (unanime 2/2); sequenza post-istruttoria DIVISA (Hoare B-poi-A · Matsakis A-poi-B condizionato a R1).

## §Priorità per l'ordine S-143/S-144
1. **Census CH_* per classe su ORM** (R1, giudice monobinario, 1 sessione) — con soglie kill-switch pre-registrate ENTRAMBE (15% oggetti / 25% oggetti+props).
2. **Profilo per famiglia lato ORACLE + attribuzione memcpy + size_of/niche Zval** (R2) — stessa sessione; KS-3 Hoare può retrocedere B senza codice.
3. **Solo se R1 passa**: modello borrow su carta + spike/micro handle-deref pre-registrata (R3) — prima di ogni riga d'arena.

════════ concilio/team-motore-costo.md ════════
# Team «motore-costo» — sintesi (S-143, fase 2) · Sedie: Bak, Hejlsberg, Leijen
I verbali individuali restano la fonte VINCOLANTE.

## §Convergenze
- Tutti e tre: CONCORDO CON EMENDAMENTI; i sei veti Q3 CONFERMATI all'unanimità (NaN-boxing; contenitori sul call path — tabella handle = slab/indice, mai HashMap; alloc-removal senza modello del costo sostitutivo, applicato ad A come suo rischio centrale; SSO inline; GC note-time — la nota è 0,1–1,2 s, il grosso è sweep; notti PhpStr-full).
- Il census CH_* per classe sui 471M pair è irrinunciabile: la quota OGGETTI è ignota e senza di essa il tetto di A «è un atto di fede» (tutti: R1).
- Sonda monobinaria dei prezzi alloc/free reali prima di usare 8–15 ns o 3,8–7,1 s come budget (Bak R4, Hejlsberg R1, Leijen R2).
- Il budget comprabile è phpr−ORACLE canale per canale, non phpr assoluto (Bak R2, Hejlsberg R2; feedback-one-sided-profile).
- Modello SCRITTO e pre-registrato del costo sostitutivo di A: handle-deref × (propget 29,9M + recv_clone 14,8M), sweep per-request, RetainSet/output-capture (Bak R3, Hejlsberg veto applicato, Leijen R5).
- Fare B-poi-A paga la migrazione del layout DUE volte (Bak e Hejlsberg esplicitamente; Leijen delibera A-poi-B): nessuno difende B autonoma come prima mossa.

## §Conflitti
- **Hejlsberg**: deliberare la DIREZIONE ORA — A+B come oggetto unico (l'handle u32 è ciò che rende possibile lo Zval ≤16B con niche); l'istruttoria è GATE dentro la sessione 1, non rinvio; B da sola refutata per aritmetica (32,5 s residui ≈ 6,5×).
- **Bak**: NESSUNA delibera di direzione prima dei numeri — regola pre-registrata che tiene aperto ANCHE B-poi-A (oggetti ≥40% ⇒ A-poi-B; <25% ⇒ B-poi-A; in mezzo ⇒ riconvoca).
- **Leijen**: istruttoria-prima ma con inclinazione A-poi-B condizionata; unico a imporre gate footprint (arena = high-water, vmmap, lo shrink −70 MB non negoziabile) e chiusura del bilancio bytes (free 33,8 > alloc 29,4 GB: incoerenza da sanare prima di ogni prezzo).
- Soglie kill divergenti: Bak 40/25% coppie; Hejlsberg <40% arena-abile; Leijen <30% coppie E bytes.

## §Delibera di team
ISTRUTTORIA-PRIMA (2/3: Bak, Leijen; Hejlsberg dissente: A+B deliberata ora con istruttoria come gate in sessione 1) — operativamente unanime: la sessione 1 è comunque census+sonda con regola di decisione pre-registrata.

## §Priorità S-143/S-144 (max 3)
1. S-143: census CH_* per classe (e taglia/bytes) + sonda monobinaria prezzi + chiusura bilancio bytes; regola di decisione PRE-REGISTRATA prima del run (soglie da armonizzare in plenaria).
2. Profilo per famiglia lato ORACLE (budget = phpr−oracle) + dichiarare sizeof(Zval).
3. Pre-prototipo A: modello scritto del costo sostitutivo + gate footprint vmmap pre-registrato; micro obj* come giudice entro ≤3 sessioni.

════════ concilio/team-semantica-confini.md ════════
# Team «semantica-confini» — Stogov · Pedersen (S-143, fase 2)

I verbali individuali restano la fonte VINCOLANTE.

## §Convergenze
- Entrambi: CONCORDO CON EMENDAMENTI; tutti i 6 veti Q3 confermati (NaN-boxing, contenitori sul call path, alloc-removal senza costo sostitutivo, SSO inline, leva note-time WP-21, notti PhpStr-full).
- **A come è scritta nel dossier è infondata**: per Stogov il claim «azzera il churn Rc» è falso rispetto a Zend (destruct differito ⇒ §3.22 sistemica, weakrefs divergenti); per Pedersen i 29,4 GB/run refutano l'arena-senza-riuso e il `__destruct` allo sweep viola il binding output-capture. A va ricondizionata: refcount conservato stile Zend, destruct refcount-driven nel punto esatto di fine-vita, pool/slab con riuso intra-request, RetainSet fuori pool (handle+generazione nel costo sostitutivo).
- **Il guadagno di A è ignoto** finché il census CH_* per classe (§7.1) non dà la quota oggetti dei 471M; **B non va prezzata** senza profilo oracle per famiglia (§7.2, one-sided-profile).
- **B è semanticamente invisibile** (nessun test a rischio per NOME, non tocca RetainSet né destructor timing) ed è apribile subito con criterio pre-registrato.
- Gate semantico per NOME obbligatorio su A: weakrefs/*, riuso spl_object_id, fixture §3.22, ordine destruct, parità 2ª richiesta; un fail nuovo non catalogabile ⇒ STOP.

## §Conflitti
- **Stogov**: delibera **B-poi-A** — sequenza firmata: B ridefinisce taglia/ABI dello Zval su cui A fissa l'handle; A si riprezza dopo l'istruttoria ma la sequenza è già decisa. Kill A se quota oggetti **<15%**. Chiede di NOMINARE la componente gc-note→possible-root come voce propria del budget (R3), distinta da A e B.
- **Pedersen**: delibera **ISTRUTTORIA-PRIMA** — il census DECIDE se A entra: sotto **30%** di quota oggetti A perde il titolo di headline e resta **B sola**; B-poi-A è solo «plausibile, da confermare». Insiste che A sia rinominata «pool a classi con handle» e che il binding non si emenda: sweep residui DENTRO request_end() DOPO la cattura.
- Soglie kill divergenti (15% vs 30%) e status della sequenza (firmata vs condizionata): non levigato.

## §Delibera di team
B apribile subito + ISTRUTTORIA-PRIMA su A ricondizionata (R1–R4 di entrambi); sequenza B-poi-A indicata da Stogov ma CONDIZIONATA al census per Pedersen — soglie kill 15%/30% non riconciliate.

## §Priorità per l'ordine S-143/S-144
1. Census CH_* per classe (quota oggetti dei 471M, monobinario, r1==r2) + profilo oracle per famiglia — decide A e sconta i canali che anche Zend paga.
2. Aprire B (Zval by-value + niche) con criterio pre-registrato: churn+memops −25% relativo, ORM in banda ±0,7%, ≤3 sessioni.
3. Ricondizionamento scritto di A (pool+refcount+generazione+costo sostitutivo, A/B contro mimalloc) e gate semantico per NOME pre-registrato — PRIMA di ogni prototipo.

════════ concilio/team-metodo-gate.md ════════
# Team «metodo-gate» — sintesi S-143 (Gregg, Klabnik)

Fonte VINCOLANTE: i verbali individuali.

## §Convergenze
1. **Entrambi: CONCORDO CON EMENDAMENTI, delibera ISTRUTTORIA-PRIMA (1 sessione, timeboxed).** Direzione «ciclo di vita» firmata bilateralmente (S-129, S-103, 4 falsificazioni micro); la ripartizione A-vs-B NO: A ha quota oggetti dei 471M IGNOTA, B ha canali da profilo a un lato solo, grade INDIZIO.
2. **Pre-commitment scritto PRIMA dei numeri**: regola di decisione firmata prima di leggere census/profilo (Gregg R1, Klabnik R2).
3. **Contenuto istruttoria**: census CH_* per classe + profilo oracle per famiglia (one-sided-profile: prerequisito) + sonda monobinaria prezzi classe S-138.
4. **Tutti i veti Q3 confermati**; alloc-removal senza costo sostitutivo = «il cuore di A» per entrambi; l'handle di A cade sotto il veto contenitori: deref da prezzare.
5. **Tappe falsificabili giudicate sulla SUITE ORM**, mai solo micro (Gregg R4, Klabnik R3).

## §Conflitti (non levigati)
- **Soglie del pre-commitment**: Gregg R1: quota oggetti ≥35% (o ≥10 s) → A-poi-B, sotto → B-poi-A (regola BINARIA). Klabnik R2: ≥⅓ → A-prima, churn memcpy-dominato ≥60% → B-prima, **entrambi sotto soglia → riconvoca su TERZA via** (esito che Gregg non prevede). Kill-switch di A divergenti: Gregg K1 alla soglia R1; Klabnik K1 fa decadere A già sotto il 25%.
- **K4**: Gregg pretende Δ ORM ≥5% entro 5 sessioni; Klabnik «fuori banda ±0,7%» ma entro 4, con revert al pin. Criterio E orizzonte diversi.
- **Asimmetria A/B**: Klabnik carica A di oneri che Gregg non pone — R4: giudici NUOVI (fixture identità/weakref/§3.22, gate RetainSet/output-capture) prima del primo commit A — e afferma la superiorità incrementale di B (compiler-driven, gate esistenti bastano). Gregg resta neutrale tra le vie.
- **Aritmetica di rotta**: Klabnik esige di dichiarare che la scommessa compra la TAPPA ≤3× (residui ⇒ ~2,9×), NON la parità; Gregg non lo richiede.
- **«other» 26,6% (11,3 s)**: per Gregg R3 va chiuso o dichiarato fuori-budget; Klabnik lo nota solo come residuo post-scommessa.

## §Delibera di team
**ISTRUTTORIA-PRIMA (unanime 2/2)** — 1 sessione timeboxed, pre-commitment scritto prima dei dati; A-vs-B deciso dai numeri firmati.

## §Priorità per l'ordine S-143/S-144
1. **S-143: istruttoria** — census CH_* per classe + profilo oracle per famiglia + sonda prezzi S-138; regola di decisione firmata PRIMA (armonizzare soglie 25%/⅓/35% e clausola terza-via).
2. **S-143: chiudere o dichiarare fuori-budget «other» 11,3 s** (Gregg R3).
3. **S-144: vertical slice della via scelta**, giudice suite ORM; se via=A, prima i giudici nuovi (Klabnik R4).

════════ concilio/verbale-hoare.md ════════
# Verbale sedia Hoare — Concilio S-143 (design runtime Rust safe-only)

VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — la variabile che decide A (quota oggetti dei 471M alloc/free, §7.1) non è un «limite» del dossier: è la grandezza deliberanda; votare A oggi è magnitudine ripartita senza A/B proprio (veto già in lista). Sequenza attesa dopo istruttoria: B-poi-A.

## §Analisi (lente: safe-only, sigilli di TIPO)

**A è costruibile in Rust safe** — arena generazionale (slot+generazione, pattern slotmap) con handle brandizzati da lifetime `'req` (precedente VmGate ZST/lifetime): il sigillo di tipo impedisce STATICAMENTE l'evasione dell'handle dal confine di richiesta, chiudendo use-after-reset per costruzione. Ma tre costi la lente nomina e il dossier NO:
1. **Costo sostitutivo per accesso**: ogni deref d'handle = bounds-check + confronto generazione + chase, al posto del solo chase di Rc. Su 29,9M propget + ogni receiver di ~27,8M chiamate, il segno del saldo NON è ovvio. Il veto «alloc-removal senza modello del costo SOSTITUTIVO» morde A alla lettera.
2. **Aliasing arena↔handle**: due `&mut` vivi nella stessa arena sono impossibili; gli accessi annidati (obj.a.b) impongono disciplina re-fetch (copia handle, re-indicizza). In compenso, se TUTTO il cammino oggetti passa per `&mut Vm`, cade il borrow-flag di RefCell: guadagno possibile, ma è un cambio di architettura, non una leva.
3. **RetainSet attraversa il confine**: oggetti che sopravvivono alla richiesta (pin per-richiesta, binding output-capture) non possono portare `'req` — serve un secondo tier (promozione/copy-out) con mine semantiche nominate: identità `===`, weakref, timing `__destruct` (§3.22 appena catalogata mostra che l'ordine di drop è GIÀ osservabile-divergente).

**B è safe-banale** (solo layout, nessun sigillo necessario), ma il dossier non dà i due numeri che la prezzano: size attuale di Zval/Option<Zval> e la quota Zval-move di memops (i 4,87→9,63 GB di realloc inquinano la famiglia). Il tetto 9,8 s (memops+churn) è sovrastimato finché non depurato; e il profilo è one-sided: anche Zend paga memcpy — il bersaglio è il DIFFERENZIALE, non la famiglia.

**Prezzi §3**: 8–15 ns/coppia plausibile per hot-path mimalloc, ma free di payload grandi + interazione gc_note non è prezzata; la sonda §7.4 va promossa da «residuo» a passo d'istruttoria.

## §Emendamenti
- **R1** (cosa/perché/misura): census CH_* per classe su ORM — decide quanto compra A; giudice monobinario census, 1 sessione.
- **R2**: profilo per famiglia lato ORACLE + attribuzione memcpy (Zval-move vs buffer) + audit `size_of::<Zval>`/niche — prezza B; statico + census, stessa sessione.
- **R3**: se A passa l'istruttoria, PRIMA del port uno spike safe: handle brandizzati + micro pre-registrata deref-arena vs deref-Rc (soglia REGOLE §3, R=5, giudici objmap/objchurn) + design promozione RetainSet con test semantico §3.22.
- **R4**: A può reclamare la riduzione gc_note obj (56,5M) solo come «direzione firmata, magnitudine non ripartita» (REGOLE §4).

## §Veti (Q3)
- **NaN-boxing**: CONFERMO — richiede bit-punning/transmute di puntatori, incompatibile col safe-only; la niche di B ne compra la parte lecita.
- **Contenitori sul call path**: CONFERMO; nota: il deref d'arena È sul call path — lo spike R3 sottostà alla stessa banda.
- **Alloc-removal senza costo sostitutivo**: CONFERMO; R3 ne è l'adempimento obbligato per A.
- **SSO inline**: CONFERMO — canale str 0,8%, non pertinente.
- **GC note-time (WP-21)**: CONFERMO — la riconciliazione §3 lo ri-firma (nota 0,5–1,2 s ≪ famiglia gc 3,4 s: domina sweep); vale R4.
- **Notti su PhpStr-full**: CONFERMO — fuori delibera.

## §Kill-switch (Q4)
- **KS-1 (A, pre-build)**: census R1: quota oggetti <15% dei 471M ⇒ acquisto A <~1,5 s ⇒ A cade senza scrivere codice. Giudice: census monobinario; 1 sessione.
- **KS-2 (A, spike)**: micro R3 con Δ deref ≤ 0 alla soglia ⇒ A cade. Giudici objmap/objchurn; entro 2 sessioni dall'istruttoria.
- **KS-3 (B)**: Zval già ≤16B con niche attiva, o Zval-move <30% di memops depurata ⇒ tetto B <2 s ⇒ B retrocessa. Giudice: size_of + attribuzione census; 1 sessione.
- **KS-4 (globale)**: entro 3 sessioni nessun modello firmato ≥5 s per A o B ⇒ riconvocazione, niente deriva in micro-leve.

════════ concilio/verbale-matsakis.md ════════
VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — il numero che decide A vs B (quota per-classe dei 471M alloc/free) NON è nel dossier (§7.1, ammesso dal dossier stesso); default pre-registrato A-poi-B se la quota oggetti+props supera la soglia di R1.

§Analisi (lente ownership/aliasing/borrow)
**A (handle+arena).** L'handle Copy elimina inc/dec Rc e la coppia alloc/free per oggetto, ma il costo SOSTITUTIVO è il ri-borrow: ogni tocco del payload = index+generation-check via `&Arena`/`&mut Arena`. Conseguenze non prezzate dal dossier: (1) i siti che oggi tengono un guard RefCell attraverso una chiamata re-entrante (call_method_sync, __get/__set, hooks) non possono tenere `&mut arena` viva — obbligo di take/put o borrow corti, cioè lookup ripetuti sul cammino caldo; (2) le op a due oggetti richiedono split-borrow (`get_disjoint_mut`) — safe, ma codice nuovo su OGNI sito; (3) beneficio: iterazione-durante-mutazione migliora (indici stabili, niente panic da alias), e il mass-drop d'arena uccide i cicli gratis. Però RetainSet e oggetti che sopravvivono la richiesta (binding output-capture) falsificano l'arena per-request PURA: serve promozione fuori arena, e §3.22 (__destruct timing) diventa perimetro, non nota. Precisione: A toglie della nota GC al più obj 56,5M su 238,6M (≈24%); gli scalar 73,7M restano.
**B (Zval by-value+niche).** Riduce clone/drop/memcpy ma i punti che oggi prestano `&mut` dentro la mappa restano identici: l'aliasing non cambia, cambia la taglia mossa. Tetto aritmetico: memops 5,4 + churn 4,4 ≈ 9,8 s; anche B perfetto lascia rapporto ≥7. Composabile, rischio semantico basso, ma non è una scommessa di parità.
**BUCO del dossier, da mettere a verbale:** anche A+B al massimo teorico (26–28 s azzerati) lasciano ~15 s su oracle 4,97 ⇒ ~3×. La scommessa compra la TAPPA ≤3×, NON la parità: va dichiarato un secondo atto (coda «other» 11,3 s + dispatch residuo).

§Emendamenti
- **R1** (decide A): census CH_* per classe su ORM, monobinario census s140, r1==r2 al singolo evento; output = quota oggetti/props vs array vs stringhe vs Vec-args dei 471M. 1 sessione.
- **R2** (decide il budget): profilo per famiglia lato ORACLE (feedback-one-sided-profile): un canale che Zend paga in quota simile esce dal budget di parità.
- **R3** (pre-implementazione A): modello dei borrow su carta — i 4 pattern (doppio-oggetto, re-entrancy con payload vivo, foreach-durante-mutazione, sopravvivenza oltre request) + sonda monobinaria del costo handle-deref (index+gen-check) PRIMA di scrivere l'arena.
- **R4**: il deliberato dichiari A+B ≠ parità; budget residuo ~15 s nominato.

§Veti (Q3)
- **NaN-boxing: CONFERMA.** In safe-only la niche si fa via enum layout (B), non bit-tricks: B ne cattura il grosso senza unsafe.
- **Contenitori sul call path: CONFERMA.** L'handle-deref deve essere slab-index O(1) prezzato da R3; se introduce hash sul cammino caldo, il veto morde.
- **Alloc-removal senza modello del costo sostitutivo: CONFERMA, ESTESO ad A per nome:** il sostitutivo di A è index+gen-check+ri-borrow per tocco; senza R3, A non si vota.
- **SSO inline: CONFERMA** (str 0,8%: fuori bersaglio).
- **Leva GC note-time (WP-21): CONFERMA.** A rimuove strutturalmente le note obj: cosa diversa dal tuning del tempo-nota.
- **Notti su PhpStr-full: CONFERMA**, non pertinente.

§Kill-switch (Q4)
- **Istruttoria:** R1 dà oggetti+props <25% dei 471M ⇒ il canale principale di A è falsificato, si ri-delibera. Giudice: census CH_* ×2, ≤1 sessione.
- **A:** prototipo arena sui micro-oggetti; objchurn/objalloc non migliorano ≥2× al giudice micro R=5 (criterio REGOLE §3) entro 3 sessioni ⇒ A cade. Soundness: corpus 1414 ×2 per NOME invariato; regressione __destruct oltre §3.22 ⇒ reject (binding output-capture).
- **B:** replica profilo suite ×2; famiglie memops+churn non calano della quota pre-registrata entro 2 sessioni ⇒ B cade.
- **Budget:** R2 mostra quote relative oracle comparabili su memops/map ⇒ attribuzione §5 da rifare prima di implementare.

════════ concilio/verbale-klabnik.md ════════
VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — la direzione strutturale è giusta, ma A e B oggi non sono confrontabili: nessuna delle due ha la FRAZIONE di canale che rimuove; una sessione di istruttoria con pre-commitment scritto decide A-prima o B-prima senza fede.

## §Analisi (lente: chiarezza, gate, testabilità, migrazione incrementale)
1. **Il dossier prezza i CANALI, non le OPZIONI.** §2-§3 dicono quanto costa il ciclo di vita (~26–28 s), ma non quanto ne rimuove A né B. Per A: la quota oggetti dei 471M alloc/free è dichiaratamente ignota (§7.1). Per B: manca perfino la baseline — size_of::<Zval>() attuale e la decomposizione del clone (memcpy vs inc Rc vs gc_note) non sono nel dossier. Se il clone è dominato da Rc-inc+nota, B non li tocca e il suo acquisto (memops 5,4 + churn 4,4) è un TETTO, non una stima. Buco che invalida un confronto A vs B oggi.
2. **Prezzi unitari non firmati** (§7.4 lo ammette): la somma 26–28 s è fatta di righe INDIZIO. REGOLE §4 vieta di ripartire magnitudini senza A/B proprio; un impegno multi-sessione su prezzi «plausibili» ripete in grande l'errore delle micro-leve.
3. **Profilo a un lato solo**: deliberare senza il profilo oracle per famiglia viola la regola vincolante feedback-one-sided-profile. Zend paga anch'esso memops e map: senza il lato Zend i canali sono sopravvalutati di un fattore ignoto.
4. **Aritmetica di rotta, da dire chiaro**: anche azzerando TUTTI i 26–28 s, 42,5−28 = 14,5 s vs 4,95 ⇒ ~2,9×. La scommessa compra la TAPPA ≤3×, non la parità. Il verbale non deve venderla come «budget di parità» completo: dopo, restano «other» 11,3 s e vm_inline.
5. **Incrementalità**: A non è big-bang-abile senza tappe nominate — tocca identità (`===`, weakref, §3.22 __destruct timing) e il binding output-capture/RetainSet. B è compiler-driven (il tipo cambia, il borrow-checker propaga) e semanticamente neutro: i gate ESISTENTI (batteria 1746, corpus 1414×2 per NOME, ORM 3E/13F, fixture bilaterali) lo validano senza giudici nuovi. A pretende giudici NUOVI: fixture identità/weakref/destruct-timing e un gate sweep-per-request sul RetainSet.

## §Emendamenti
- **R1 (istruttoria, 1 sessione, timebox)**: (a) census CH_* per classe su ORM (quota oggetti/array/stringhe/Vec-args dei 471M); (b) profilo oracle per famiglia (stessa lente); (c) **sonda-B monobinaria**: size_of Zval + ripartizione del churn in memcpy/Rc/nota (classe S-138). Misura: tre numeri firmati agli atti.
- **R2 (pre-commitment scritto PRIMA dei numeri)**: se quota-oggetti ≥⅓ dei 471M → A-prima; se churn memcpy-dominato (≥60%) → B-prima; entrambi sotto soglia → il concilio riconvoca su terza via.
- **R3 (tappe falsificabili della via scelta)**: ogni fetta ha criterio ≤10 righe, giudice micro proprio (objalloc/objchurn per A; memops via churn-probe per B), poi coppia ORM: Δ ≥ banda ±0,7% entro la tappa 2, o si ferma.
- **R4 (giudici nuovi per A, prima del primo commit A)**: fixture identità/weakref/§3.22 bilaterali + gate RetainSet/output-capture.

## §Veti (Q3)
- NaN-boxing: **CONFERMA** — la niche di B non deve degradare in NaN-box.
- Contenitori sul call path: **CONFERMA con emenda strettissima**: la tabella handle di A è ammessa SOLO con modello del costo di deref + A/B con disasm bl-count per fetta.
- Alloc-removal senza modello del costo SOSTITUTIVO: **CONFERMA** — è il cuore di A: sweep-per-request e deref vanno prezzati prima.
- SSO inline · notti PhpStr-full: **CONFERMA** (str 0,8%, fuori bersaglio).
- Leva GC note-time (WP-21): **CONFERMA** come leva puntuale; la rimozione STRUTTURALE della nota obj (56,5M) in A è cosa diversa e non la riapre.

## §Kill-switch (Q4)
- **K1**: census per-classe: oggetti <25% dei 471M ⇒ A decade (giudice: census monobinario, 1 sessione).
- **K2**: sonda-B: churn ≥60% Rc+nota ⇒ B decade (giudice: sonda S-138, 1 sessione).
- **K3**: profilo oracle: se Zend paga ≥50% della quota phpr su memops+map ⇒ canali riprezzati, delibera rifatta.
- **K4**: via scelta: dopo 4 sessioni di fette spedite, coppia ORM ferma dentro banda ±0,7% ⇒ revert al pin e riconvoca.

════════ concilio/verbale-hejlsberg.md ════════
# Verbale — sedia Hejlsberg (lente: ingegneria dei compilatori, layout del valore)

VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: A+B — handle unificato: l'handle di A è ciò che RENDE POSSIBILE lo Zval 16B con niche di B; deliberare la direzione ORA, con l'istruttoria §7.1–§7.2 come GATE dentro la sessione 1, non come rinvio.

## §Analisi (lente compilatori)

1. **B da sola non chiude, per aritmetica.** Bersagli di B: memops 5,4 s + churn_zval 4,4 s (+ quota drop-glue in vm_inline). Azzerandoli TUTTI: 42,5−10 ≈ 32,5 s vs 5 s oracle → ancora ~6,5×. B come opzione autonoma è refutata dai numeri stessi del dossier.

2. **A e B non sono indipendenti: sono lo stesso oggetto di layout.** Oggi lo Zval porta (presumo) puntatori Rc a payload boxati; un handle u32+generation in arena è esattamente la rappresentazione che permette un enum Zval by-value ≤16B con niche (Option<Zval> gratis, discriminante nel niche del handle). Fare B prima (layout attorno agli Rc) e poi A significa rifare il layout due volte. Sequenza corretta: il design del handle DETTA il layout; B-per-oggetti è un corollario di A, B-per-stringhe/array resta leva successiva.

3. **Buco nominato n.1: il dossier non dichiara sizeof(Zval) attuale.** I 5,4 s di memops non sono valutabili come bersaglio senza la larghezza del memcpy per movimento, prima/dopo. Va misurato e scritto (una riga, costo zero).

4. **Buco nominato n.2 (il più duro): i 26–28 s sono a un lato solo.** Il comprabile per canale è phpr−oracle, non phpr: Zend paga anch'esso memcpy, hash ins/lookup, sweep. Il §7.2 lo ammette. Deliberare la DIREZIONE ora è lecito (§1 del dossier: la sottrazione dei canali minori non arriva a parità, quindi la scommessa è strutturale per esclusione); deliberare la MAGNITUDINE senza profilo oracle no.

5. **Dispatch non è il bersaglio** (~9–10 ns/op invariante, S-103): confermo che nessuna via che tocchi il dispatch (NaN-boxing incluso) compra sul collo vero. Il guadagno secondario atteso di A+B sul run_loop è la riduzione della drop-glue inline (34,5% dei subtree Zval-glue): meno glue → icache e inliner respirano (lezione H-C2: le leve run_loop pretendono disasm/bl-count prima-dopo).

## §Emendamenti

- **R1 (gate, sessione 1)**: census CH_* per classe su ORM + sonda monobinaria prezzi (§7.4). Decide il tetto di A: quota oggetti/array dei 471M. Senza R1 il tetto di A è un atto di fede.
- **R2 (gate, sessione 1–2)**: profilo per famiglia lato ORACLE (stessa lente). Il budget comprabile si riscrive canale per canale come phpr−oracle.
- **R3 (design)**: il handle nasce col layout: Zval enum by-value ≤16B, niche documentato, sizeof asserito in `cargo test`. Vietato spedire A con Zval invariato "per poi fare B".
- **R4 (misura)**: prototipo su micro objchurn/objalloc/objmap PRIMA della suite; criterio pre-registrato ≤10 righe (REGOLE §3); disasm bl-count sul run_loop prima/dopo.
- **R5 (fedeltà)**: §3.22 (__destruct timing), `===`, weakref, binding output-capture-before-reset: fixture nominate nel gate, non promesse.

## §Veti (Q3)

- **NaN-boxing: CONFERMO.** Safe-only + niche rende il punning inutile; il dispatch non è il collo; distrugge il pattern-matching dell'enum.
- **Contenitori sul call path: CONFERMO** (calls = 1,7 s, non paga).
- **Alloc-removal senza modello del costo sostitutivo: CONFERMO e APPLICO ad A**: l'arena È alloc-removal; A è autorizzata solo con modello nominato di bump-alloc + sweep per-request + promozione RetainSet, prezzato da sonda monobinaria.
- **SSO inline: CONFERMO** — riesame solo se R1 mostra quota stringhe dominante.
- **GC note-time (WP-21): CONFERMO** — la riconciliazione §3 lo ribadisce (nota 0,5–1,2 s, il grosso è sweep). A riduce le note strutturalmente, non è una leva note-time.
- **Notti su PhpStr-full: CONFERMO.**

## §Kill-switch (Q4)

- **KS1** (1 sessione): R1 mostra quota arena-abile (oggetti+array a vita per-request) <40% dei 471M → tetto di A sotto il necessario → A decade, si ridelibera.
- **KS2** (≤3 sessioni): prototipo handle+arena sulle micro obj* non porta objchurn/objalloc/objmap da 6–12× a ≤3× → meccanismo refutato al suo giudice (wp97 micro R=5).
- **KS3** (≤5 sessioni): suite ORM in coppia A/B non scende ≥10% (≫ banda ±0,7%) → la tesi «la struttura compra il prezzo unitario di tutti i siti» è falsificata come le micro-leve.
- **KS4** (permanente): regressione per NOME su corpus congelato/gate ORM 3E/13F o divergenza semantica §3.22/`===`/weakref non curabile → revert al byte.

════════ concilio/verbale-bak.md ════════
# Verbale sedia Bak (lente: VM di produzione — V8/HotSpot) · S-143

VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — una sessione per §7.1+§7.2 con regola di decisione PRE-REGISTRATA che converte i numeri in B-poi-A (lean attuale) o A-poi-B; nessuna scommessa pluri-sessione su quote non censite.

## §Analisi
Da una VM di produzione: nessun team V8/HotSpot spedisce un cambio di rappresentazione senza allocation-site profiling per classe. Il dossier è onesto (§7.1): la quota OGGETTI dei 471M pair è IGNOTA. L'acquisto di A oggi non è un numero — in workload ORM-hydration l'esperienza dice che stringhe/array/Vec-args spesso dominano gli alloc, non gli oggetti. B invece attacca il PREZZO UNITARIO di ogni movimento in ogni sito (memops 5,4 + churn 4,4 + parte del drop-glue in vm_inline): è l'unica opzione che risponde per costruzione alla lezione delle 4 falsificazioni («il singolo sito non muove la suite»). A attacca UNA famiglia con quota ignota e AGGIUNGE un load per accesso (handle-deref): V8 lo paga su Local<> e lo ammortizza con HandleScope on-stack + pointer compression + IC; phpr ha PropIc ma propget 29,9M + recv_clone 14,8M sono il moltiplicatore della nuova tassa — va prezzato PRIMA (veto alloc-removal). Nota: l'acquisto GC di A sulla nota è piccolo (obj 56,5M × 2–5 ns = 0,1–0,3 s); il grosso gc è sweep (59,2M siti), che A ridisegna ma non azzera. Dubbi sui prezzi §3: il pair 8–15 ns è plausibile ma la riconciliazione regge solo includendo i memcpy; realloc 19,5M (9,6 GB mossi) è Vec-growth che A NON tocca e B tocca solo via taglia. Ordine: se A mette un handle u32 nello Zval, RISCRIVE il layout — fare B poi A significa pagare due volte la migrazione; anche per questo la sequenza si decide sui numeri del census, non a gusto.

## §Emendamenti
- **R1**: census CH_* per classe (oggetti/array/str/Vec-args sui 471M) + tabella accessi (propget/recv_clone) = moltiplicatore handle-tax. Regola pre-registrata: oggetti ≥40% dei pair ⇒ A-poi-B; <25% ⇒ B-poi-A; in mezzo ⇒ riconvoca.
- **R2**: profilo per famiglia lato ORACLE (feedback-one-sided-profile): i canali che anche Zend paga (memops, map) vanno scontati o si sopravvaluta l'acquisto di entrambe.
- **R3**: per A, modello del costo SOSTITUTIVO obbligatorio e pre-registrato: handle-deref × accessi, sweep arena per-request, crescita tabella, RetainSet/output-capture.
- **R4** (§7.4): sonda monobinaria classe S-138 sul prezzo pair alloc/free prima di usare 3,8–7,1 s come budget.
- **R5**: «other» 26,6% riquantificata (S-141) prima di attribuire alla struttura guadagni residui — non è riserva di caccia.

## §Veti (Q3)
- NaN-boxing: CONFERMO. B = Option/niche by-value, non NaN-boxing; riaprirlo esige dossier proprio.
- Contenitori sul call path: CONFERMO. La tabella handle di A = slab indicizzata per-request, mai HashMap sul cammino caldo.
- Alloc-removal senza modello del costo sostitutivo: CONFERMO ed ELEVO — è esattamente il rischio di A (R3 vincolante).
- SSO inline: CONFERMO (stringhe fuori perimetro di A e B).
- Leva GC note-time (WP-21): CONFERMO — A non si vende come cura della nota (0,1–0,3 s); il suo acquisto gc è sulla forma dello sweep, da modellare.
- Notti su PhpStr-full: CONFERMO.

## §Kill-switch (Q4)
1. Istruttoria (1 sessione): census CH_* r1==r2 <1% + profilo oracle; se oggetti <25% E l'oracle paga quota memops comparabile ⇒ la scommessa torna in concilio.
2. Via deliberata (B o A): prototipo entro ≤3 sessioni; giudici = micro churn/objdatains (A/B ABAB, soglia REGOLE §3) E coppia suite ORM 2/lato net: Δ suite <3% ⇒ FALSIFICATA (la struttura promette la suite: il giudice che ha ucciso 4 micro-leve la giudica).
3. Solo A: corpus 1414 per NOME ×2 + fixture ===/weakref/__destruct (§3.22); violazione binding output-capture = reject immediato; handle-tax misurata oltre l'UB di R3 su micro prop ⇒ kill.

════════ concilio/verbale-pedersen.md ════════
VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — il census CH_* per classe (§7.1) decide quanto compra A; intanto B è apribile subito perché non tocca il confine per-richiesta; A entra solo ricondizionata da R1–R4.

§Analisi (lente: confine per-richiesta e lifecycle)
1. **Il dossier refuta da sé l'opzione A come è SCRITTA.** §3 dichiara 29,4 GB allocati/run: un'arena per-request che non recupera intra-request richiederebbe ~29 GB residenti sulla suite ORM. Zend NON vive di sweep-a-fine-request: vive di pool/slab con free refcount-driven e riuso immediato. Quindi A va rinominata: «pool a classi con handle», non «arena+sweep». Il dossier non lo dice; è un buco che invalida la formulazione, non la direzione.
2. **A tocca esattamente il BINDING Pedersen/Stogov** (output capture PRIMA di request_end(); parità per-richiesta sul RetainSet). Un `__destruct` differito allo sweep emette output DOPO la cattura → violazione da reject; §3.22 mostra che la classe di divergenza esiste già oggi e si moltiplicherebbe.
3. **Il guadagno di A è NON quantificato**: dei 471M coppie non sappiamo la quota oggetti; gc_note obj è 56,5M su 238,6M (24%), e la famiglia gc è dominata dallo sweep (riconciliazione §3), non dalla nota. A potrebbe comprare 2 s come 12 s: differenza che decide la scommessa.
4. **B è coerente coi numeri firmati** (memops 5,4 s + churn 4,4 s + parte di vm_inline drop-glue) e non tocca RetainSet né destructor timing. Ma B da sola non compra la parità (~10 s su 37,6): la sequenza plausibile resta B-poi-A, da confermare col census.
5. **Manca il profilo lato oracle** (§7.2): senza, i canali che anche Zend paga (map, memops) sono sopravvalutabili — feedback-one-sided-profile.

§Emendamenti
- **R1 (destructor determinism)**: in A il `__destruct` resta refcount-driven nel punto esatto di fine-vita, MAI delegato allo sweep. Misura: fixture `__destruct`+echo con output-capture attiva; gate corpus per NOME sui test d'ordine di drop.
- **R2 (RetainSet fuori arena)**: i payload che sopravvivono alla richiesta non vivono nel pool per-request, o vengono evacuati/pinnati; handle con generazione, e il costo del generation-check ENTRA nel modello del costo sostitutivo. Misura: fixture parità per-richiesta WP (2ª richiesta byte-id).
- **R3 (pool, non arena)**: A ridefinita come slab con riuso intra-request; obbligo di A/B contro mimalloc (già slab, 8–15 ns/coppia) su objalloc/objchurn, criterio pre-registrato REGOLE §3.
- **R4 (ordine del confine)**: ogni sweep residuo per-request (cicli/leak) eseguito DENTRO request_end() DOPO la cattura output — il binding non si emenda, si implementa.

§Veti (Q3)
- NaN-boxing: **CONFERMA** (B = Option+niche by-value, non NaN-box).
- Contenitori sul call path: **CONFERMA**; il deref di handle deve essere indice O(1) in slab, mai hash; disasm prima/dopo sul run_loop obbligatorio.
- Alloc-removal senza modello del costo SOSTITUTIVO: **CONFERMA** — è il cuore di R3; nessuna promozione di A senza il prezzo di handle+generazione firmato.
- SSO inline: **CONFERMA** (str 0,8%: B non deve reintrodurlo).
- Leva GC note-time (WP-21): **CONFERMA**; A che azzera la nota per gli oggetti in pool è rimozione strutturale, NON riapre la leva sul tempo della nota.
- Notti su PhpStr-full: **CONFERMA** (fuori bersaglio).

§Kill-switch (Q4)
- **K1 (istruttoria, 1 sessione)**: se il census CH_* dà quota oggetti <30% delle coppie, A perde il titolo di headline → B sola prima; giudice: census monobinario datato, r1==r2.
- **K2 (istruttoria, 1 sessione)**: profilo oracle per famiglia; se Zend paga quota comparabile su map/memops, quei canali si scontano PRIMA della scelta.
- **K3 (B, ≤3 sessioni dall'apertura)**: A/B pre-registrato su objchurn/objdatains, segno 5/5; se la suite ORM non esce dalla banda ±0,7% a B completa → B ridimensionata, non estesa.
- **K4 (A, ≤4 sessioni dal prototipo)**: se su objalloc il costo sostitutivo (handle+generation-check) mangia ≥50% del rimosso, o un solo fail per NOME su ordine `__destruct`/RetainSet, o la fixture parità 2ª richiesta rompe → STOP A, si tiene B.

════════ concilio/verbale-leijen.md ════════
VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — census CH_* per classe + sonda prezzi + bilancio bytes (1 sessione), poi A-poi-B condizionata all'esito.

§Analisi (lente allocatore/footprint)
1. Il dossier sopravvaluta il canale ns/coppia. Il fast-path mimalloc (pop thread-local) sta a 4–8 ns; un bump-allocator sta a 2–4. Su 471M coppie il guadagno DIRETTO di un'arena è ~1–3 s, non 26–28. La famiglia alloc leaf è 2,3 s (§2): anche azzerandola tutta, il rapporto resta >7. Quindi l'opzione A NON si giustifica come «arena batte mimalloc»: si giustifica solo se compra i canali adiacenti — dec/inc Rc nel churn (4,4 s), drop-glue in vm_inline (quota dei 7,0 s), nota GC obj (56,5M), e la pressione cache dei 29,4 GB mossi — tutte voci a magnitudine NON ripartita (REGOLE §4). La delibera «A adesso» poggerebbe interamente su grandezze non misurate: buco nominato.
2. Il vero regalo dell'arena non è ns/coppia ma (a) eliminazione del FREE individuale (bulk reset: metà della coppia + niente purge/deferred mimalloc) e (b) località: mimalloc sparge per size-class page, l'arena serializza i payload che l'ORM tocca insieme — il bersaglio è memops 5,4 s e la coda cache-bound di «other» 26,6%. Non prezzabile oggi: il dossier lo ammette (§3 ultima riga). Serve il prototipo-giudice, non la fede.
3. Incoerenza di bilancio: free 33,8 GB > alloc 29,4 GB per run. O il census conta i realloc due volte (9,6 GB mossi) o c'è doppio conteggio nei path di drop. Prima di prezzare qualunque cosa sui bytes, il bilancio deve chiudere.
4. Footprint: un'arena per-request è HIGH-WATER — ciò che muore a metà richiesta resta vivo fino al reset. Su ORM (migliaia di entità/hydration per request) il picco può esplodere e lo shrink −70 MB conquistato è un vincolo storico, non negoziabile. A senza riuso interno o senza gate footprint è inammissibile.
5. Costo sostitutivo di A: l'handle aggiunge un deref (load dalla tabella) su OGNI accesso payload; su propget 29,9M+ letture è un prezzo reale, da modellare per iscritto prima del prototipo (veto pertinente, sotto).

§Emendamenti
R1 — Census CH_* per classe E per taglia: quota oggetti/props dei 471M e dei 29,4 GB. Decide A vs B coi numeri, non a lettura. Giudice: monobinario census, ×2 repliche.
R2 — Sonda monobinaria prezzi alloc/free reali (classe S-138): sostituire l'8–15 ns «plausibile» con una misura firmata.
R3 — Chiudere il bilancio bytes (free>alloc) prima di ogni prezzo sui GB.
R4 — Gate footprint permanente su A: vmmap Physical footprint (mai RSS) su WP e ORM, soglia pre-registrata.
R5 — Modello scritto del costo sostitutivo di A (deref handle × conteggi propget/recv_clone) PRIMA del prototipo.

§Veti (Q3)
NaN-boxing: CONFERMA (B = Option+niche; se scivola verso NaN-boxing il veto morde). Contenitori sul call path: CONFERMA — la tabella handle sia slab/indice diretto, mai HashMap. Alloc-removal senza modello del costo SOSTITUTIVO: CONFERMA, ed è il cuore di A (R5: deref + high-water SONO il costo). SSO inline: CONFERMA, fuori perimetro. Leva GC note-time (WP-21): CONFERMA — il dossier stesso mostra nota 0,5–1,2 s vs sweep dominante; A la riduce solo come sottoprodotto. Notti su PhpStr-full: CONFERMA, stringhe fuori da A/B.

§Kill-switch (Q4)
KS1: census CH_* — oggetti+props <30% delle coppie E <30% dei bytes ⇒ A retrocede, B prima. Giudice: census; 1 sessione.
KS2: prototipo A su micro oggetti — guadagno ABAB <15% su objchurn/objalloc (o segno opposto, R=5, banda) entro 3 sessioni ⇒ A si ferma.
KS3: footprint — WP > baseline+10% o picco ORM > +20% (vmmap) ⇒ stop/redesign riuso.
KS4: suite ORM post-promozione — rapporto net non sotto 8,0 fuori banda ±0,7% entro 2 sessioni dal prototipo promosso ⇒ lettura ciclo-di-vita falsificata per la quota oggetti.

════════ concilio/verbale-stogov.md ════════
VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: B-poi-A — B è semanticamente invisibile e ridefinisce la taglia/ABI dello Zval su cui A deve poi fissare l'handle; A com'è scritta nel dossier è semanticamente infondata e va riprezzata dopo l'istruttoria §7.1.

§Analisi (lente: semantica Zend/engine)
1. Il buco del dossier che nomino: «A azzera il dec/inc Rc nel churn» (§5) è FALSO rispetto a Zend. Zend NON azzera il refcount sugli oggetti: zend_object porta GC_REFCOUNT, ogni copia di zval obj fa GC_ADDREF/GC_DELREF, e il __destruct scatta a DELREF→0 (zend_objects_store_del). Togliere inc/dec sposta il destruct a fine request: la divergenza §3.22 (oggi UN caso catalogato su unset di elemento) diventerebbe SISTEMICA, e WeakReference::get() restituirebbe oggetti che l'oracle dichiara morti (corpus: `Zend/tests/weakrefs/*.phpt`, gh10043-001..010 in testa). L'acquisto onesto di A è: coppia alloc/free (quota oggetti dei 471M, IGNOTA fino al census per classe) + località + parte della nota obj — non il churn Rc.
2. A è comunque Zend-shaped: gli oggetti Zend sono GIÀ handle+store (EG(objects_store).object_buckets indicizzato da handle uint32, free-list di riuso; spl_object_id È l'handle). `===` via handle è sano. Ma il RIUSO degli handle dopo la morte è osservabile (spl_object_id ripetuti): un'arena che non ricicla gli handle diverge in modo lieve ma catalogabile PRIMA.
3. Ciò che Zend davvero non paga e phpr sì: la nota GC PER-MOVIMENTO (238,6M eventi). Zend annota solo possible-root a DELREF che non arriva a zero, bufferizzato. Questa terza componente non è né A né B nel dossier e non coincide col veto WP-21 (che era una leva di timing dentro lo schema esistente): va NOMINATA nella scommessa strutturale.
4. B (Zval by-value+niche) non tocca superficie semantica: nessun test del corpus a rischio per NOME. Ma il suo prezzo netto è sopravvalutabile: anche Zend paga memcpy da 16B e memops di hashtable — senza il profilo lato oracle (§7.2, regola one-sided) i 5,4+4,4 s non sono un tetto di acquisto. La direzione resta firmata dalle tasse §4 (Field* ~10×, costo/op invariante).

§Emendamenti
R1. A conserva il refcount stile Zend (arena = solo alloc/free + località); riprezzarla su questa base DOPO il census CH_* per classe (§7.1). Misura: quota oggetti dei 471M.
R2. Gate semantico pre-registrato per A: weakrefs/*, spl_object_id-riuso, fixture §3.22, destructor-set — fail-set per NOME invariato; ogni divergenza nuova a catalogo PRIMA della promozione, mai dopo.
R3. Nominare la componente gc-note→possible-root-at-decrement come voce propria del budget (0,5–1,2 s nota + parte famiglia gc), con A/B proprio.
R4. Profilo oracle per famiglia (§7.2) PRIMA di prezzare B; B parte subito solo come progettazione+criterio, la promozione aspetta il profilo.

§Veti (Q3)
NaN-boxing: CONFERMO (B via niche non lo richiede). Contenitori sul call path: CONFERMO (l'arena non ne introduce; vietato che A lo faccia). Alloc-removal senza modello del costo SOSTITUTIVO: CONFERMO e lo APPLICO ad A — bump/free-list, sweep di fine request, RSS e RetainSet (binding output-capture) vanno modellati prima del giudizio. SSO inline: CONFERMO (stringhe fuori perimetro). Leva GC note-time WP-21: CONFERMO sul timing; EMENDO il perimetro: la ristrutturazione possible-root (R3) è altra cosa, ammessa solo con criterio pre-registrato. Notti su PhpStr-full: CONFERMO.

§Kill-switch (Q4)
B: falsificata se, a leva spedita, churn_zval+memops (profilo campionario S-140, 2 repliche) non calano ≥25% relativo E la suite ORM resta dentro banda ±0,7% — entro 3 sessioni; giudici: profilo campionario + coppia ORM.
A: falsificata in istruttoria se il census CH_* dà agli oggetti <15% delle 471M coppie, o se il modello del costo sostitutivo mangia >50% del guadagno stimato — entro 2 sessioni; giudice: census monobinario + modello scritto. Gate semantico: un fail nuovo per NOME in weakrefs/destructor non catalogabile ⇒ STOP immediato.

════════ concilio/verbale-gregg.md ════════
# Verbale sedia GREGG — lente: metodologia di misura e attribuzione (S-143)

VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — 1 sessione timeboxed (census CH_* per classe + profilo oracle per famiglia), con regola di decisione A-vs-B PRE-REGISTRATA prima di leggere i dati; la direzione «ciclo di vita» è già firmata bilateralmente, la RIPARTIZIONE A/B no.

## §Analisi (mandato inverso: cosa sappiamo oggi che ieri non sapevamo)
1. Sappiamo, con quattro falsificazioni pre-registrate consecutive (dim-write Δ≈0, HC1 0,13%, L-RD1 0,24–0,53%, teardown=canale intero ~2%), che il divario ORM è DIFFUSO: nessun sito supera la risoluzione della coppia (±0,7%). È conoscenza positiva di grado A/B: legittima da sola l'abbandono delle micro-leve.
2. Sappiamo (S-129, bilaterale, chiusura 96%) che la tassa per-statement è ~10× QUASI INVARIANTE per forma, e (S-103) che il costo/op del loop è fermo a 9–10 ns. Questa è l'unica evidenza BILATERALE che punta al ciclo di vita per-valore — ed è la vera base della scommessa, non il profilo.
3. Il resto NON regge da solo una scelta tra A e B: il profilo §2 è campionario grade=INDIZIO a un lato solo; i «26–28 s cumulati» sono una SOMMA di magnitudini non ripartite — REGOLE §4 vieta di trattarla come cifra; i prezzi §3 sono «plausibili» con banda ~2× (alloc 3,8–7,1 s); «other» 26,6% (11,3 s) è la voce più grande del profilo e non ha nome.
4. Quindi: la scommessa strutturale in sé è deliberabile OGGI (direzione+meccanismo firmati). La scelta A-vs-B no: A compra una QUOTA IGNOTA dei 471M (census per-classe assente); B bersaglia memops+churn (9,8 s) che sono INDIZIO unilaterale — se anche Zend paga memcpy in proporzione simile, B compra meno del previsto (feedback-one-sided-profile: prerequisito, non rifinitura, per B; per A il census CH_* è prerequisito per definizione — «quota da censire» lo dice il dossier stesso).

## §Emendamenti
- **R1 — Regola di decisione pre-registrata** (≤10 righe, PRIMA del census): se quota oggetti+Rc dei 471M ≥ soglia dichiarata (proposta: ≥35% coppie o ≥10 s ai prezzi firmati) → A-poi-B; sotto → B-poi-A. Niente lettura dei dati prima della firma della regola.
- **R2 — Prezzi firmati, non plausibili**: sonda monobinaria classe S-138 su alloc/free e gc_note (§7.4) DENTRO l'istruttoria; senza prezzo firmato il budget di A resta una banda 2× e il kill-switch non ha giudice.
- **R3 — «other» 26,6%**: la riquantificazione S-141 va chiusa o la voce dichiarata fuori-budget della scommessa; una struttura che promette 26–28 s con 11,3 s senza nome ha il denominatore scoperto.
- **R4 — Vertical slice come primo atto della via scelta**: arena/layout su un perimetro nominato (oggetti senza __destruct/weakref), giudicato sulla SUITE ORM, mai solo sulle micro (le micro hanno già ingannato quattro volte in direzione opposta).

## §Veti (Q3)
- NaN-boxing: CONFERMA (B = Option/niche, safe-only; NaN-boxing non necessario né ammesso).
- Contenitori sul call path: CONFERMA; l'handle di A è un'indirezione sul cammino caldo — cade sotto lo spirito del veto: va prezzata nel modello sostitutivo.
- Alloc-removal senza modello del costo SOSTITUTIVO: CONFERMA e RAFFORZO — è il cuore di A: bump+deref+sweep per-request vanno prezzati PRIMA dell'A/B, col binding output-capture intatto.
- SSO inline: CONFERMA (fuori perimetro A/B).
- Leva GC note-time (WP-21): CONFERMA — la famiglia gc è sweep-dominata (§3); A può contare la nota obj 56,5M solo come collaterale, mai come canale giustificante.
- Notti su PhpStr-full: CONFERMA.

## §Kill-switch (Q4)
- **K1 (istruttoria, 1 sessione)**: census CH_* r1==r2 <1%; se quota oggetti sotto soglia R1 → A decade a seconda via, senza appello.
- **K2 (prezzi)**: se la sonda R2 firma alloc/free tale che il budget A < 2 s → il canale alloc di A è falsificato; giudice: sonda monobinaria, stessa sessione.
- **K3 (bilaterale)**: se il profilo oracle mostra memops/churn in proporzione comparabile a phpr → i bersagli di B non sono divario → B decade; giudice: stessa lente sui due motori, net-pavimenti per-binario.
- **K4 (slice)**: la via scelta deve muovere il rapporto ORM di suite ≥5% (fuori banda ±0,7%, 2 gambe/lato) entro 5 sessioni dalla prima promozione, o si dichiara falsificata e si torna al concilio. Niente proroghe implicite: la quinta sessione emette verdetto.

