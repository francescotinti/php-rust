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
