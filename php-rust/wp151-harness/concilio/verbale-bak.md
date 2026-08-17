# Verbale sedia BAK — S-151 (lente: VM di produzione; alloc-rate, I-cache/BTB, path caldi)

`VERDETTO: CONCORDO CON EMENDAMENTI` (impianto A1..A4 ratificabile; la FORMA di A3 e il
perimetro di A2 no, così come scritti nel parere Gemini).

## La fisica di A3, letta dal codice (non dal report)
- Oggi: `Zval::Object(Rc<RefCell<Object>>)` (php-types/src/zval.rs:41); il cammino caldo è
  deref-Rc → borrow-flag (load+2 store) → slot. Domani (store a indici): base-store in registro
  (il loop possiede già `&mut Vm`) → shift+add → bounds-check → slot. La catena di load
  dipendenti è PARAGONABILE; il bounds-check è un branch never-taken quasi perfettamente
  predetto — NON è il pattern WP-44 (lì il `match` su enum operando era branch dati-dipendente,
  4–8 per opcode: BTB in tilt). Il precedente WP-44 NON refuta A3; non lo benedice neanche.
- MA: il vincolo semantico (timing distruttori osservabile ⇒ refcount ESPLICITO) rende A3 un
  modello CPython, non V8. V8 ha handle+tracing-GC proprio perché ha ABOLITO il refcount; noi
  non possiamo. Quindi il traffico inc/dec sui movimenti di handle RESTA anche dopo A3 —
  coerente col TETTO movimenti 1,27 s ≈ 3,4%: quel canale è cappato PRIMA e DOPO la chirurgia.
- Tensione TETTO↔Gemini: RISOLTA CONTRO Gemini. Il pilastro «azzerare il traffico di movimento»
  (§2.2-1, «-35% su Doctrine», «40% in Rc churn») poggia su ObjectId Copy senza refcount, già
  vietato dal vincolo semantico; con refcount esplicito il guadagno da movimento è ≤3,4%.
  Il caso A3 si regge SOLO sui canali non-movimento: borrow-flag RefCell, conteggio allocazioni
  a costruzione (Rc header + Vec props + Vec dynamic), gc_note su handle, località/arena.
  Ciò che l'esperienza V8 firma davvero: in-object properties (= props inline, pilastro 3) e
  bump-alloc stile TLAB per carichi alloc-densi (objalloc 6,4×, objchurn 6,7×). Il pilastro
  «store a indici» è quello a evidenza PIÙ DEBOLE; props-inline quello a evidenza più forte.
- Costo nuovo che Gemini non prezza: la RE-ENTRANCY. `__get`/hook/`__destruct` eseguono PHP
  mentre un accesso allo store è in corso; oggi RefCell la trasforma in panic dinamico, domani
  la disciplina `&mut Vm` impone re-lookup post-call o take/put-back (= copie reintrodotte).
  Va nominata nel criterio di A3c, con prezzo.

## Q1 — sequenza
A1→A2→A3 GIUSTA, con emenda: i numeri DECISIONALI di A1 devono essere per CANALE
(specie×operazione), che sopravvivono al refactor; l'inventario per-SITO va dichiarato
DEPERIBILE a ogni tranche A2 (il refactor sposta i siti). L'alternativa «interleaving» va
adottata nella forma R3: refactor SOLO dei moduli che la chirurgia toccherà.

## Q2 — gate delle tranche A2
Gemini dichiara la Fase 1 «rischio zero»: REFUTATO dai nostri stessi atti — FR1 ha mostrato
+3,00 ns/iter da solo delta STRUTTURALE (+3180 B/+26 bl, S-150). Spostare 25k righe cambia
inlining e layout: il prezzo esiste. Gate per tranche: batteria (rc dal comando) + corpus
1412×2 per NOME + fixture bilaterali + micro R=5 a SOLO-REGRESSIONE con banda-layout +
**disasm di run_loop (istr/bl) prima/dopo come sentinella di layout** — è questo che
sostituisce la byte-identità vietata. Coppia WP: a OGNI pin nuovo (regola utente, non
negoziabile). Partizione a rischio minimo: prima il codice FREDDO (reflect/dom/system da
host.rs, census, builtin dispatch tables), per ultimo (o mai, in A2) ciò che sta nel raggio
d'inlining di run_loop; funzioni spostate VERBATIM, `#[inline]` e firme intatte.

## Q3 — forma di A3 e canali decisivi del census
Forma: A3 monolitica NO. Scomposta in tre atti promo-gated separati (R2). Cosa si rompe:
- `WeakHandle(std::rc::Weak<RefCell<Object>>)` (zval.rs:62): lo store deve dare weak-table
  con generazione, o gli id riusati creano ABA (PHP riusa gli id: host.rs:2178).
- Tutta la macchineria su `Rc::strong_count`: created map (mod.rs:3270), buffer distruttori
  in NOTE ORDER = free-order Zend (mod.rs:3283), cycle collector strong_count vs in-edges
  (mod.rs:4718–4986), gc_cascade, get_gc: da RISCRIVERE sul refcount esplicito preservando
  l'ORDINE delle note (il free-order è osservabile: object_reference.phpt).
- Ref/Resource/Generator/Closure RESTANO Rc: il churn su quelle specie non lo tocca A3.
- spl_object_id NON si rompe (id già campo di Object). Re-entrancy: v. sopra.
Canali del census DECISIVI (aritmetica = prezzo×conteggio, lezione HC1: 35,6M = 0,13%):
1. clone/drop di handle Object per sito (inc/dec) — GIÀ cappato: 1,27 s ≈ 3,4%. Serve solo
   la conferma di tranche-5; NON può motivare A3c.
2. borrow/borrow_mut su Object per sito × prezzo sonda (~1–2 ns): l'unico canale che lo
   store-senza-RefCell azzera davvero. NON è sotto il tetto movimenti: va contato a parte.
3. allocazioni per costruzione oggetto (n. malloc/oggetto × prezzo malloc misurato):
   decide props-inline+arena (A3a). Denominatore: oggetti costruiti nella suite ORM.
4. gc_note con argomento Object per sito × prezzo gc_note_slow (borrow+insert): quota
   evitabile su puro movimento con refcount esplicito.
5. clone/drop di VALORI Zval a PropGet/PropSet (memcpy 16B + eventuale Rc payload): decide
   borrow-first (A3b) — è qui che vive il «44% clone INLINE da run_loop» del census S-140,
   e NON è traffico di handle.
DECIDIBILITÀ: A3c (store) si apre SOLO se UB = Σ(2)+(4)[+resto di (1) sotto tetto] supera una
soglia PRE-REGISTRATA PRIMA di leggere A1 (proposta: ≥5% del gap ORM su binario census, ~2×
il miglior esito storico di una leva). Sotto soglia ⇒ A3 = A3a+A3b soltanto, veto stile
NaN-boxing su A3c. Attesa di suite SEMPRE col pavimento dichiarato (lezione S-150).

## Q4 — dente A4
Sede: BATTERIA (morde a ogni promozione; la CI ha backlog ~3 giorni = non morde in tempo,
resta advisory; pre-commit hook NO: pipeline composte passano raw, path con spazi — lezione
forge-silent-failure). Perimetro: cap assoluto ~2.000 righe sui file NUOVI + ratchet
DECRESCENTE sui monoliti esistenti (cap = righe correnti al pin, aggiornato solo al ribasso a
ogni tranche A2): senza ratchet il dente o non morde mai o morde subito su mod.rs 25,7k.
Meccanica: conteggio righe via fs, NIENTE pattern-matching (auto-morso bea7ea3).

## Q5 — cosa manca (mandato inverso)
1. **BT1 (−6 s, la leva più grande della storia recente) NON era memory-model: era un outlier
   ALGORITMICO trovato dal census per-NOME.** Prima di ipotecare 4–6 sessioni, A1 deve
   ripetere la caccia all'outlier sulle teste nuove (none.other 94,6M, class_exists 9,7M,
   __reflect_* 12,4M): un secondo BT1 vale più di A2+A3 a rischio quasi zero.
2. Census UNILATERALE (⭐⭐ one-sided-profile): senza pavimenti/profilo oracle per categoria i
   rapporti attesi post-A3 non sono calcolabili. Minimo: pavimenti oracle per canale prezzato.
3. Nessuna soglia di decidibilità A3c pre-registrata: senza, i numeri A1 verranno letti a
   posteriori con ragionamento motivato. Va scritta PRIMA del run del census.
4. Nessuno STOP pre-registrato per A2 (v. kill-switch KS-1).

## Emendamenti
- **R1**: a verbale: il pilastro «azzerare il movimento» è CAPPATO (1,27 s ≈ 3,4%, binario
  census); nessuna cifra Gemini (-35%, 30–45%, 40%) entra in alcun criterio finché non firmata
  dai nostri strumenti. Il caso A3 si istruisce sui canali 2/3/4/5.
- **R2**: A3 scomposta: A3a props-inline+riduzione alloc (canale 3) · A3b borrow-first sui
  valori (canale 5) · A3c store a indici senza RefCell (canali 2+4) — ciascuna promo-gated,
  ordinata dai numeri A1; A3c sotto soglia pre-registrata NON si apre.
- **R3**: A2 a perimetro MINIMO: solo i moduli che A3a/b/c toccheranno (gc, object/props,
  ops-objects, dispatch hostcall) + dente A4; la mappa estetica completa di Gemini §4.3 non è
  un obiettivo di sessione. Riduce le sessioni-senza-leva e la superficie di rischio layout.
- **R4**: gate di tranche come in Q2, con disasm run_loop (istr/bl) sentinella obbligatoria;
  «rischio zero» di Gemini Fase 1 respinto agli atti (precedente FR1).
- **R5**: dente A4 come in Q4 (batteria + ratchet decrescente + conteggio fs).
- **R6**: dentro A1: scan outlier per-NOME sulle teste hostcall nuove PRIMA di aprire A2.
- **R7**: pavimenti oracle per categoria nel fascicolo A1 (bilateralità minima).
- **R8**: il criterio di A3c nomina e prezza la re-entrancy (re-lookup post-call / take-put-back)
  e la weak-table con generazione; attese con PAVIMENTO dichiarato.

## Kill-switch pre-registrabili
- **KS-1 (A2)**: tranche con micro fuori banda-layout o Δbl run_loop oltre banda dichiarata,
  non riparata entro 1 sessione → freeze del refactor, si riparte dall'ultimo pin buono.
- **KS-2 (A3c)**: UB(canali 2+4) dal census < soglia pre-registrata → A3c chiusa senza appello
  (stessa classe del veto NaN-boxing); restano A3a/A3b.
- **KS-3 (A1)**: scarto s149↔s150 (+3,2%) non istruito entro la sessione census → i numeri A1
  declassati a INDICATIVI: nessuna decisione A3 su di essi.
- **KS-4 (A3a/b)**: A/B della prima leva del filone sotto soglia del criterio → il filone torna
  in istruttoria, non si «compone sopra» (feedback keep-partial-wins: il guadagno misurato
  resta, l'ipotesi no).
