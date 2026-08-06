# WP_SESSION_101 — S-101: l'ordine WP-102 su H-C eseguito — census che arbitra, due leve promosse dai loro criteri, proprietà 12,4× → 11,5×

**In una frase**: abbiamo misurato con precisione DOVE il nostro motore PHP
perde tempo quando un programma legge e scrive le proprietà degli oggetti, e
con due interventi mirati — entrambi decisi dai numeri raccolti prima, e
verificati con migliaia di test e col sito WordPress completo — l'accesso
alle proprietà è diventato circa il 7% più veloce senza cambiare alcun
comportamento visibile.

**Data**: 2026-08-06 (00:0x–03:0x). **Modello verificato all'apertura**:
Fable 5. **Ordine eseguito**: Concilio WP-102 §S-101 punti 1, 2, 3a, 3b, 3c,
5, 6-parziale (punto 4 in handoff — nessuna finestra per ~10 run full
aggiuntive dopo la coppia). **Commit**: 725d5a1 → 437a2ca → d4bbb96 →
5baa369 → f9e9f22 → d3d50ca → c7815c2 → 19803fb → f808017, tutti su main,
pushati.

## Ordine eseguito

| # | Esito |
|---|---|
| **1 · Ri-baseline sei categorie (modo default)** | Prima finestra VOID (spread arith 2,53 s sotto burst Chrome-RD — regola banda<rumore, conservata come `micro-baseline-s101-r1-VOID.out`); seconda finestra PULITA: **arith 12,7 · prop 12,4 · calls 7,9 · str 7,1 · arr 4,6 · re 3,6** (prop == S-100 al centesimo: 5,22/0,42) |
| **2 · Census dinamico specie×sito×canale** | Contatori S-101 in `zvalcensus.rs` (riga separata `zvalcensus_s101`; la riga storica resta byte-identica). **P1 CONFERMATA** (0 valori refcounted sui 3 canali); **P2 CONFERMATA E AGGRAVATA** (ricevitore = 6 coppie clone+drop Rc/iter: 3 load + 3 deref_clone — il dinamico ha MORSO lo statico: il primo LoadSlot carica `$o`, non `$s`); **P3 RAFFINATA** (4 gc_note/iter, non 3 — il residuo è `reg_store_slot`; 100% su Long). alloc/iter≈0 (mimalloc stats). Linearità 300:1 esatta su prop_small. Oracle verificato per NOME (`zend_jit_zval_copy_deref`: addref solo sotto `Z_OPT_REFCOUNTED`). **Profilo INLINE-AWARE** (samply+dSYM+atos -i, 1:1 su 426 indirizzi): il 50% di run_loop APERTO = 21,2% dispatch/corpi + **~26,6% meccanica della pila operandi** (as_slice/len/pop/push su Vec<Zval>); ciclo di vita Zval confermato 28,2%. Le tre predizioni rivali di sedia sono TUTTE vere e sommano |
| **3a · Fixture semantiche** | 13 fixture in-tree (`wp101-harness/hc1-fixtures/`, attese in testa PRIMA): specie int/string/array, mutazione interlacciata re-entrante, __get+hook 8.4, lazy ghost, ref nello slot, riassegnazione intra-espressione, unset-durante-lettura, readonly/typed-uninit, scrittura in-place di pila, ordine __destruct, ciclo GC. **13/13 PASS nei 2 modi**; trovata e catalogata **§3.13** (warning undef-prop attribuito alla riga SUCCESSIVA — pre-esistente, identica nei 2 modi; carve-out per NOME a diff ESATTO nel runner) |
| **3b · H-C1a bypass scalari** | Criterio PRE-registrato (banda attesa [4,13] ns/iter dal canale contato). Leva: split `gc_note` in guardia `#[inline]` (specie contenitore) + `gc_note_slow`. **Δ=7,3 ns/iter** (A/B interleaved R=7 contro il pin, B al centesimo), **prop 12,4→11,9**; controllo positivo: census byte-identico. Gate: fixture 13/13 + batteria 1735/0 + corpus 1418×2 per NOME + diff per-test ZERO |
| **3c · H-C1b canale ricevitore** | Forma decisa DAI CONTEGGI: **MOVE dell'handle owned** (niente prestito, niente addref — il sigillo di tipo non serve: nessun borrow attraversa un confine di op; il braccio Ref conserva deref_clone). **Δ=6,0 ns/iter ≥ pavimento MA SOTTO la banda attesa [7,20]**: la sovrastima contabile (~2,0 ns/coppia effettivi vs 4,4 stimati dal profilo) è REGISTRATA. Cumulato **prop → 11,5**. Controllo positivo: `recv_clone_prop` 90M→**0**. Gate: fixture + batteria full **1737/0** + corpus 1418×2 + diff ZERO + **coppia WP bimodale**: media 0 fail IDENTICI, full == SOLO il delta pre-esistente S-100 (`test_wp_is_stream ftp://`), invariante di modo |
| **5 · Denti residui** | A-KL-102-3 (assente≡`=1`: stesso modo E stesso `{main}`) VERDE al primo colpo; **A-HE-102-1 ha morso il suo AUTORE**: prima stesura a polarità invertita (produzione OFF = `BinaryAdd` DIRETTO da `emit_binary`, non `Binary(Add)`) — corretta leggendo il codice, ora pinna il sintomo vero del bug A-HO-102-1 |
| **6 · H-D (timebox)** | Prima pietra CENSUS su calls.php: **5 gc_note/iter tutte scalari** (2 Pop + 3 siti da nominare), 1 slot_read/iter, zero canali proprietà (`hd-census-primo.out`); la guardia H-C1a morde anche qui — **calls 7,9→7,3 nella ri-baseline di chiusura**. Census bi-regime call-path completo → S-102 |
| **Chiusura · ri-baseline cumulativa** | Binario cumulativo (spread ≤0,03): **prop 11,5 · arith 12,2 · calls 7,3 · str 7,0 · arr 4,3 · re 3,5** — coerenza A/B al centesimo (prop netto 4,83 vs braccio A 4,81); coppia WP: full CPU ~1,89× (off 794,8/419,8 · on 792,2/419,2; S-100 1,873 — in banda tra-sere), media ~2,60× (S-100 2,639), peak full on 1863,8 MiB / off 1979,5 MiB (SOLO riferimento: il full-peak oracle ha ballato anche stanotte 720,9↔795,5 MiB intra-sera = rumore ~10% CONFERMATO) |

## 🔵 Scoperte

1. **Il churn del ricevitore era più grande del previsto**: 6 coppie
   clone+drop Rc/iter (il census ha corretto lo statico DUE volte: il primo
   LoadSlot carica `$o`; la quarta gc_note viene da `reg_store_slot`).
2. **Dentro run_loop il costo nominato non è un handler: è la meccanica
   della pila operandi** (~26,6% del totale phpr in accessor Vec inlined) —
   più grande del ciclo di vita Zval stesso. È la prossima gamba nominata.
3. **L'handle poppato è già owned**: il MOVE elimina il clone del ricevitore
   senza prestiti né sigilli — la forma più forte era anche la più semplice.
4. **La guardia scalari morde trasversalmente**: arith −0,5, calls −0,6,
   arr −0,3 — ogni categoria paga gc_note su scalari (calls: 5/iter censite).
5. **`liveness.rs` (feature-gated) sfuggiva al tripwire dei match esaustivi**:
   `BinaryAdd` (S-100) non compilava sotto `zval-census` — un modulo dietro
   feature non è protetto dalla batteria che gira senza feature.
6. **Il full-peak oracle è rumoroso ~10% anche intra-sera** (720,9↔795,5 MiB
   stanotte): terza conferma; ogni banda futura sul full-peak parte dal
   rumore misurato.

## ⭐ Lezioni

- ⭐⭐ **Un dente scritto senza leggere il codice che pinna morde il suo
  autore**: A-HE-102-1 prima stesura asseriva la polarità INVERSA di
  emit_binary (rosso in batteria) — la polarità VERA si legge nel corpo
  (`!ctx.reg_lower ⇒ BinaryAdd`), mai dedotta dal riassunto di un verbale.
- ⭐⭐ **Il census dinamico è un giudice, non una conferma**: due predizioni
  statiche su tre corrette in dettaglio (3 load non 2; 4 gc_note non 3) —
  ogni atteso contabile costruito sullo statico avrebbe sbagliato bersaglio.
- ⭐⭐ **L'attribuzione a campioni sui simboli inlined ha un errore di
  fattore ~2 A SEGNO IGNOTO** (banda H-C1b [7,20] vs 6,0 misurato; la
  forma «sovraconta» è stata refutata come legge dal Concilio WP-103,
  R-GR-103-1: n=1): il controfattuale si scrive dal canale contato, il
  costo/evento fa fede solo dall'A/B — e la refutazione della banda si
  registra, non si nasconde.
- ⭐⭐ **A/B interleaved contro il rumore remoto**: coi burst di Chrome
  Remote Desktop, alternare ABAB nella stessa finestra ha dato spread 0,04
  dove la prima finestra sequenziale era VOID (2,53).
- ⭐ **git add -u a valle di edit multipli mischia i cambi**: le righe
  H-C1b sono entrate nel commit dei gate di H-C1a (f9e9f22) — dichiarato
  nel commit successivo; lo staging si fa per FILE nominati.

## Stato binari e processi

- phpr: **48a5d4384970d8ff** @ HEAD f808017 (hash churna col relink: fa fede
  HEAD) — DEFAULT flag-ON; contiene H-C1a+b. Stash ADDITIVO `phpr-s101`.
  Batteria 1737/0 (coi 2 denti nuovi). Corpus 1418×2 per NOME + diff ZERO.
- php-server: **2c4242b6c8120b8e** (ricetta axum-server rispettata) — stash
  `php-server-s101`. **NON collaudato** (sentinella bimodale non eseguita
  in S-101): grado parziale nel registro; primo atto se si tocca il server.
- Nessun processo orfano; uploads ripristinata dalla guardia (2 entry
  utente); MySQL wp8 su.
- Harness di sessione: `wp101-harness/` (census, profilo inline-aware,
  fixture H-C1, criteri+A/B, corpus gate/diff, pair101, H-D).
