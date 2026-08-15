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
