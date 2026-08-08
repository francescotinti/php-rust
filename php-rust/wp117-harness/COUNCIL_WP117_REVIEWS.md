# CONCILIO A 9 — DECISIONE DI ROTTA (convocato dall'utente in S-116, 2026-08-08) — VERBALI VINCOLANTI per S-117

**In una frase per non addetti**: dopo quattro sessioni in cui i ritocchi piccoli venivano bocciati dal rumore di misura, il concilio ha deliberato di cambiare prima la FABBRICA del programma (come viene compilato e impaginato), poi giudicare i miglioramenti a GRUPPI invece che uno a uno, e di aprire comunque il cantiere sul vero collo di bottiglia (il ciclo di vita dei valori PHP).

## §FONDAMENTALI
- **Oggetto**: arith 5,5 · prop 7,6 · calls 5,2 · str 5,3 · arr 4,2 · re 3,4 (obiettivo ≤3×); WP full 1,867. **4 sessioni senza leva spedita** (S-113..116): il regime mono-leva è saturo — banda layout fino a 10 ns/iter contro leve da 3-30.
- **Cosa sappiamo oggi che prima non sapevamo** (mandato inverso, Gregg): banda layout MISURATA con leve nulle (max 10 ns/iter, N=2, per categoria); costo/op ~9-10 ns INVARIANTE tra categorie ⇒ il collo è il ciclo di vita Zval, non le singole op; la tassa calls di L-A è SISTEMATICA oltre banda nulla (5 campioni); un gate a soglia fissa senza banda non è diagnostico; magnitudine L-A +29,33 ns/iter STABILITA (3 conferme).
- **Rischio d'oggetto più trascurato** (refutazione trasversale, 6 sedie): l'ARITMETICA del bersaglio. prop richiede ~−65 ns/iter per scendere a 3×; A′+L-A ne coprono −31..−45. **A+B da soli NON chiudono**: la «C in riserva» del conduttore è REFUTATA — il ciclo di vita Zval va istruito comunque (Zend vince con scalari rc-free, interned strings, COW — non con handler più furbi).

## CONVERGENZE (9/9 CONCORDO CON EMENDAMENTI; nessun MI OPPONGO)
1. **BOLT ESPUNTO: non esiste su Mach-O/Apple Silicon** (rilevato indipendentemente da TUTTE e 9 le sedie). La rotta A diventa **A′ = fat LTO + codegen-units=1 + ld64 `-order_file` + PGO rustc**, a stadi, con toolchain e profdata VERSIONATI. Oggi la release è SENZA `[profile.release]` tuning: lo stadio 1 è a portata.
2. **A′ = «build emendata»**: TUTTE le bande e baseline decadono. Prima di qualunque verdetto sul binario nuovo: ri-banda con ≥2 leve nulle micro + held-out. «Metro riparato» = banda_new ≤5 ns/iter (≈½ della vecchia); altrimenti A′ non ha riparato il metro e lo si dichiara.
3. **PGO**: profilo MAI addestrato sui sei giudici (né sugli held-out); deve includere request_end/teardown (Pedersen); profdata CONGELATO per entrambi i bracci di ogni A/B futuro (Leijen).
4. **B (treno)**: fedeltà/admission PER-VAGONE, perf PER-TRENO; manifest dei vagoni per NOME (cap 5); somma-bersaglio ≥2× banda; leva-nulla-treno di taglia comparabile; bisezione pre-registrata in caso di sfondamento; guardie giudicate sul treno intero (le tasse si sommano: netto ≥ −banda per OGNI categoria, Matsakis).
5. **C RISCOPATA, non in riserva**: NaN-boxing VIETATO (unsafe per costruzione — veto Hoare, conferma Pedersen su distruttori/RetainSet/capture-prima-del-reset). La via SAFE = **C-lite**: elisione refcount sugli scalari, borrow-non-clone, arene/handle generazionali — come VAGONI del treno. Istruttoria con harness contatori rc-op/alloc per categoria su ENTRAMBI i motori (team-engine) da S-118, comunque entro S-119.
6. **D ordinato dai contatori, non dall'intuito**: interned strings, inline cache, COW («D è C a rate», Stogov); VETO threaded-dispatch (Hejlsberg). Ogni vagone D dichiara classe lifecycle (interned per-processo, invalidazione IC) e strategia ownership.

## ORDINE S-117 (vincolante, dai 3 team)
1. **Spike A′ stadio-1** (timebox ½ sessione): `[profile.release]` lto=fat + codegen-units=1 (+ order_file ld64 se lo stadio regge); **determinismo provato: build ×2 → hash IDENTICO**; gate PIENI (batteria con rc in FILE · corpus per NOME ×2 · fixture · parità); poi stadio-2 PGO col profilo pinnato.
2. **Ri-banda su A′**: ≥2 leve nulle micro (metro riparato se max ≤5 ns/iter) + banda held-out N≥2.
3. **Rigiudizio L-A** (cherry-pick 2c18b2e) sotto A′ con le bande NUOVE — è anche il test della tassa calls; micro R=5 = nuove baseline; scoreboard rifondato.
**KILL-SWITCH (KS-A)**: uplift A′ mediano <2% E banda_new >5 ns ⇒ pipeline non rende né ripara: treno B sulla pipeline vecchia + istruttoria C ANTICIPATA a S-118.

## DISSENSI REGISTRATI (non appianati — restano a verbale per NOME)
- Statuto di C: Matsakis la vuole come vagoni GIÀ da S-118; Bak/Leijen/Gregg prima l'istruttoria contatori (entro S-119).
- Logica E/O dei kill-switch e soglie esatte KS-A (team-misura vs team-struttura).
- Workload del profilo PGO: Klabnik esige che NON coincida coi giudici; Pedersen esige che includa il teardown — da conciliare nel criterio S-117.

## Fonte vincolante
I verbali INTEGRALI delle 9 sedie + 3 note di team seguono in questo file (assemblati da `wp117-harness/verbali/`). In caso di conflitto tra sintesi e verbale individuale, VINCE il verbale individuale.

---


===== VERBALE SEDIA: hoare =====

# Verbale sedia Hoare — Concilio S-116→S-117 (lente: safe Rust, soundness, sigilli di tipo)

## VERDETTO: CONCORDO CON EMENDAMENTI

La raccomandazione (A subito / B regime / D metodo / C riserva) è giusta nell'ordine ma
sbagliata in DUE punti di sostanza dalla mia lente: contiene un componente inesistente
sulla piattaforma (BOLT) e lascia aperta in C una variante che violerebbe il sigillo
SAFE-only (NaN-boxing). Senza gli emendamenti R1 e R4 mi sarei OPPOSTO.

## ROTTA DALLA MIA LENTE (orizzonte 3 sessioni)

**A′ → B(+D) → C-perimetrata.**

- **A′ (S-117)**: PGO rustc (`-Cprofile-generate`/`-Cprofile-use`) + fat LTO +
  `codegen-units=1` + **order-file ld64** (`-order_file`) per layout deterministico.
  NIENTE BOLT: BOLT riscrive ELF, **non supporta Mach-O su Apple Silicon** — su questo
  Darwin non è una rotta, è una casella vuota. L'order-file ld64 è il sostituto nativo
  e per giunta più affine al problema vero (il metro che boccia le leve è
  l'impaginazione, banda misurata fino a 10 ns/iter).
  Dalla mia lente A′ è **priva di rischi di soundness**: PGO/LTO non alterano la
  semantica del programma safe (garanzia del compilatore, non del profilo) e i sigilli
  VmGate ZST sono fatti di TIPO a compile-time — nessuna passata di codegen li tocca.
  BOLT invece avrebbe bypassato il compilatore riscrivendo il binario: la sua assenza
  su Mach-O ci risparmia l'unico pezzo di A che avrei rifiutato.
- **B come regime, D come selezione dei vagoni**: sì, con R2 (fedeltà per-vagone).
- **C riserva**: sì, ma col perimetro R4 pre-registrato ORA, non quando la si apre.

**Mossa concreta S-117**: spike A′ in un atto solo stile REGOLE §2 — ricetta build
emendata in `scripts/pin-phpr.sh` (profilo raccolto su workload DETERMINISTICO: sei
micro + held-out + smoke WP; `.profdata` CONGELATO e versionato fuori repo, mai
rigenerato a ogni build), poi ripetere ESATTAMENTE la batteria di attribuzione S-114/115:
2 leve nulle → banda N=2 per categoria sul binario A′. Solo dopo, micro R=5 e gate pieni.

## EMENDAMENTI

- **R1 — Depurare A da BOLT.** Cosa: A = PGO+LTO+cgu=1+order-file ld64. Perché: BOLT
  non esiste su Mach-O; inseguirlo brucia la sessione. Misura: il criterio PRE di S-117
  nomina solo strumenti eseguiti con successo su questo host.
- **R2 — Nel treno B, fedeltà PER-VAGONE, perf PER-TRENO.** Cosa: ogni vagone passa da
  solo admission/parità/batteria/corpus 1415 per NOME; solo il cronometro è giudicato
  sulla somma. Perché: una somma promossa può nascondere la regressione semantica di un
  vagone; la parità di output non è additiva. Misura: gate fedeltà eseguiti a ogni
  aggancio di vagone, verbale per NOME.
- **R3 — A′ = «build emendata»: TUTTE le bande decadono.** Cosa: banda micro N=2,
  banda held-out, banda layout si RIMISURANO sul binario A′ prima di giudicare
  qualunque leva (L-A inclusa). Perché: cgu=1+LTO cambia inlining e impaginazione; la
  tassa calls può cambiare segno. Misura: 2 leve nulle sul pin A′, banda pubblicata.
- **R4 — Perimetro safe di C, pre-registrato.** Cosa: se C si apre, la variante ammessa
  è arena per-richiesta + handle indicizzati generazionali + elisione refcount su path
  caldo. **NaN-boxing VIETATO**: impacchettare puntatori in bit di f64 richiede
  transmute/provenance-cast — unsafe per costruzione, rompe il sigillo VmGate.
  L'arena safe converte l'use-after-free in use-after-recycle (bug logico, non UB):
  i generation counter sono parte del design, non un optional. Misura: nessun `unsafe`
  nuovo (grep di gate già esistente), batteria+corpus invariati.

## KILL-SWITCH (pre-registrati)

- **KS-A1**: se dopo A′ la banda globale delle nulle non scende (≤ metà dell'attuale
  10 ns/iter su N=2) E il mediano delle sei micro non migliora ≥2%, A′ decade a fine
  S-118: si tiene solo ciò che ripara il metro, si passa a B su build corrente.
- **KS-A2**: se il `.profdata` non è riproducibile (due raccolte → layout con banda
  diversa oltre N=2), il PGO si sospende e resta solo order-file+LTO.
- **KS-B**: treno di 3 vagoni sotto la soglia di somma pre-registrata sui giudici, o
  un vagone che fallisce fedeltà → il vagone esce, il treno non muore.
- **KS-C**: C si apre SOLO se dopo 3 sessioni di A′+B il peggiore resta >3×.

## APPARATO minimo

Solo l'emendamento della ricetta in `scripts/pin-phpr.sh` (R3 lo esige: pin/stash
nascono già collaudati sull'atto nuovo). Nient'altro blocca l'oggetto.


===== VERBALE SEDIA: matsakis =====

# Verbale sedia Matsakis — S-116/117, lente ownership/aliasing/borrow

## VERDETTO: CONCORDO CON EMENDAMENTI

A subito è giusto per la ragione giusta (ripara il METRO, non il divario); B come regime regge solo con una clausola anti-tassa; D come selezione va bene con traduzione di ownership obbligatoria; **«C solo se dopo A+B» è l'errore**: l'aritmetica già disponibile dimostra che A+B non bastano.

## ROTTA DALLA MIA LENTE (3 sessioni)

Ordine: **A (S-117) → B+C-incrementale come unico treno (S-118/119) → D come fabbrica di vagoni**. C non è riserva: è la sorgente dei vagoni grossi.

L'aritmetica che refuta «C in riserva»: prop è 7,6× ⇒ oracle ≈14 ns/iter, phpr ≈107; il 3× vale 42 ⇒ servono **−65 ns/iter**. A rende 5-15% (−5..−16), L-A −26..−29: somma −31..−45. Il resto (~25-35 ns) è esattamente il costo invariante 9-10 ns/op del lifecycle Zval (S-103): nascita/morte/refcount. Nessun treno di peephole lo tocca; lo tocca solo C. Ma C ha una scala incrementale SAFE-only, non è big-bang:
- **C1 borrow-non-clone**: eliminare i clone Rc dei ricevitori sui path IC-hit (H-P1 ne valeva ~3 ns su UN sito; il census dei call-site caldi enumera i siti restanti). Prestito `&` al posto del clone = zero costo, il borrow checker lo garantisce.
- **C2 arena per-richiesta per Zval transitori** dietro indici generazionali (slotmap-style, safe): il refcount sparisce dal path caldo per i valori che non escano dal frame; il drop diventa bulk-free a request_end (compatibile col binding output-capture PRIMA del reset).
- **C3 rappresentazione** (NaN-box): **safe SOLO su indici**, mai su puntatori (il roundtrip puntatore↔f64 esige unsafe ⇒ vietato dal sigillo). C3 resta l'ultima carta, previa decisione utente.

**Mossa concreta S-117 (rotta A, ridimensionata alla piattaforma)**: BOLT **non esiste su Mach-O/AArch64** — la raccomandazione lo cita a vuoto. Pipeline reale: PGO (`-Cprofile-generate/use` con profilo da sei micro + held-out + WP media) + LTO fat + `codegen-units=1` + **ld64 `-order_file`** per il layout deterministico. Primo esito da giudicare: **banda leve-nulle RIMISURATA (N=2) sul binario PGO** — il claim di A è che la banda scende; l'uplift micro è il secondo esito.

## EMENDAMENTI

- **R1 (piattaforma)**: sostituire BOLT con PGO+LTO+order_file come sopra. Misura: banda nulla N=2 sul nuovo binario; meter-riparato se max(banda) ≤ 5 ns/iter (oggi 10).
- **R2 (incommensurabilità)**: A cambia il metro ⇒ **tutte le bande e i binari conservati (052ea417, nulla2) DECADONO**; L-A si rigiudica ricompilando la patch 2c18b2e sotto la nuova pipeline. Vietato trascinare soglie pre-PGO.
- **R3 (anti-tassa nel treno B)**: il treno passa solo se il NETTO per OGNI categoria ≥ −banda(cat) del nuovo metro. Tre campioni L-A (−6,5/−7,0/−6,5) oltre le due nulle (−5,5): le tasse sono reali e SI SOMMANO; un treno di 5 vagoni può regredire calls di 5-7 ns mascherandolo nell'aggregato.
- **R4 (D con traduzione di ownership)**: le tecniche Zend vivono di aliasing che il borrow checker vieta (zval* mutati in place, HashTable auto-puntante). Ogni vagone D dichiara PRIMA la strategia: indice / borrow / RefCell; RefCell sul path caldo = refcount travestito, ammesso solo con A/B che ne dimostri il costo.
- **R5 (C1 nel treno)**: S-118 istruisce C1 col census e mette i siti borrow-non-clone come vagoni del primo treno, accanto a L-A.

## KILL-SWITCH

- **A**: se dopo PGO+order_file la banda nulla non scende ≤ 5 e l'uplift mediano globale < 2% ⇒ A chiusa (si tengono i guadagni gratis, si torna al metro attuale).
- **B**: due treni consecutivi bocciati per accumulo tasse ⇒ regime treno sospeso, si va a C2 diretto.
- **C2**: se il prototipo arena su UNA categoria non rende ≥ banda(prop) ⇒ si ferma prima di toccare la rappresentazione; C3 mai senza mandato utente esplicito.

## APPARATO minimo

Solo lo script di build PGO in `scripts/` (ricetta nel pin, REGOLE §2: il pin PGO nasce collaudato-nell'atto) — mezzo pomeriggio, blocca l'oggetto perché senza di esso nessun A/B S-117 è riproducibile.


===== VERBALE SEDIA: klabnik =====

# Verbale Klabnik — S-116 concilio di rotta (lente: chiarezza, spec dei gate, testabilità)

## VERDETTO: CONCORDO CON EMENDAMENTI

La raccomandazione (A subito / B regime / D metodo / C riserva) è ordinabile ma sotto-specificata in due punti che, dalla mia lente, la invaliderebbero così com'è: (1) **BOLT non esiste su Mach-O/Darwin** — l'ambiente è macOS ARM; la voce va sostituita con strumenti reali (PGO + LTO fat + codegen-units=1 + eventuale `-order_file` di ld64) o la rotta A nasce non testabile; (2) **A vuota TUTTE le bande pre-registrate** (banda micro N=2, banda held-out N=1, famiglia calls −5,50): cambiata la pipeline, i binari conservati (s114-la ecc.) e le nulle misurate non sono più applicabili. «A subito» senza ri-misura delle bande ricrea esattamente il vizio già vietato: gate a soglia fissa su giudice senza banda.

## ROTTA DALLA MIA LENTE (orizzonte 3 sessioni)
Ordine: **A′ (A emendata) → ri-banda → B(treno con vagoni D)**; C riserva con trigger nominato.
- **S-117 (mossa concreta)**: spike A′ = `scripts/pgo-build.sh` (profilo pinnato, `.profdata` hashata e stashata nell'atto); prova di determinismo (build ×2 stessa ricetta ⇒ hash identico, altrimenti la pipeline non può fare da pin); gate pieni sul binario PGO (batteria rc da file, corpus per NOME ×2, parità); micro R=5 per la taglia del guadagno; PRIMA leva-nulla post-A per il primo campione di banda nuova.
- **S-118**: bande complete (≥2 nulle micro, ≥3 campioni held-out) + manifest del treno.
- **S-119**: giudizio del treno.

## EMENDAMENTI
- **R1 (piattaforma)**: la spec di A nomina solo strumenti esistenti su Darwin/aarch64; BOLT espunto. Misura: build ×2 ⇒ hash identico; PGO on/off A/B interleaved sui sei micro.
- **R2 (bande azzerate)**: dopo A, NESSUN gate sul treno finché banda micro (N≥2) e banda held-out (N≥3) non sono ri-misurate sulla pipeline nuova. Misura: verdetti-nulla committati prima del manifest.
- **R3 (circolarità del profilo)**: il workload di profiling è NOMINATO e non coincide coi giudici (preferito: WP+held-out profilano, i micro giudicano); se coincide, la circolarità si dichiara nel verbale. Misura: lista dei file di profilo nel criterio PRE.
- **R4 (spec del treno — il cuore)**: (i) manifest pre-registrato: vagoni per NOME e ordine, cap 5; (ii) admission PER-vagone (parità output, dump, batteria, corpus) PRIMA dell'imbarco; (iii) l'A/B di promozione è UNO SOLO: binario-treno vs pin — mai somma di delta da A/B distinti (già vietato); (iv) guardie con banda da **nulla-treno di taglia comparabile** (byte-delta ±30%): la banda delle nulle piccole non si estende per fede a +15 KB; (v) tie su valori a 2 decimali con le regole S-116(c) (uguale⇒PASS promozione, uguale⇒tiene guardia); rc SOLO da file S-116(d); (vi) **bisezione pre-registrata**: treno bocciato ⇒ si stacca l'ultimo vagone e si rigiudica, max 2 iterazioni, poi verdetto secco. Senza (vi) la domanda «quale vagone incolpo» ricrea S-115.
- **R5 (formula di promozione)**: UNA formula committata prima dello smoke — proposta: promozione se la categoria peggiore migliora ≥ soglia E nessuna categoria peggiora oltre max(2×spread_dep; banda(cat)) E held-out entro banda misurata. La scelta (i)/(ii) del verdetto S-116 (ridurre la tassa calls vs gate a beneficio netto pesato) è **decisione utente pre-registrata**, non deroga in corsa.
- **R6 (leggibilità)**: `treno-manifest.md` ≤20 righe; verdetto `.out` appeso dagli script; scoreboard con voce «vagoni imbarcati: N».

## KILL-SWITCH
- **KS-A**: build non riproducibile ×2 O (guadagno micro globale <2% E banda nulla nuova non più stretta della vecchia, globale ≥10,00) ⇒ A si archivia in 1 sessione, si tiene solo LTO se gratis.
- **KS-B**: treno bocciato 2 volte DOPO bisezione ⇒ B si sospende, si apre C.
- **KS-D**: vagone senza direzione firmata (smoke R=2, segni concordi) non si imbarca — nessun «forse» a bordo.
- **Trigger C**: dopo A′+un treno giudicato, se prop resta >6× ⇒ C diventa cantiere nominato, non riserva.

## APPARATO minimo
Solo `scripts/pgo-build.sh` (profilo+build+hash+stash in UN atto, REGOLE §2) e lo script nulla-treno. Nient'altro: il resto esiste.


===== VERBALE SEDIA: hejlsberg =====

# Verbale Hejlsberg — Concilio S-116/117 (lente: compilatori, interning, codegen)

## VERDETTO: CONCORDO CON EMENDAMENTI

La raccomandazione (A subito / B regime / D metodo / C riserva) è giusta nell'ordine ma sbagliata in DUE punti tecnici che, non emendati, la invalidano.

**Refutazione 1 — BOLT non esiste su questa piattaforma.** Siamo su Darwin/Mach-O (Apple Silicon): BOLT lavora su ELF con profili Linux-perf; non c'è un BOLT maturo per Mach-O. «PGO + BOLT» come scritto è ineseguibile. Il sostituto nativo c'è ed è migliore per il nostro problema: **ld64 `-order_file`** — ordine di funzioni DETERMINISTICO e pinnato. È questo, non il PGO, che «ripara il metro»: il PGO da solo ritira i dadi del layout a ogni build; l'order_file li toglie dal tavolo.

**Refutazione 2 — la riserva su C è aritmeticamente una finzione.** Per ≤3× servono −45% su arith (5,5) e −60% su prop (7,6). A vale 5-15% una-tantum; il treno B decine di ns su ~107-160 ns/iter (L-A ≈ −25% su prop). Composto ottimistico: prop 7,6→~5,1. Il fatto S-103 (9-10 ns/op invarianti = ciclo di vita Zval) dice che il fattore residuo vive in C. Il «solo se >3× dopo A+B» scatterà con quasi-certezza: C non è riserva, è la coda già visibile — l'istruttoria (design su census WP-95/96, perimetro compatibile col verdetto TakeSlot) parte in parallelo, il cantiere si delibera a concilio con le cifre A+B in mano.

## ROTTA DALLA MIA LENTE (3 sessioni)
1. **S-117 = A, in due stadi.** Fatto verificato oggi: il workspace **non ha `[profile.release]`** — build a default (16 CGU, niente LTO fat). Stadio A0: `lto="fat"` + `codegen-units=1` (+ order_file estratto e versionato) — gratis, deterministico, e i 16 CGU sono essi stessi una sorgente della lotteria di layout. Stadio A1: PGO (`-Cprofile-generate` → workload → `llvm-profdata merge` → `-Cprofile-use`), profilo hashato nella ricetta di `pin-phpr.sh`. Poi: **rimisurare la banda con leva nulla sul nuovo assetto** (attesa: ben sotto 10) e ri-giudicare **L-A da sola** su binari RICOSTRUITI (mai A/B cross-pipeline coi candidati vecchi).
2. **S-118: verdetto L-A sul nuovo assetto.** Se la tassa calls persiste a layout deterministico, non era layout: cura = outlining del probe miss (`#[cold]`/`#[inline(never)]`) — coerente con S-104 (run_loop icache-bound). B (treno, max 3 vagoni firmati) solo se le leve singole affogano ANCORA.
3. **S-119: D census-gated** + istruttoria C.

## EMENDAMENTI
- **R1**: sostituire BOLT con order_file ld64; A giudicata pipeline-vs-pipeline stessa sera (micro+held-out+WP) + banda nulla rimisurata.
- **R2**: profilo PGO addestrato su WP request-loop + corpus misto, **mai sui soli sei micro** (overfit del giudice); profilo e order_file versionati; determinismo provato: due build stessa ricetta → hash .text identico.
- **R3**: tutte le bande/soglie pre-A decadono sul nuovo assetto; primo atto post-A = leva nulla.
- **R4**: per D, **veto sul threaded dispatch**: Rust stabile non garantisce tail-call (già famiglia-refutato S-111). Ordine d'istruttoria: interned strings SOLO se il census conta hash/memcmp residui sul path caldo oltre le IC già esistenti; HashTable packed per arr; specializzazione handler solo census-giustificata (tetto icache S-104).
- **R5**: C promossa da riserva a istruttoria parallela (≤20% finestra), decisione di cantiere al prossimo concilio.

## KILL-SWITCH
- A0/A1: banda nulla non ridotta E guadagno mediano sei-micro <3% ⇒ tenere solo ciò che dimezza la banda, abbandonare il resto.
- Riproducibilità rotta (due build stessa ricetta ≠ identiche) ⇒ solo order_file, niente PGO.
- B: somma del treno sotto la somma delle soglie ⇒ smontare.
- D: census interning sotto soglia pre-registrata ⇒ non portare.

## APPARATO minimo
`scripts/build-pgo.sh` (ricetta unica: profilo→merge→use→order_file→hash), integrato in `pin-phpr.sh` — senza, il pin non è «collaudo-nell'atto».


===== VERBALE SEDIA: bak =====

# Verbale sedia Bak — S-116/117, lente: VM di produzione (dispatch, IC, layout, path caldi)

## VERDETTO: CONCORDO CON EMENDAMENTI

## ROTTA DALLA MIA LENTE (orizzonte 3 sessioni)
Ordine: **A (ridimensionata a PGO+LTO) → B (regime) → C promossa da riserva a cantiere istruito → D come alimentatore di B e C**.

Il fatto che governa tutto è il costo/op ~9-10 ns INVARIANTE tra categorie: è la firma di una tassa fissa per opcode (dispatch + ciclo di vita del valore), non di operazioni lente. Una VM di produzione non risponde a questa firma con peephole da 3-30 ns: risponde cambiando la rappresentazione dei valori. Zend stessa NON refcounta gli scalari (IS_LONG/IS_DOUBLE viaggiano senza rc); se phpr paga Rc/clone su path scalari, lì sta il 4-7× delle micro contro l'1,87× del full. L-A (fusione trigramma) è esattamente una superinstruction: giusta, ma è la classe di leva che rende 20-30 ns, non 60.

**Aritmetica che smonta "C solo se serve"**: prop lato pin ~107 ns/iter, rapporto 7,6× ⇒ per ≤3× servono ~−65 ns/iter. A al meglio 10-15% (~10-16 ns) + L-A (~27) + 2-3 vagoni (~10-20) ≈ 50-60: al limite teorico, sotto nel realistico. A+B non chiudono prop né arith. C non è riserva: è la destinazione. Va istruita ORA (censimento, non codice).

**Mossa concreta S-117**: build PGO (cargo -Cprofile-generate/use, workload pre-registrato = sei micro + held-out + slice WP; cgu=1 + fat LTO se non già attivi) → gate parità PIENO sul binario PGO (batteria, corpus per NOME ×2) → micro R=5 → **rimisura della banda leva-nulla (patch s115-zavorra2 riapplicata) sul binario PGO**. Il claim "A ripara il metro" si giudica lì, non si racconta.

## EMENDAMENTI
- **R1 — BOLT non esiste sulla nostra piattaforma**: llvm-bolt è ELF/x86-centrico; su Mach-O ARM64 il supporto è assente/sperimentale. La rotta A è PGO+LTO+cgu, non "PGO+BOLT". Misura: verifica tooling in pre-flight; atteso dichiarato 5-10%, non 5-15%.
- **R2 — "ripara il metro" è ipotesi, con rischio inverso**: PGO rende il layout funzione del PROFILO ⇒ ogni leva futura cambia profilo ⇒ possibile instabilità NUOVA. Giudice: banda nulla su binario PGO (N≥2); il claim passa solo se banda_globale scende materialmente (es. ≤5 vs 10). Se non scende, A resta guadagno una-tantum e B diventa obbligatorio.
- **R3 — profilo = artefatto pinnato**: workload PGO pre-registrato, profilo versionato, rigenerazione SOLO via scripts/pin-phpr.sh emendato; altrimenti ogni build è "emendata" con metro diverso.
- **R4 — treno B**: somma-bersaglio pre-registrata ≥ 2× banda globale vigente (oggi ≥20 ns; meno se PGO riduce la banda); ogni vagone conserva ammissione+parità+direzione individuali; le guardie non-bersaglio si giudicano SUL TRENO INTERO (la tassa layout per-vagone non è additiva — i 5 campioni calls lo mostrano).
- **R5 — C fuori dalla riserva**: istruttoria entro S-119, timebox ½ sessione: census del traffico refcount per iterazione (inc/dec per op, per categoria) phpr vs Zend. Decide la variante (scalari non contati / deferred RC / arena) coi numeri, prima di aprire il cantiere.
- **R6 — D dichiara le dipendenze**: le tecniche Zend (run-time cache/IC, interned strings) presuppongono la SUA rappresentazione valori; ogni porting D dichiara se dipende dalla repr — se sì, è un vagone di C, non di B.

## KILL-SWITCH pre-registrati
- **KS-A1**: PGO < +3% mediano sulle sei micro E banda nulla invariata ⇒ A chiusa in 1 sessione, treno B su banda attuale.
- **KS-A2**: gate parità fallisce su binario PGO e il fix costa > ½ sessione ⇒ PGO rinviata, non inseguita.
- **KS-B1**: treno da 3 vagoni sotto la somma-bersaglio ⇒ treno sciolto, vagoni conservati, C rotta primaria.
- **KS-C1**: se l'istruttoria mostra RC-traffic/op ≈ Zend (collo NON è refcount) ⇒ la variante C cambia bersaglio (dispatch/IC), niente cantiere cieco.

## APPARATO minimo
Solo l'emendamento allo script pin per la ricetta PGO (R3) e il contatore RC-traffic per R5 (strumentazione a compile-flag, MAI nel binario di misura).


===== VERBALE SEDIA: pedersen =====

# Verbale Pedersen — Concilio S-116/117 (lente: confine per-richiesta, lifecycle, parità per-request)

## VERDETTO: CONCORDO CON EMENDAMENTI

A subito, B regime, D metodo: sì, ma la raccomandazione com'è scritta contiene tre punti che dalla mia lente non reggono. E la clausola su C («solo se dopo A+B resta >3×») è mal posta: C non è UNA rotta, è due — una compatibile coi vincoli, una no.

## ROTTA DALLA MIA LENTE (3 sessioni)

Ordine: **A' (rescopata) → B (con ammissione per-vagone) → D (con classe-lifecycle dichiarata per tecnica) → C-lite come vagone di D, C-piena VIETATA nella forma arena/NaN-box**.

**Mossa S-117**: spike pipeline — LTO fat + codegen-units=1 + PGO cargo, con profilo che INCLUDE il teardown (request_end, distruttori, sweep RetainSet), non solo i loop micro; poi **rimisurare la banda leve-nulle (×2) sulla pipeline nuova PRIMA di qualunque verdetto di leva**. Gate pieni: batteria, corpus 1415 per NOME ×2, fixture, e tripla census per-request (obj/req deve restare 0,000 — la parità di WP-72 non è negoziabile).

## EMENDAMENTI

**R1 — A: BOLT probabilmente non esiste qui.** Siamo su Darwin ARM64/Mach-O; BOLT è di fatto ELF/Linux. «Layout deterministico» su macOS = order-file di ld64 o niente. MISURA: verifica di piattaforma in apertura S-117; se il tool non c'è, A si dichiara PGO+LTO-only, senza vantarsi del metro.

**R2 — A: «ripara il metro» è un'ipotesi, non un fatto.** PGO rende il layout FUNZIONE del profilo: ogni modifica di codice può rimescolare inlining/impaginazione anche PIÙ di oggi. MISURA: banda leve-nulle rimisurata sulla pipeline nuova; A «ripara il metro» solo se banda_nuova < 10 ns attuali, altrimenti A vale solo per il guadagno assoluto.

**R3 — A: il profilo deve pesare il confine.** Un profilo raccolto solo sui sei micro declassa a freddo request_end/distruttori/output-capture: il full WP (1,867×) e la parità dei giudici held-out possono peggiorare mentre i micro migliorano. MISURA: WP full ON e tripla census nel gate di promozione della pipeline.

**R4 — B: la somma giudica SOLO la performance.** Fedeltà e ammissione restano PER-VAGONE (parità output, dump, batteria, divergenze a catalogo per NOME): un treno che compensa una regressione di un vagone col guadagno di un altro è esattamente il buco che il fail-set congelato esiste per chiudere. Treno bocciato ⇒ revert AL BYTE dell'intero treno.

**R5 — C: spacchettare.** (i) NaN-boxing = bit-play che in Rust vive di transmute: collide col sigillo SAFE-only/VmGate — non riproporre senza dimostrazione safe. (ii) Arena per-richiesta con bulk-free: i distruttori PHP girano in ordine definito, eseguono codice arbitrario e possono risuscitare valori; il RetainSet persistente e ogni valore che sopravvive alla richiesta esigono promozione/copy-out; il bulk-free è lecito solo per valori trivially-drop. E l'ORDINE è mandato permanente: capture output → distruttori (FIFO gc_queue+gc_birth) → reset. (iii) **C-lite** = elisione di refcount sul path caldo con prova di lifetime safe: questa è compatibile e attacca proprio il collo Zval da 9-10 ns/op. MISURA d'ingresso per ogni pezzo di C: tripla census 0,000 + batteria ordering 8/8 byte-id + fail-set per NOME.

**R6 — D: ogni tecnica portata dichiara la sua classe di lifecycle PRIMA del codice.** Interned strings Zend sono per-processo: un intern che cattura stringhe request-local è un leak/una violazione di confine (la lezione RetainSet). Inline cache su class-pointer va invalidata se la tabella classi resetta per-richiesta. Checklist di confine nel criterio PRE di ogni vagone D.

## KILL-SWITCH
- KS-A1: piattaforma senza layout-tool E banda_nuova ≥ banda attuale ⇒ A decade a LTO-only in 1 sessione.
- KS-A2: PGO cambia il fail-set per NOME o la batteria ⇒ abort A, revert pipeline.
- KS-A3: 2 sessioni di A senza Δ spedito ⇒ si torna a B sulla pipeline corrente.
- KS-B1: vagone che fallisce ammissione ⇒ fuori dal treno, treno rigiudicato da capo.
- KS-C1: C-lite inesprimibile safe ⇒ morta, nel registro «NON riproporre».

## NON TOCCARE
request_end e il suo ordine; output-capture prima del reset; pinning per-richiesta del RetainSet; free-order FIFO dei distruttori. Qualunque rotta che li sfiora paga i gate di R5 per intero.


===== VERBALE SEDIA: leijen =====

# Verbale Leijen (lente: allocatore mimalloc, footprint fisico) — S-116, concilio di rotta

## VERDETTO: CONCORDO CON EMENDAMENTI

A subito sì, B come regime sì, D come selezione sì. MI OPPONGO al solo punto «C riserva»: dalla mia lente C non è una riserva, è aritmeticamente l'unica rotta che chiude, e la sua istruttoria deve partire subito.

## ROTTA DALLA MIA LENTE (3 sessioni)

**A′ (S-117) → B+D (S-118) → istruttoria C in parallelo da S-118.**

L'aritmetica: prop è ~107 ns/iter lato pin contro ~14 dell'oracle; ≤3× vuol dire ~42. L-A vale ~27; A, se rende il 5-15%, altri ~8-11. Restano ~70: ancora ~5×. Il fatto S-103 (costo/op 9-10 ns INVARIANTE tra categorie) dice che il pavimento è il ciclo di vita Zval — nascita/morte/refcount — cioè territorio dell'allocatore, non dell'impaginazione. PGO non toglie un solo incremento di refcount né una alloc: A ripara il METRO (ed è per questo che va fatta subito), non il collo. Chi vende A come rotta di chiusura si illude.

**Mossa concreta S-117**: pipeline PGO rustc (`-Cprofile-generate` sulle sei micro + media WP, merge, `-Cprofile-use` + LTO fat + codegen-units=1), profdata CONGELATO come artefatto versionato; poi RI-MISURARE la banda leve-nulle (nulla-1/nulla-2 ricostruite sotto la nuova pipeline) PRIMA di qualunque verdetto su leve. La banda N=2 attuale muore con la pipeline nuova: non si eredita.

## EMENDAMENTI

**R1 — BOLT non esiste su questa piattaforma.** BOLT è ELF/Linux-centrico; su macOS arm64 Mach-O il layout deterministico si fa con PGO rustc + `ld64 -order_file` (o linker order equivalente). Pre-registrare la toolchain REALE prima di promettere «BOLT»; apparato timebox ½ sessione (REGOLE §1). Misura: la pipeline esiste se produce due build byte-stabili a sorgente invariato.

**R2 — Profilo congelato o A peggiora il metro.** PGO rende il layout FUNZIONE del profilo: se il profdata cambia tra i bracci, ogni A/B confronta impaginazioni diverse e la banda esplode. Regola: stesso profdata pinnato per ENTRAMBI i bracci di ogni A/B; ri-profilare solo a promozione avvenuta, con ri-misura banda. Misura di successo di A come riparazione del metro: banda nulla N=2 sotto PGO+order_file ≤ 5 ns/iter (oggi 10). Sotto quella soglia le leve da 3-30 ns tornano giudicabili.

**R3 — C non è riserva: istruttoria da S-118.** Il censimento alloc/op e refcount-op/op per categoria, su ENTRAMBI i motori (apparato free-hist H-D già esistente, S-103: 1 alloc×32 B/chiamata), si fa dentro la finestra leva di S-118. Decide QUALE variante C: se il pavimento è refcount/drop e non malloc, la variante giusta ELIMINA allocazioni (scalari inline/NaN-box, niente Rc sui scalari), non le sposta.

**R4 — Arena per-richiesta SOPRA mimalloc: presunzione di colpevolezza.** WP-59 ha misurato frag mimalloc 2% al picco: mimalloc non spreca e il suo fast path è già a pochi ns. Un secondo livello (arena) = doppia contabilità + high-water ritenuto fino a request_end ⇒ rischio diretto sul vincolo peak WP 1842 MiB. Gate pre-registrato per OGNI variante C: peak WP ≤ 1842 MiB +2%, pena bocciatura della variante (non della rotta).

**R5 — B pretende la banda dei TRENI.** Le tasse sistematiche si sommano come i guadagni: tre vagoni da −1..−1,5 su calls fanno −4 veri. Prima di giudicare un treno: treno-NULLO (3-5 commit vuoti) per la banda multi-commit sotto la pipeline nuova.

## KILL-SWITCH

- **A**: se dopo PGO+order_file la banda nulla N=2 resta >5 ns O il geomean micro non migliora ≥3% ⇒ A declassata a solo-layout-freeze, si passa a B/D.
- **A-apparato**: toolchain non in piedi in ½ sessione ⇒ ripiego a PGO-solo.
- **C-arena**: peak WP > 1842+2% in qualunque variante arena ⇒ variante morta.
- **C-tutta**: se il censimento R3 mostra <1 alloc/op e refcount <30% del budget ciclo sulle categorie calde ⇒ l'ipotesi Zval-lifecycle è refutata, C si ridisegna prima di spendere sessioni.

## APPARATO minimo
Solo R1 (pipeline PGO/order_file) e ri-misura bande: tutto il resto usa strumenti già esistenti (free-hist, leve nulle, vmmap physical footprint).


===== VERBALE SEDIA: stogov =====

# Verbale sedia Stogov — S-116 → S-117 (lente: Zend engine/opcache)

## VERDETTO: CONCORDO CON EMENDAMENTI

La raccomandazione è giusta nell'ordine operativo, sbagliata nella tassonomia:
tratta C e D come rotte separate e relega C a riserva. Dalla mia lente è
insostenibile: il fatto S-103 (costo/op ~9-10 ns INVARIANTE tra categorie) è la
firma del ciclo di vita dei valori, non delle operazioni. Zend arriva a ~2-3
ns/op non con handler più furbi ma perché la maggioranza degli zval NON ha
ciclo di vita: scalari per valore in 16 byte senza allocazione né refcount;
stringhe interned refcount-free (nomi di proprietà, literal); COW sui soli
contenitori. La lezione S-113 («1 clone Rc/op ≈ 3 ns») lo conferma: 3-4
Rc-op/iterazione SONO il pavimento. Aritmetica: prop pin ~107 ns/iter, oracle
~14; per il 3× servono −65 ns. A (5-15%) + treno di leve da 3-30 ns non li
trova: solo togliere il refcount dal path caldo li trova. Quindi **D fatto
bene È C a rate** — e comincia in S-118, non «se dopo A+B resta >3×».

## ROTTA DALLA MIA LENTE (3 sessioni)

1. **S-117 = A**, per il METRO prima che per i ns: pipeline PGO+LTO fat+
   codegen-units=1 + **order-file ld64** — ⚠️ REFUTAZIONE PUNTUALE: **BOLT non
   esiste su Mach-O** (solo ELF); su darwin il layout deterministico si fa con
   `-order_file`/`-Wl,-order_file`, non con BOLT. Chi scrive «BOLT» nel piano
   S-117 sta pianificando su Linux.
2. **S-118 = D ordinato dal ciclo di vita**: census rc-op/alloc per iterazione
   e per categoria su ENTRAMBI i motori (non frequenze opcode). Ordine
   previsto da Zend: (1) scalari per valore rc-free, (2) stringhe
   interned/literal rc-free, (3) COW contenitori, (4) inline cache famiglia
   L-A, (5) specializzazione handler **ULTIMA** — S-103 la refuta come
   priorità: attacca il costo per-op che non è il collo.
3. **B = regime di promozione** del treno (L-A primo vagone: magnitudine
   +27-29 stabilita ×3; la tassa calls ~1-1,5 ns è esattamente ciò che un
   giudizio a somma assorbe). Parità output e admission restano PER VAGONE.

## EMENDAMENTI

- **R1 — profilo PGO mai sui giudici**: corpus di profiling = WP + held-out,
  MAI le sei micro (teaching-to-the-test). Misura: micro giudicate post-hoc
  col criterio solito.
- **R2 — le bande DECADONO con la pipeline**: dopo A, banda micro 
  (0,40…10,00) e held-out sono VOID. Prima di ogni verdetto: ≥2 leve nulle
  sulla pipeline nuova, banda ri-pre-registrata. Misura: file banda v2
  committato PRIMA del primo A/B.
- **R3 — D si seleziona con contatori di vita, non di dispatch**: harness che
  conta nascite/morti/rc-op per iter su phpr e (via Vexp/DTrace) su Zend.
  Misura: tabella per categoria, delta rc-op ↔ delta ns previsto.
- **R4 — trappole semantiche pre-registrate per ogni rata di C/D**: COW ×
  references (is_ref sospende la separazione); ordine di distruzione
  OSSERVABILE (lezione sweep EAGER); interned ≠ per-richiesta (mai liberate
  nel reset, mandato output-capture intatto); IS_UNDEF ≠ NULL. Gate: fail-set
  1415 per NOME ×2 + batteria, come sempre. NIENTE NaN-boxing: Zend non lo
  usa, perde i tipi-sentinella e complica il flag rc — la rata giusta è
  «tagged value 16B + rc solo sui tipi contati».

## KILL-SWITCH

- **A**: se la banda ri-misurata post-pipeline > 10,00 globale (metro
  peggiorato) o WP < −1% oltre spread A-A′ → revert pipeline, si tiene solo
  ciò che non degrada il metro.
- **B**: se la somma del treno sui giudici < ½ della somma delle magnitudini
  firmate → i vagoni si annichilano (layout): treno fermo, si torna a R2.
- **D/C a rate**: ogni rata che muove il fail-set per NOME o rompe la parità
  output si riverte al byte in sessione; due rate consecutive revertate →
  concilio.

## APPARATO minimo
Solo R3 (contatori rc-op): blocca la selezione dei vagoni; timebox ½ sessione.


===== VERBALE SEDIA: gregg =====

# Verbale sedia Gregg — lente: metodologia di misura e attribuzione (S-116, concilio di rotta)

## COSA SAPPIAMO OGGI DI PHPR CHE PRIMA NON SAPEVAMO (S-113..116, fatti secchi)
1. **La banda-layout esiste ed è misurata**: leve NULLE spostano le micro fino a 10 ns/iter per categoria (re 0→10 tra N=1 e N=2); una nulla fa 5/5 segni concordi su 3 categorie. Una banda a N=1 mente.
2. **Le leve singole (3-30 ns) affogano nel layout**: H-P1 (+3,33) è indistinguibile dal nulla; solo L-A (+26/29/30, tre campioni, spread_A depurato 2-4) emerge.
3. **La tassa calls è SISTEMATICA, non banda**: L-A −6,50/−7,00/−6,50 tutti oltre le due nulle −5,50 identiche → ~1-1,5 ns/iter reali su un sentiero non toccato nei dump. L'attribuzione «layout/icache del probe» è ipotesi nominata, non firmata.
4. **Il gate held-out a soglia fissa è REFUTATO come diagnostico**: la nulla-2 lo sfonderebbe (9,80>9,71) a semantica zero.
5. **Costo/op ~9-10 ns quasi invariante tra categorie (S-103)**: il collo è il ciclo di vita degli Zval, non i singoli opcode.
6. **Le famiglie 1,3×min con esclusione per NOME recuperano il metro senza toccare la leva** (spread 47→2); i binari CONSERVATI (zero rebuild) rendono le bande riusabili.

## VERDETTO: CONCORDO CON EMENDAMENTI
(su: A subito / B regime / D metodo / C riserva)

## ROTTA DALLA MIA LENTE (orizzonte 3 sessioni)
Ordine: **A (S-117) → B con gate netto pesato (S-118) → istruttoria C in parallelo (da S-118/119) → D come cava di vagoni**. Due refutazioni alla raccomandazione:
- **A non «ripara» il metro per fede**: PGO cambia il layout in funzione del profilo — se il profilo non è versionato e deterministico, A AGGIUNGE una sorgente di rumore. E **BOLT non supporta Mach-O** (target ELF): su Darwin la via è PGO (llvm-profdata) + **ld64 `-order_file`** per l'ordinamento deterministico delle funzioni. A va trattata come ipotesi con banda pre/post, non come cura.
- **C non è «riserva»: l'aritmetica è già computabile OGGI**. Prop pin ~107 ns/iter vs obiettivo 3× ≈ 42: servono ~65 ns; la migliore leva mai vista ne vale 29; A promette 5-15%. Fatto 5 (invarianza) dice che il pavimento è lo Zval. Aspettare 2-3 sessioni per «scoprire» che A+B non bastano è spreco pre-registrabile: l'istruttoria C (design + strumenti, niente codice sul path caldo) parte comunque.

**Mossa concreta S-117**: (i) feasibility spike PGO+order_file (mezza sessione, timebox REGOLE §1): build passa batteria 1742/0 + corpus 1415 per NOME ×2 + parità output; (ii) su pipeline nuova: micro R=5 + **2 leve nulle** (patch s114/s115 riusate) → banda_new; (iii) test mirato: L-A ricompilata sotto PGO — se la tassa calls rientra nella banda nulla nuova, l'ipotesi «probe/layout» è firmata e il metro è riparato nei fatti.

## EMENDAMENTI
- **R1 (feasibility)**: niente BOLT su Darwin; A = PGO + order_file. Misura: build riproducibile (2 build stesso sorgente → hash identico = definizione operativa di «layout deterministico»).
- **R2 (doppio criterio per A, pre-registrato)**: A promossa come RIPARA-METRO solo se banda_new ≤ ½ banda_old (globale 10→≤5) su ≥2 nulle; altrimenti promuovibile come sola leva velocità (Δ micro/WP sopra banda_new). Mai fondere i due claim.
- **R3 (reset dichiarato)**: A invalida TUTTE le bande e baseline: nuovo pin, micro R=5, held-out e bande rifatte sulla pipeline nuova. Riusare bande vecchie su binari PGO = vietato.
- **R4 (gate del treno B)**: giudizio sul NETTO pesato tra categorie con regressioni cap = banda_new; vagoni ammessi solo con direzione firmata su binari conservati; protocollo di espulsione leave-one-out pre-registrato, MAX 1 giro, altrimenti il treno cade intero.
- **R5 (tassa calls)**: primo lavoro-vagone = collocazione cold/outlining del probe con dente disasm bl-count (già nominato nel verdetto S-116) — oppure assolto gratis da A (test R5 = punto iii della mossa).
- **R6 (C non condizionale)**: pre-registrare la proiezione aritmetica; design doc C entro S-119 comunque.

## KILL-SWITCH (pre-registrati)
- **KS1**: build PGO fallisce parità (batteria/corpus per NOME/output) → A abbandonata in sessione, pipeline revertata.
- **KS2**: banda_new ≥ banda_old E Δ velocità sotto banda → A revertata intera; non tenere una pipeline che complica il build senza pagare.
- **KS3**: treno B fallisce il netto dopo 1 giro di espulsione → smontato, vagoni in coda singola; niente ricomposizione sugli stessi binari.
- **KS4**: 2 sessioni di solo apparato-A senza un A/B misurato → anomalia dichiarata, stop rotta.

## APPARATO minimo (blocca l'oggetto)
Script pipeline in `scripts/` (variante pin-phpr.sh con passo profilo: input = 6 micro + WP script, profdata hashato e committato); kit leva-nulla riusabile (patch zavorra versionate: apply/misura/revert in un atto); rc dei gate da FILE, mai da pipe.


===== NOTA TEAM: misura =====

# Verbale team-misura (relatore) — sedie: Gregg, Bak, Leijen

## CONVERGENZE
1. Verdetto unanime: CONCORDO CON EMENDAMENTI; ordine A → B (regime) con D come cava/alimentatore.
2. **BOLT non esiste su Mach-O ARM64**: rotta A reale = PGO rustc (`-Cprofile-generate/use`) + LTO fat + cgu=1 + `ld64 -order_file`. Verifica tooling in pre-flight.
3. **Profilo = artefatto pinnato**: workload pre-registrato (sei micro + WP), profdata versionato/congelato, stesso profdata per ENTRAMBI i bracci di ogni A/B; rigenerazione solo via scripts/pin-*.sh emendato.
4. **«A ripara il metro» è ipotesi, non cura**: si giudica ri-misurando la banda leva-nulla (N≥2, patch s114/s115 riusate) sul binario PGO. Reset dichiarato: A invalida TUTTE le bande/baseline; nuovo pin, nulla si eredita.
5. Gate parità PIENO sul binario PGO (batteria, corpus per NOME ×2, output) prima di qualunque micro.
6. **C non è riserva**: l'aritmetica (~107 ns/iter prop vs ~42 target; L-A ~27 + A 5-15% non chiudono) è pre-registrabile oggi; istruttoria C (censimento alloc/op e RC/op per categoria, entrambi i motori, senza codice sul path caldo) entro S-118/119.
7. Treno B: le tasse sistematiche si sommano come i guadagni; giudizio sul treno INTERO, vagoni con ammissione/parità/direzione individuali su binari conservati.

## CONFLITTI
- **Resa attesa A**: Bak impone 5-10% dichiarato; Gregg e Leijen citano 5-15%.
- **Logica kill A**: Bak chiude con congiunzione (PGO<+3% mediano **E** banda invariata); Leijen declassa con disgiunzione (banda>5 **O** geomean<3%). Non riconciliato.
- **Cosa può diventare A se la banda non scende**: Gregg «sola leva velocità»; Bak «guadagno una-tantum, B obbligatorio»; Leijen «solo-layout-freeze».
- **Giudice del treno B**: Gregg netto pesato + espulsione leave-one-out max 1 giro; Bak somma-bersaglio ≥2× banda vigente; Leijen pretende un treno-NULLO (3-5 commit vuoti) preliminare.
- **Variante C**: Leijen presume colpevole l'arena sopra mimalloc (gate peak ≤1842 MiB+2%) e privilegia l'eliminazione delle alloc (scalari inline/NaN-box); Bak decide coi numeri tra scalari-non-contati/deferred-RC/arena; Gregg si limita al design doc entro S-119.
- **Tassa calls**: solo Gregg la eleva a primo vagone (cold/outlining del probe, dente disasm bl-count) se non assolta gratis da A.

## PRIORITÀ PER L'ORDINE S-117
1. **Spike PGO+LTO+order_file** (timebox ½ sessione). Misura: due build a sorgente invariato con hash identico + gate parità pieno (batteria 1742/0, corpus per NOME ×2, output).
2. **Ri-misura banda nulla sul binario PGO**: micro R=5 + ≥2 nulle. Misura: banda_new ≤5 ns/iter (vs 10) = claim ripara-metro; altrimenti solo Δ velocità sopra banda_new.
3. **L-A ricompilata sotto PGO (tassa calls)**. Misura: Δ calls dentro banda nulla nuova ⇒ ipotesi probe/layout firmata; dente disasm bl-count.

## KILL-SWITCH consolidati
- Parità fallita su binario PGO e fix >½ sessione ⇒ A revertata/rinviata in sessione (KS1/KS-A2).
- Banda invariata e velocità sotto soglia ⇒ A cade o declassa (KS2/KS-A1/Leijen-A; **divergenza E/O registrata**).
- Toolchain non in piedi in ½ sessione ⇒ ripiego PGO-solo; 2 sessioni solo-apparato senza A/B ⇒ anomalia, stop rotta (KS4).
- Treno B sotto bersaglio dopo 1 giro espulsione ⇒ sciolto, vagoni in coda singola, C primaria (KS3/KS-B1).
- Censimento C: <1 alloc/op e RC<30% budget, o RC-traffic ≈ Zend ⇒ C si ridisegna (KS-C1/C-tutta); arena con peak WP >1842+2% ⇒ variante morta.


===== NOTA TEAM: engine =====

# Verbale TEAM-ENGINE — Concilio S-116/117 (sedie: Stogov · Hejlsberg · Pedersen)

Verdetto unanime delle sedie: CONCORDO CON EMENDAMENTI (3/3).

## CONVERGENZE
1. **BOLT refutato 3/3**: su Darwin/Mach-O non esiste; il layout deterministico si fa con ld64 `-order_file`. Chi scrive «BOLT» pianifica su Linux.
2. **S-117 = A (pipeline)**: LTO fat + codegen-units=1 + order_file (il workspace oggi non ha `[profile.release]`: build a default, 16 CGU), PGO a stadio successivo.
3. **Le bande DECADONO con la pipeline**: prima di qualunque verdetto di leva, ≥2 leve nulle sul nuovo assetto e banda v2 pre-registrata/committata. «Ripara il metro» è ipotesi, non fatto: vale solo se banda_nuova < 10 ns.
4. **Profilo PGO mai addestrato sui giudici** (le sei micro): corpus = WP request-loop + held-out/misto; profilo e order_file versionati; determinismo provato (due build stessa ricetta → hash .text identico).
5. **Fedeltà e ammissione restano PER-VAGONE**: parità output, fail-set 1415 per NOME ×2, batteria, tripla census (obj/req = 0,000); la somma del treno B giudica SOLO la performance.
6. **C non è una vera riserva**: l'aritmetica (S-103, 9-10 ns/op invarianti = ciclo di vita Zval; per ≤3× servono −45/−60%) dice che il fattore residuo vive lì. NaN-boxing escluso 2/2 (Stogov: Zend non lo usa; Pedersen: transmute vs sigillo SAFE-only).
7. **D si seleziona con contatori di vita** (census rc-op/alloc per iterazione, entrambi i motori), non con frequenze opcode; specializzazione handler ULTIMA (tetto icache S-104); veto threaded dispatch (S-111).

## CONFLITTI (non appianati)
- **Cosa fa S-118** — Stogov: D ordinato dal ciclo di vita, subito («D fatto bene È C a rate»). Hejlsberg: verdetto L-A da sola su binari ricostruiti; D slitta a S-119 census-gated. Pedersen: B con ammissione per-vagone prima di D.
- **Statuto di C** — Stogov: C comincia in S-118 come rate di D (tagged value 16B + rc solo sui tipi contati). Hejlsberg: istruttoria parallela ≤20% finestra, cantiere deliberato al prossimo concilio con cifre A+B. Pedersen: spacchettare — solo C-lite (elisione refcount con prova lifetime safe) come vagone di D; arena/NaN-box VIETATA nella forma piena.
- **Corpus del profilo** — Pedersen esige che INCLUDA il teardown (request_end, distruttori, sweep RetainSet), oltre al WP+held-out degli altri due.

## PRIORITÀ PER L'ORDINE S-117 (max 3)
1. **Pipeline A a stadi**: A0 = `lto="fat"` + `codegen-units=1` + order_file estratto e versionato; A1 = PGO (`scripts/build-pgo.sh`, integrato in `pin-phpr.sh`). MISURA: giudizio pipeline-vs-pipeline stessa sera (sei micro + held-out + WP full ON + tripla census 0,000); hash .text identico su due build.
2. **Ri-pre-registrazione bande**: ≥2 leve nulle sulla pipeline nuova; file banda v2 committato PRIMA del primo A/B. MISURA: banda_nuova < 10 ns, altrimenti A vale solo per il guadagno assoluto.
3. **Harness contatori di vita (R3 Stogov, timebox ½ sessione)**: nascite/morti/rc-op per iter e per categoria su phpr e Zend (Vexp/DTrace). MISURA: tabella per categoria, delta rc-op ↔ delta ns previsto — ordina i vagoni D/C-lite.

## KILL-SWITCH CONSOLIDATI
- **A**: PGO cambia fail-set per NOME o batteria ⇒ abort, revert pipeline. Banda non ridotta E mediana sei-micro <3% ⇒ tenere solo ciò che dimezza la banda. Riproducibilità rotta ⇒ solo order_file, niente PGO. 2 sessioni senza Δ spedito ⇒ tornare a B sulla pipeline corrente. WP < −1% oltre spread A-A′ ⇒ revert.
- **B**: somma treno < ½ somma magnitudini firmate ⇒ treno fermo, smontare; vagone che fallisce ammissione ⇒ fuori, treno rigiudicato; treno bocciato ⇒ revert AL BYTE dell'intero treno.
- **D/C**: rata che muove il fail-set per NOME o rompe la parità ⇒ revert al byte in sessione; due rate consecutive revertate ⇒ concilio; census interning sotto soglia pre-registrata ⇒ non portare; C-lite inesprimibile safe ⇒ morta, registro «NON riproporre».

## NON TOCCARE (mandato semantico, Pedersen)
request_end e il suo ordine; output-capture PRIMA del reset; pinning per-richiesta del RetainSet; free-order FIFO dei distruttori; interned mai liberate nel reset. Ogni tecnica D dichiara la sua classe di lifecycle PRIMA del codice.


===== NOTA TEAM: struttura =====

# Verbale TEAM-STRUTTURA — Concilio S-116→S-117 (Hoare · Matsakis · Klabnik)

Verdetti sedie: 3× CONCORDO CON EMENDAMENTI.

## CONVERGENZE
1. **BOLT espunto (unanimità)**: non esiste su Mach-O/aarch64. A′ = PGO rustc (`-Cprofile-generate/use`) + LTO fat + `codegen-units=1` + `-order_file` ld64. Il criterio PRE nomina solo strumenti eseguiti con successo su questo host.
2. **A′ vuota TUTTE le bande (unanimità)**: banda micro N=2, held-out, layout e binari conservati (052ea417, nulla2) DECADONO; nessun gate finché le bande non sono ri-misurate sul binario A′ (≥2 leve nulle). L-A si rigiudica ricompilando 2c18b2e sotto la pipeline nuova.
3. **Treno B**: manifest pre-registrato per NOME (cap 5); fedeltà/admission PER-VAGONE (parità, batteria, corpus per NOME), cronometro PER-TRENO con UN solo A/B binario-treno vs pin; anti-tassa: netto per OGNI categoria ≥ −banda(cat) del metro nuovo; guardie da nulla-treno di taglia comparabile (byte-delta ±30%); tie a 2 decimali regole S-116(c); rc SOLO da file; bisezione pre-registrata (stacca ultimo vagone, max 2 iterazioni).
4. **C safe-only**: NaN-boxing su PUNTATORI vietato (unsafe per costruzione, rompe VmGate); variante ammessa = arena per-richiesta + indici generazionali + elisione refcount (use-after-free → use-after-recycle logico); compatibile col binding output-capture.
5. **Apparato minimo**: solo script build PGO in UN atto (profilo pinnato, `.profdata` hashata/stashata, REGOLE §2) + prova determinismo build ×2 ⇒ hash identico.
6. **D**: ogni vagone dichiara PRIMA la strategia di ownership (indice/borrow/RefCell; RefCell su path caldo solo con A/B del costo) e serve direzione firmata (smoke R=2 segni concordi) per imbarcarsi.

## CONFLITTI (registrati, non appianati)
- **Posizione di C** — Hoare: riserva, KS-C solo se dopo 3 sessioni A′+B il peggiore resta >3×. Klabnik: riserva ma trigger anticipato (dopo A′+UN treno, prop >6× ⇒ cantiere nominato). Matsakis: «C in riserva» è REFUTATO dall'aritmetica (−65 ns/iter necessari; A+L-A rendono −31..−45; il resto è lifecycle Zval che solo C tocca): C1 borrow-non-clone entra come vagone già in S-118, C2 arena subito dopo.
- **Workload di profiling** — Hoare e Matsakis includono i sei micro nel profilo; Klabnik R3: il profilo NON deve coincidere coi giudici (WP+held-out profilano, micro giudicano), altrimenti circolarità da dichiarare.
- **Soglia KS-A** — Matsakis: banda ≤5 ns/iter E uplift ≥2% o chiusa; Hoare: banda ≤ metà di 10 E mediano ≥2%, decade a fine S-118; Klabnik: archiviazione in 1 sessione, si tiene solo LTO se gratis.
- **NaN-boxing su indici** — Hoare: vietato tout court (R4). Matsakis: C3 su soli indici resta ultima carta, previa decisione utente esplicita.

## PRIORITÀ PER L'ORDINE S-117 (max 3)
1. **Spike A′ in un atto**: script PGO+LTO+cgu=1+order_file; misura: build ×2 ⇒ hash identico, gate pieni (batteria rc da file, corpus per NOME ×2, parità) sul binario A′.
2. **Ri-banda sul binario A′**: ≥2 leve nulle per il primo campione N=2; misura: verdetti-nulla committati; meter-riparato se max(banda) ≤5 ns/iter (soglia Matsakis; Hoare: ≤ metà dell'attuale).
3. **Taglia del guadagno + rigiudizio L-A**: micro R=5 su A′ e ricompila 2c18b2e; misura: uplift mediano globale ≥2%, altrimenti scatta KS-A.

## KILL-SWITCH CONSOLIDATI
- **KS-A**: build non riproducibile ×2 (o `.profdata` non riproducibile ⇒ resta solo order-file+LTO) O (uplift <2% E banda non più stretta) ⇒ A′ si archivia; si tiene solo ciò che è gratis.
- **KS-B**: vagone che fallisce fedeltà esce, il treno non muore; treno bocciato 2 volte DOPO bisezione ⇒ B sospeso, si apre C (Matsakis: due bocciature per accumulo tasse ⇒ C2 diretto).
- **KS-D**: vagone senza direzione firmata non si imbarca.
- **KS-C/trigger C**: divergente per sedia (vedi CONFLITTI); da sciogliere con decisione utente pre-registrata, come la formula di promozione R5-Klabnik.
