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
