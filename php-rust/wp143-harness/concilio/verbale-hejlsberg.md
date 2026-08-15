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
