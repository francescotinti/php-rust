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
