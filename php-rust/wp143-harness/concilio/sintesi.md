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
