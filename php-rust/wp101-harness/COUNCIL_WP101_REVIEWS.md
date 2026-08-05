# COUNCIL_WP101_REVIEWS — Concilio WP-101 su S-99.0 e programma S-100 (2026-08-05)

Protocollo a due fasi (regola utente 2026-08-02): fase 1 = nove verbali indipendenti; fase 2 = quattro team tematici (flip-contratto HO+MA+HE · evidenza-server KL+PE · coda-stack BA+ST · misura-attribuzione LE+GR). Verbali individuali = fonte VINCOLANTE.

════════════════════════════════════════════
## FILE: verbali/verbale-1-hoare.md

# Verbale Sedia 1 — Tony Hoare (design linguaggio/runtime, safe-only) — Concilio WP-101

**Oggetto**: report S-99.0 + bozza §S-100. **Perimetro**: sigillo eager, controfattuale statico, forma INT1, catena pin, bozza promozione.

## VERDETTO: CON EMENDAMENTI (una refutazione capitale sulla bozza §S-100, punto 2)

Riconosco ciò che regge: il controfattuale 4a è STATICO ma onesto — ho riletto `run.rs:1005-1074` e il corpo BinarySS/SC/Dst è esattamente come descritto (borrow su slot, guardie `matches!(Undef|Ref)`, `binary_fast` inline, zero call/marshalling/pop); il rimovibile è davvero solo carico+match del payload, e il criterio di riapertura (D_registro ≥ 0,7 misurato) è pre-registrato, non un editto. La patch INT1 è genuinamente di sola misura: pop/pop/push preserva la semantica (miss path identico, smoke byte-id), albero ripristinato, target-dir separato. Ho verificato il buco-sigillo richiesto: **l'unico lettore di `PHPR_REG_LOWER` è `reg_lower::enabled()`** — nessun canale secondario legge l'ambiente dopo il boot.

## REFUTAZIONE CAPITALE

**R1 — Il flip del default (§S-100 punto 2) è sottospecificato al punto da rendere incoerenti i suoi stessi gate.** `enabled()` è PRESENCE-based (`var_os(..).is_some()`: anche `PHPR_REG_LOWER=0` significa ON, trappola già latente oggi). "Pass registro ON senza env" non dice COME si ottiene il modo OFF dopo il flip: senza un contratto value-based, il gate A-PE-100-4 (braccio flag-OFF nel funnel) diventa irraggiungibile, i due bracci del dente anti-putenv (set→on rifiutato / unset→off rifiutato) vanno RI-DERIVATI sulla mappatura nuova, e la guardia di premessa M5 in `reg_lower.rs:597` (`assert!(!enabled())`) FALLIRÀ o mentirà nella batteria standard — la cifra 1726/0 non sopravvive al flip così com'è scritto. Un invariante non nominato non è mantenibile: il contratto d'ambiente va scritto PRIMA di flippare.

## EMENDAMENTI

- **A-HO-101-1 "Sigillo di tipo, non allowlist di call-site"**: la garanzia attuale è una convenzione su DUE main; ogni terzo embedder del runtime (test in-process via `run_source`, futuri bin, pool) resta non sigillato — cura enumerabile contro attacco non enumerabile (lezione WP-96). Prima del flip, forzare l'init a un chokepoint di costruzione del motore (token di boot richiesto dalla compile-entry, stile VmGate ZST di WP-83) o, minimo, dente che ogni `[[bin]]` linkante php-runtime sigilli.
- **A-HO-101-2 "Contratto d'ambiente del flip"**: parse value-based nominato (es. assente→ON, `=0`→OFF), ri-derivazione dei due bracci anti-putenv e della premessa M5, unit-cache key che continua a distinguere i modi. Rimedio di R1.
- **A-HO-101-3 "Timing non prova bit-identità"**: "5,43→5,44 ⇒ emissione davvero bit-identica" è un'inferenza invalida (uguaglianza di misura → proprietà statica). Declassare a ipotesi; l'unica evidenza ammessa è il dump-diff (A-HE-100-4), da eseguire sull'albero candidato al flip.
- **A-HO-101-4 "Identità del pin"**: un hash che "churna col relink — fa fede HEAD" rende il campo hash del PIN_REGISTRY non autoritativo: due binari diversi possono reclamare lo stesso pin. Registrare la tripla (commit, ricetta, hash esatto) e legare ogni `collaudato: sì` all'hash SU CUI il collaudo è girato.
- **A-HO-101-5 "57/43 non è una tariffa"**: la decomposizione INT1 assume additività su tre binari distinti (layout/I-cache non controllati) e la banda [0, 0,5] di 4a è convenzione (½ pavimento sonda), non misura. Pubblicare sempre come bande; vietare l'ereditarietà delle quote 57/43 fuori dal percorso pila (stessa classe dell'errore D=6,07 appena sepolto).

## KILL-SWITCH

- **KS-HO-101-1**: flip VOID se eseguito prima di (a) contratto d'ambiente nominato + dente anti-putenv ri-derivato, (b) dump-diff off/on sullo stesso albero candidato.
- **KS-HO-101-2**: ogni claim d'identità d'emissione fondato su timing anziché dump-diff ⇒ il gate corrispondente è NON soddisfatto.
- **KS-HO-101-3**: batteria post-flip con premesse invertite non aggiornate (M5 e affini) ⇒ la cifra 1726/0 non vale come gate di promozione.

════════════════════════════════════════════
## FILE: verbali/verbale-2-matsakis.md

# Verbale sedia 2 — Niko Matsakis (ownership/aliasing/borrow) — Concilio WP-101

Oggetto: S-99.0 (pre-misura rollout, patch INT1, controfattuale registro) + bozza S-100 (promozione flag-on).

## VERDETTO

**La lettura ownership del controfattuale 4a REGGE; la fedeltà drop/clone di INT1 REGGE; ma il verbale porta TRE refutazioni capitali sul contorno** — l'attribuzione 57/43 poggia su una sottrazione CROSS-TREE, il flip come abbozzato SPEGNE H-B2 sui siti stack residui dell'emissione on, e A-MA-100-2 è evaporata dal registro.

Conferme dal codice (run.rs:1005-1066, HEAD): il hit-path di BinarySS/SSDst/SC è due prestiti condivisi di `frames[top].slots` + guardie `matches!` + `binary_fast` `#[inline(always)]` su prestiti + push/`reg_store_slot` del risultato. Zero clone, zero drop non banali, zero marshalling. Una BinaryAddSS rimuoverebbe SOLO il carico del payload `b` e il match inlineato: predizione D=0 coerente. Aliasing sano: `l==r` è legale sotto prestiti condivisi; `dst==l|r` in SSDst è sicuro per NLL (res computata a prestiti chiusi, semantica eval-then-assign di PHP); slots e stack sono campi disgiunti; su `?` di `reg_store_slot` res viene droppata senza stato parziale. INT1 (patch): sul hit entrambe le forme droppano due Long banali (C2: overwrite in-place + rhs a fine scope; INT1: lhs+rhs a fine scope) — parità drop/clone confermata, miss-path identico modulo ordine dei pop.

## Refutazioni capitali

**R1 — C0 è un ALBERO DIVERSO.** C0 = stash `phpr-s97-ha1`; INT1/C2 = HEAD±patch. Solo INT1−C2 (43%, traffico Vec) è same-tree-pulita; C0−INT1 (57%, "call/marshalling") sottrae attraverso il drift di S-98. In più INT1 cambia una cosa NON dichiarata: il tag-check di lhs passa da lettura through-Vec (`stack.last()` + strato Option) a locale posseduto — un termine di addressing finisce nella quota "call/marshalling". La direzione è robusta, le cifre 57/43 NO.

**R2 — Il flip ritira H-B2 dove non si fonde.** `reg_lower.rs:184-187`: «the production flag-on pipeline only ever emits `Binary(Add)`». Sotto emissione on i siti stack NON fusi perdono la specializzazione BinaryAdd (il −16,2% tenuto). E il punto 4 della bozza pre-registra il controfattuale Sub/cmp sul giudice flag-OFF — un modo in via di ritiro. Il giudice giusto post-flip sono i siti stack RESIDUI dell'emissione on.

**R3 — Ledger leak.** A-MA-100-2 (wildcard `_ => None` in `bin_op_of`, reg_lower.rs:188-194) non è né nei saldati né nel backlog della rotazione: caduta in silenzio. Resta viva: una futura variante specializzata (BinarySub…) de-fonde in silenzio — l'esatto anti-pattern che i match esaustivi S-96 hanno bandito.

## Emendamenti

- **A-MA-101-1**: prima di citare 57/43 come decomposizione ereditabile, costruire C0' SAME-TREE (HEAD con emissione BinaryAdd disattivata); fino ad allora pubblicare solo l'ordinamento («call+marshalling ≥ metà») come banda.
- **A-MA-101-2**: saldare o re-iscrivere A-MA-100-2 — `bin_op_of` a match esaustivo sugli Op Binary-like (variante nuova ⇒ NON COMPILA).
- **A-MA-101-3**: prima del flip, decidere CON MISURA la sorte di H-B2 sotto emissione on (estendere BinaryAdd ai siti stack residui della pipeline on, oppure pre-registrare la rinuncia); spostare il giudice del punto 4 sui residui post-flip.
- **A-MA-101-4**: «ramo costante per sito» conflaziona sito bytecode e sito macchina: il match di `binary_fast` inlineato nell'arm BinarySS è UN branch hardware condiviso da tutti i siti — la banda [0, 0,5] è provata sul micro a pochi siti, non su stream misto di BinOp. La sonda dell'eventuale riapertura (D≥0,7) va fatta su stream MISTO.

## Kill-switch

- **KS-MA-101-1**: il flip NON riusa `PHPR_REG_LOWER` presence-tested né con significato invertito — oggi `enabled()` è `is_some()`: `PHPR_REG_LOWER=0` ACCENDE il pass. L'opt-out post-flip è value-parsed; sigillo eager + dente anti-putenv ri-collaudati sulla semantica NUOVA, pena flip VOID.
- **KS-MA-101-2**: nessuna cifra 57/43 fuori-banda in criteri futuri finché C0' same-tree non esiste; ogni criterio derivato da essa è VOID.

════════════════════════════════════════════
## FILE: verbali/verbale-3-klabnik.md

# Verbale sedia 3 — Klabnik (spec, testabilità, matrici e gate) — Concilio WP-101

## VERDETTO

S-99.0 è una sessione di misure solida e di onestà rara (il pin refutato a
tavolino, il gate che urla su sé stesso). Ma due denti dichiarati NON possono
mordere, un claim di identità è dedotto da un cronometro, e la bozza §S-100
contiene tre ambiguità che renderebbero il flip del default un salto con gate
di carta. **Refutazioni capitali: sì (R1–R3).**

## Refutazioni capitali

**R1 — `reg_lower_antiputenv.rs`: la selezione del chunk può giudicare l'unità
sbagliata.** `included_chunk` prende il PRIMO chunk di `split("== unit ")` che
`contains(inc_name)`; ma il sorgente del main contiene il path dell'included
come letterale (`include '…/antiputenv-inc-….php'`). Se il dump del `{main}`
stampa quella costante, il primo match è il chunk del MAIN: nel braccio
set→on è compilato flag-off (nessuna forma ⇒ negativa passa vacuamente), nel
braccio unset→off è compilato flag-on (forme presenti ⇒ positiva passa
vacuamente). Entrambi i bracci possono passare senza aver mai guardato
l'unità inclusa. Sommato al compile-order accident già dichiarato, il dente
CLI è doppiamente senza denti.

**R2 — Il sigillo non ha UN test che possa fallire, e il server non è mai
stato eseguito flag-on.** CLI: ammesso nel test stesso. Server: «chiuso per
costruzione» = zero collaudo dinamico; `s99-parity-server.sh` costruisce
l'ambiente con `PHPR_REG_LOWER` ASSENTE, quindi sentinella+option+restapi
hanno collaudato il pin server SOLO flag-off. La promozione a default
accenderebbe di colpo un modo in cui il server non ha mai servito una
richiesta.

**R3 — «Emissione davvero bit-identica» dedotta da 5,43→5,44.** Un cronometro
non prova bit-identità; la prova è il diff dei dump (A-HE-100-4). Il claim va
declassato a «compatibile con»: com'è scritto, dichiara ciò che non prova.

## Refutazioni minori

**R4** — Il gate corpus è un bit/fail per NOME: 1418 nomi uguali non dicono
COME falliscono, né confrontano off↔on tra loro (solo contro la lista wp82).
A-KL-100-2 è senza definizione operativa: quale diff, normalizzato come,
criterio di PASS scritto dove?

**R5** — `pair99.sh names()`: `sed 's/^[0-9]*) …/'` cattura qualunque riga
«N) » anche nei corpi dei messaggi; e «media: 0 nomi IDENTICI» è
indistinguibile da estrattore morto (stessa classe del morso (c)). Nemmeno il
corpus gate ASSERISCE che `wc -l` dei nomi quadri con «failures: N» del
runner: lo stampa e basta.

**R6** — Matrice REG_FORMS asimmetrica: `BinarySS` è negato in antiputenv ma
nessun controllo positivo prova che il corpo lo emetterebbe (negativa vacua);
il braccio unset si accontenta di `any()` mentre il funnel esige tutte e tre
le forme: una regressione parziale del pass passa.

**R7 — Ambiguità §S-100**: (i) post-flip, «flag-OFF» non ha semantica: oggi
il flag è presenza/assenza — serve un disable esplicito (`PHPR_REG_LOWER=0`)
pinnato nel funnel PRIMA del flip, o A-PE-100-4 è ineseguibile; (ii) la
«parità server (script 1a)» con lista chiusa non può esprimere i DUE modi
senza modifica del launcher; (iii) N_OPS≤255 (186/256, A-LE-100-3) assente
dai gate del flip mentre la coda H-B2 aggiunge opcodi.

## Emendamenti

- **A-KL-101-1**: fix selezione chunk (match sull'header esatto dell'unità,
  non substring) + assert che il chunk scelto NON sia il `{main}`.
- **A-KL-101-2**: braccio unset esige TUTTE le forme del funnel; ogni forma
  negata ha il suo controllo positivo o esce dalla matrice.
- **A-KL-101-3**: assert conteggi↔nomi in pair e corpus gate (estratti ==
  dichiarati dallo strumento).
- **A-KL-101-4**: definizione operativa di A-KL-100-2: diff normalizzato
  per-test off↔on, criterio pre-registrato = zero differenze.
- **A-KL-101-5**: semantica del disable post-flip definita e pinnata nel
  funnel PRIMA del flip.
- **A-KL-101-6**: launcher parità bimodale (parametro di modo nella lista
  chiusa).

## Kill-switch

- **KS-KL-101-1**: VIETATO il flip del default finché il pin server non è
  collaudato flag-on (sentinella + option + restapi); violazione ⇒ flip VOID.
- **KS-KL-101-2**: ogni claim «bit-identico» senza diff di dump/output ⇒
  VOID; il cronometro non giudica l'identità.
- **KS-KL-101-3**: un gate i cui nomi estratti non quadrano col conteggio
  dichiarato dallo strumento è VOID: urla su sé stesso prima che sul motore.

════════════════════════════════════════════
## FILE: verbali/verbale-4-hejlsberg.md

# Verbale SEDIA 4 — Hejlsberg (pipeline di emissione, unit-cache/modes) — Concilio WP-101

## VERDETTO

**L'obiettivo della promozione è AMMISSIBILE; la bozza §S-100 come scritta è VOID nei punti 1–2**: l'ordine dei gate è invertito, il contratto di selezione del modo dopo il flip non è nemmeno nominato, e uno dei "gate già soddisfatti" poggia su un'evidenza sovradichiarata che l'unico strumento capace di verificarla (il dump) oggi non può verificare.

## Refutazioni capitali

**R1 — Il flip non ha un contratto di modo: il collaudo "nei DUE modi" rischia il falso verde stesso-modo.** Oggi `enabled()` è `env::var_os("PHPR_REG_LOWER").is_some()` (reg_lower.rs:50-53): QUALUNQUE valore, incluso `PHPR_REG_LOWER=0` o stringa vuota, significa ON. La bozza ordina "flip del default + corpus per NOME nei DUE modi" senza dire COME si ottiene l'OFF a default invertito. Se l'opt-out viene speso come `=0` sull'attuale `is_some()`, i due bracci del funnel e di `s99-corpus-gate.sh` girano ENTRAMBI on: gate identici per costruzione = forgia che fallisce in silenzio [[feedback-forge-silent-failure]]. Anche i due bracci del dente anti-putenv (set→on rifiutato, unset→off rifiutato) sono scritti per default=off e vanno riscritti, come i tre launcher.

**R2 — "Emissione flag-on davvero bit-identica" è dedotta da un delta timing (5,43→5,44), non da un diff di emissione.** Invarianza di cronometro ≠ identità di bit. Lo strumento che può dichiararla — dump/`lowered()` — è CIECO sugli hook riscritti (è esattamente A-HE-100-4), che la bozza mette ULTIMO nel punto 1. Ordine sbagliato: la sanatoria del dump è il PRE-REQUISITO del dump-assert di A-PE-100-4 e del tripwire A-HE-100-1; asserire "zero forme registro" con un dump cieco è un controllo positivo fallito.

**R3 — `visit_addrs` ha `_ => {}` su `Op` (reg_lower.rs:74): la promozione trasforma un footgun latente in un vettore di corruzione di default.** Il commento stesso ammette: variante nuova con `Addr` non aggiunta lì ⇒ il pass corrompe gli indirizzi. È la stessa classe che S-96 ha eliminato altrove ("una variante nuova di `Op` ora NON COMPILA"). Con flag-on opt-in, il danno colpisce chi opta; con default ON colpisce TUTTI, silenziosamente. A-HE-100-2 non è backlog: è BLOCCANTE per il flip.

**R4 — RC-1 si INVERTE col flip e la bozza non lo vede: il percorso OFF diventa il pin incollaudato.** Dopo il flip, OFF è il kill-switch di rollback — e nessun ordine della bozza lo tiene collaudato nel tempo. Un rollback mai eseguito è il pin 365f4d40: refutabile per costruzione al primo uso.

## Classificazione gate (mandato)

**BLOCCANTI per il flip**: A-HE-100-4 (primo: è lo strumento), A-HE-100-2 (match esaustivo), A-HE-100-1 (tripwire, col dump sanato), A-HE-100-3 (differenziale BinaryAdd≡Binary(Add) su overflow/coercizioni/union/warning — costa un file, il corpus non è prova: "corretto per fortuna del corpus" ≠ corretto). **Backlog**: nulla dei quattro; restano backlog i gate non-HE non nominati dal mio perimetro.

## Emendamenti

- **A-HE-101-1**: PRIMA di ogni riga del flip, definire il contratto di modo (opt-out value-parsed, non `is_some()`), riscrivere i due bracci del dente e i tre launcher per il default nuovo.
- **A-HE-101-2**: riordinare il punto 1: A-HE-100-4 in testa; A-HE-100-2 promosso bloccante; poi A-PE-100-4/A-KL-100-1/2 sugli strumenti sanati.
- **A-HE-101-3**: un controllo positivo sulla claim "la chiave unit-cache porta il modo" (doc di reg_lower.rs, mai collaudata): con cache TL-al-MAIN e modo sigillato è cintura ridondante — va o provata o riclassificata, non contata come gate soddisfatto per documentazione.
- **A-HE-101-4**: il funnel-probe esca da `{main}`: almeno un corpo non-main (funzione/chiusura) asserito nei due bracci — è il punto cieco stesso di A-HE-100-4.

## Kill-switch

- **KS-HE-101-1**: flip VOID se i due bracci del funnel non provano emissione DIVERSA su una probe (hash dei due dump pubblicati) — anti falso-verde stesso-modo.
- **KS-HE-101-2**: nessun flip finché `visit_addrs` conserva un braccio wildcard su `Op`.
- **KS-HE-101-3**: post-flip, ogni rotazione pin include un braccio flag-OFF collaudato (campo `modo:` in PIN_REGISTRY); OFF non eseguito ⇒ pin non collaudato.
- **KS-HE-101-4**: "bit-identico" si dichiara SOLO da diff di dump/emissione, mai da un cronometro.

════════════════════════════════════════════
## FILE: verbali/verbale-5-bak.md

# Verbale Sedia 5 — Lars Bak (microarchitettura, path caldi, alloc-rate)
## Concilio WP-101 su S-99.0 + bozza §S-100

## VERDETTO

Metodo della pre-misura SOLIDO (target-dir separato, albero ripristinato,
smoke semantico, stessa finestra, tre binari): la DIREZIONE regge — rollout
nelle forme registro morto, percorso pila vivo. Ma la decomposizione 57/43
è pubblicata come quanto fisico quando è uno split dipendente dall'ordine
di rimozione; il criterio pre-registrato 0,7 ns siede SOTTO il pavimento
dichiarato della sonda; e §S-100 punto 3 ripropone il metodo
conteggio×costo senza il profilo a campioni (A-BA-100-3) che avevo chiesto.

## Refutazioni capitali

**R1 — Le componenti di D SI MESCOLANO.** INT1−C2=2,67 non è "traffico Vec"
puro: il pop di lhs in INT1 aggiunge un branch (Option/expect) + memcpy 24B
+ store di len, e il push un capacity-check; C2 sul hit fa last()+last_mut()
(doppio check) + store in place. Su un core OoO questi micro-costi si
SOVRAPPONGONO col carico del payload: C0−INT1 e INT1−C2 sono UN ordine di
rimozione, il termine d'interazione è attribuito in silenzio a una gamba.
Inoltre lo spread INT1 (0,05 s ≈ ±0,33 ns/occ) rende lo split 52–62%, non
"57%": la mia stima 50-70% è "confermata" da una misura la cui banda copre
mezzo intervallo. Si pubblichi come BANDA con l'interazione nominata.

**R2 — Kill-switch sotto il pavimento.** premisura-rollout99.out dichiara
pavimento sonda 1,0 ns (KS-BA-100-2) e criterio di riapertura D_registro ≥
0,7 ns/occ: un criterio SOTTO la risoluzione della sonda non è
aggiudicabile — non può né mordere né assolvere onestamente.

**R3 — Il controfattuale "branch costante = gratis" vale al MICRO.** Con 3
siti caldi il predittore mangia; con 186 corpi e centinaia di copie inlined
di `binary_fast` la pressione BTB/PHT su WordPress è un'altra bestia
(lezione V8). La banda [0, 0,5] vale per il giudice micro; NON esportarla
al macro senza un campione.

**R4 — Provenienza stantia nel file VERDICT.** micro-rebaseline99.out ha in
testa "S-97.0" e pin 4e268c3f (pre-sigillo) mentre il pin corrente è
52330330; e la sessione pubblica DUE arith flag-off (7,52 rebaseline vs
7,44 gamba 4c) senza dire quale è LA baseline: 17,5 diventa 17,3 con
l'altra gamba. R=3 con spread fino a 0,09 s: le cifre decimali dei rapporti
sono al limite della risoluzione.

**R5 — Il census del punto 4 rischia il bersaglio sbagliato due volte.**
(a) Va eseguito sull'emissione POST-FLIP (il punto 2 ritira i siti
register-eligible: frequenze misurate su un'emissione in pensione =
census void). (b) `cmp int-int` ai loop-head vive in CmpJmp/CmpJmpConst
(run.rs:982-1004), che chiamano binary_value col plumbing INTERO
(call+marshalling+push del Bool evitato solo a metà): un census che conta
solo Binary(cmp) manca il consumatore più caldo del meccanismo H-B2.

## Emendamenti

- **A-BA-101-1**: pubblicare la decomposizione come bande
  (call/marshalling 52–62%) col termine d'interazione dichiarato; INT2
  (in-place tenuto, call reintrodotta) SOLO se una decisione di rollout
  dipenderà dallo split — oggi è descrittivo, non decisionale.
- **A-BA-101-2**: la prima misura H-C (§S-100 punto 3) include il profilo
  a campioni sul loop di `prop.php` (A-BA-100-3 eseguito, entrambi i
  motori) come gamba CO-EQUALE alla tavola conteggio×costo — la classe
  "conteggio senza cammino critico" è già stata refutata in S-98.
- **A-BA-101-3**: census del punto 4 = frequenza per-op sul giudice con
  emissione post-flip, forme fuse (CmpJmp*) contate a parte; l'atteso si
  scala per frequenza PRIMA di scegliere l'occorrenza.

## Kill-switch

- **KS-BA-101-1**: ogni sonda futura su D_registro pubblica una BANDA e il
  suo pavimento; se criterio < pavimento, la sonda è VOID e il criterio va
  rialzato al pavimento.
- **KS-BA-101-2**: se il census post-flip mostra che l'occorrenza scelta
  pesa così poco che atteso < risoluzione della coppia di misura,
  l'occorrenza CADE A TAVOLINO prima di ogni riga di codice.

════════════════════════════════════════════
## FILE: verbali/verbale-6-pedersen.md

# Verbale Sedia 6 — Pedersen (confine per-richiesta, lifecycle, ambiente) — Concilio WP-101

## VERDETTO

**Esecuzione di R1: esemplare.** Il pin 365f4d40 è stato refutato PRIMA di ogni uso
(la refutazione "senza eseguire nulla" — manca `axum-server` — è la conferma più
forte possibile del sospetto WP-100), il pin nuovo è nato CON ricetta, la ri-parità
post-sigillo è sulla ROTAZIONE GIUSTA (a838866e @ HEAD 4c34f61, script fail-closed su
`PIN_SRV_ATTESO`, phpr ruotato a 52330330 con corpus off+on). Il sigillo è davvero il
primo atto UTILE dei due main (`logging::init()` lo precede ma non esegue codice PHP:
fuori dal modello di minaccia putenv). La nota di metodo nel `.out` è onesta.
**Ma due dichiarazioni sono più larghe dell'evidenza** — refutazioni sotto.

## Refutazioni capitali

**R1 (capitale) — «php-server collaudato: sì» poggia, per la gamba SERVER, su DUE
richieste sequenziali identiche.** Option 413 + restapi 3508 girano via phpr CLI
(stesso albero, binario DIVERSO): collaudano l'emissione, non il lifecycle del
server. La sentinella G-APERTURA-2 verifica il reset (GLOBALS/static/object_id) tra
richiesta 1 e 2 — niente payload diversi interleaved, niente richiesta 3+ (i leak
monotoni tipo WP-78 mordono dopo), niente workers>1, niente restapi via HTTP. Il
registro deve GRADUARE: `collaudato: sì (emissione CLI + smoke server 2-req)`, non
un «sì» secco. La classe «dichiarazione più larga dell'evidenza» è la stessa del pin
effetto-collaterale, spostata dal binario al verbo.

**R2 (capitale, sulla bozza S-100) — il flip del default INVERTE il significato di
«flag ASSENTE» e nessun gate della bozza lo nomina.** `reg_lower_antiputenv.rs`
braccio 1 asserisce «assente ⇒ zero forme registro»: dopo il flip è FALSO per
costruzione. `s99-parity-server.sh` dichiara «PHPR_REG_LOWER ASSENTE per
costruzione» come garanzia di flag-OFF: dopo il flip lo stesso script collauda
silenziosamente il modo NUOVO. E l'opt-out (`PHPR_REG_LOWER=0`? unset?) oggi non ha
una grafia definita né un dente che lo sigilli. Punto 2 della bozza incompleto.

**R3 (non capitale) — il registro pin chiude la classe solo per la feature axum.**
Il gate meccanico vero è la sentinella (esercita `--axum`); l'hash sha256/16
certifica «è il binario atteso», NON «è costruito con la ricetta» (altre feature —
mem-census, allocatore — non lasciano traccia; e il phpr-hash «churna col relink»
è già l'ammissione che l'hash è identità debole). Nel run di parità l'hash phpr è
«registrato, non gate»: la gamba B non è vincolata al binario che l'ha eseguita.

## Emendamenti

- **A-PE-101-1**: `PIN_REGISTRY.md` — campo `collaudato:` GRADUATO per gamba
  (emissione-CLI / server-smoke / server-N-req / server-HTTP-suite); vietato il «sì»
  secco quando le gambe differiscono.
- **A-PE-101-2**: census di TUTTI i `std::env::var("PHPR_*")` (runtime+compile) con
  classificazione eager/lazy; ogni flag lazy che tocca emissione o semantica del
  motore riceve il sigillo o la motivazione scritta. (PHPR_DUMP_OPS incluso.)
- **A-PE-101-3**: sentinella estesa: N≥16 richieste, payload interleaved diversi,
  workers>1, almeno una richiesta restapi-shaped via HTTP — prerequisito del «sì»
  server pieno, obbligatoria in modo nuovo dopo il flip.
- **A-PE-101-4**: PRIMA del flip: grafia dell'opt-out definita e documentata;
  riscrittura dei due bracci anti-putenv nel modo nuovo (assente⇒on, =0⇒off, putenv
  impotente in TUTTE le direzioni); aggiornare il commento-garanzia dello script di
  parità.
- **A-PE-101-5**: `--build-info` nei due binari (feature abilitate + HEAD): il gate
  pin verifica ricetta, non solo hash; gamba B gate ANCHE sull'hash phpr.

## Kill-switch

- **KS-PE-101-1**: il flip del default NON si dichiara eseguito finché i bracci
  anti-putenv riscritti (A-PE-101-4) e la parità server in ENTRAMBI i modi non
  passano sulla STESSA rotazione.
- **KS-PE-101-2**: nessuna cifra WordPress attribuita a un pin server la cui riga
  registro non espliciti la gamba di collaudo che la copre.

════════════════════════════════════════════
## FILE: verbali/verbale-7-leijen.md

# Verbale sedia 7 — Daan Leijen (allocatore, footprint fisico, layout) — Concilio WP-101

## VERDETTO

**APPROVO CON RISERVA.** Il mio A-LE-100-1 è RISPETTATO: `pair99.sh` applica
`/usr/bin/time -l` + `MIMALLOC_PURGE_DELAY=0` su tutte e quattro le gambe,
ricetta identica a pair94, identity pinnata (phpr 4e268c3f, oracle 8.5.7
Jun-2, rustc 1.96.0). I rapporti sono derivazione meccanica dai `.time` —
verificati a mano sui raw: tornano tutti. Ma due letture del report vanno
corrette e la bozza S-100 è CIECA sul footprint del flip.

## Refutazioni capitali — SÌ (una)

**R1 (capitale) — la lettura (b) del media peak è INCOMPLETA nei report.**
"Il rapporto media si muove per la gamba ORACLE" è vera solo a metà: dai
raw, oracle 346.325.904→445.809.528 B (+28,7%) ma **phpr
1.170.785.648→1.202.701.752 B (+2,7% = +31,9 MB)** — la gamba phpr media
NON è piatta e nessun report lo dice (REPORT_GAP_99 scrive solo "phpr
1202,7 MB"; collaudo99 dichiara piatta solo la full, correttamente, ma
attribuisce il movimento dei rapporti "media/full peak" alla sola gamba
oracle). +31,9 MB senza banda di spread né attribuzione non è "piatto":
è NON MISURATO.

**R2 — il denominatore peak è uno strumento non calibrato.** L'oracle
(stesso binario brew, stessa suite, 24h di distanza) si muove +12,1% full
e +28,7% media. Nessuno ha mai misurato lo spread del peak oracle
stessa-sera: finché non esiste un controllo positivo del metro (due run
oracle adiacenti), ogni rapporto peak è uno strumento con ±30% di
incertezza sul denominatore. La lettura (a) sul FULL è invece **provata
dai raw** (pair94-ratios + pair99-ratios, entrambi output macchina):
phpr −0,45%, oracle +12,1% — la frase è supportata, la causa no.

**R3 — la coppia fotografa un binario già superseded.** Il peak 1892,56
MiB è del pin 4e268c3f (S-98); la sessione ha poi ruotato a 52330330 (col
sigillo eager). Corpus per NOME identico ≠ footprint identico. Tollerabile
solo perché S-100 ri-fotografa in modo flag-on (regola n.2).

**R4 — (c) A-LE-100-3 resta APERTO e la bozza S-100 non lo nomina.**
Oggi 186/256 col BinaryAdd. Il flip flag-on (punto 2) non aggiunge shape
(le forme registro esistono già nell'enum), ma il punto 4 (Sub/cmp
int-int stack-path) SÌ: ogni specializzazione aggiunge varianti. Peggio:
una variante col payload sbagliato allarga `size_of::<Op>()` — cioè lo
STRIDE di ogni istruzione di ogni unità compilata: unit cache, arena del
preludio (i sei huge di WP-93), icache. La bozza non ha né il budget
shape né il gate di taglia.

**R5 — il flip cambia lo stream di istruzioni e nessuno misura la taglia
unità.** H-A1: flag-on = 19→11 op/iter — meno op per corpo, stessa
taglia/op ⇒ le unità compilate DOVREBBERO calare, ma non esiste una
misura bytes-unità off vs on. La roadmap footprint (d) è ferma proprio
perché manca l'asse compile-side fresco: la coppia peak da sola non lo dà
(un calo unit cache può essere mascherato da crescita runtime nel peak).

## Emendamenti

- **A-LE-101-1**: nel collaudo flag-on di S-100, il gate footprint della
  promozione è **phpr-off vs phpr-on stessa-sera** (stesso metro, stesso
  malloc), MAI il rapporto vs oracle (R2). In più: un doppio run oracle
  media adiacente, una volta, per bandare lo spread del metro.
- **A-LE-101-2**: al collaudo flag-on aggiungere una sonda taglia-unità
  (bytes totali delle unità compilate sui sei micro o sul corpus) off vs
  on — è l'asse che rianima la roadmap footprint (d).
- **A-LE-101-3**: ogni nuova variante Op del punto 4 porta nel commit:
  N_OPS aggiornato (gate ≤255, const-assert) + `size_of::<Op>()`
  invariato dichiarato.
- **A-LE-101-4**: sanare nei report la riga media-peak con le DUE gambe
  (R1).

## Kill-switch

- **KS-LE-101-1**: se al flip il peak phpr-on supera phpr-off stessa-sera
  oltre lo spread bandato, la promozione si FERMA finché la crescita non
  è attribuita per owner.
- **KS-LE-101-2**: se `size_of::<Op>()` cresce per una specializzazione
  del punto 4, la variante è RESPINTA (ripiegare il payload), qualunque
  sia il suo D misurato.

════════════════════════════════════════════
## FILE: verbali/verbale-8-stogov.md

# Verbale sedia 8 — Dmitry Stogov (Zend engine, semantica PHP) — Concilio WP-101

Oggetto: S-99.0 + bozza §S-100. Mandato: REFUTARE.

## VERDETTO

**A-ST-100-1 (controfattuale registro): ESEGUITO BENE, conclusione VALIDA
nel suo perimetro — ma il perimetro non è dichiarato per intero.
Bozza S-100: DUE refutazioni capitali** (precondizione AssignOp sparita
dai gate della promozione; perimetro compare non dichiarato al punto 4).

## Giudizio sull'esecuzione di A-ST-100-1 (gamba 4a)

Letto `run.rs:1005-1070` a HEAD contro `premisura-rollout99.out`. Il
ragionamento regge: nelle forme SS/SSDst il hit-path è già borrow → guardie
→ `binary_fast` inline → push/store; una `BinaryAddSS` rimuoverebbe solo il
payload BinOp e il `match b`, ramo costante per sito. **Corretto anche il
non-detto esplicito: le guardie `matches!(Undef|Ref)` NON sono rimovibili**
— Undef porta il warning LoadVar, Ref il deref — e nessun risparmio vi è
stato dichiarato. Banda [0, 0,5] pubblicata, criterio D_registro ≥ 0,7
pre-registrato: metodo rispettato. Due incompletezze semantiche (non
capitali, la conclusione per Add int-int sopravvive):

1. **Il rimovibile NON è identico nelle tre forme**: in BinarySC/SCDst il
   hit-path paga anche `func.consts[cidx].to_zval()` (riga 1053) — ~0 per
   Long/Double, ma NON per ZStr (refcount bump). L'enunciato «SOLO carico
   payload + match b» è vero per SS, sovra-ampio come frase generale.
2. **Il controfattuale è hit-path-only**: la banda vale per giudici
   guard-always-hit (arith/add); il `.out` non lo scrive. Su workload
   miss-heavy il costo vive in `reg_load_slot` owned — fuori perimetro.

## Refutazioni capitali sulla bozza S-100

**R1 — Le sette fixture-trappola AssignOp sono SPARITE dai gate della
promozione.** Il backlog di NEXT le nomina (A-ST-100-2/3, «non slot di
sessione se non bloccante») ma il punto 1 di §S-100 elenca solo
A-KL-100-1/2, A-PE-100-4, A-HE-100-4. Col flip del default (punto 2)
l'emissione registro degli AssignOp diventa il percorso di TUTTI —
`$s += $i*3` è esattamente la forma del funnel antiputenv. Una precondizione
nominata che non compare nell'ordine che la consuma è un'equivalenza
dichiarata senza fixture: **bloccante, non backlog**.

**R2 — Il punto 4 tratta «Sub o cmp int-int» come fungibili. Non lo sono.**
Il perimetro di una specializzazione compare è più delicato di Add/Sub: i
cmp_op hanno il ramo __toString (oggetto vs stringa in loose compare, con
side effect e ORDINE osservabili), la loose equality numeric-string vive
SOLO in smart_streq («10»=="1e1", commento run.rs:108), Double porta
NaN/-0.0 nelle forme swapped (Gt = smaller(b,a)), e Spaceship cambia SEGNO
sotto swap degli operandi — rilevante per ogni normalizzazione const-lhs
tipo CmpJmpConst. Se il census sceglie cmp, questo perimetro va DICHIARATO
PRIMA del controfattuale, con fixture per classe; la bozza non lo dice.

## Emendamenti

- **A-ST-101-1**: ri-perimetrare l'enunciato 4a in NEXT: rimovibile
  per-FORMA (SC paga to_zval sul hit-path); banda valida solo
  guard-always-hit; conclusione D≈0 ristretta ad Add int-int.
- **A-ST-101-2**: se il census indica cmp, il controfattuale nasce DOPO la
  dichiarazione scritta del perimetro compare (__toString/ordine,
  smart_streq, NaN/-0.0, segno di Spaceship sotto swap, const_lhs) + una
  fixture per classe.
- **A-ST-101-3**: le sette trappole AssignOp entrano PER NOME nel punto 1
  di §S-100 come gate della promozione.
- **A-ST-101-4**: enumerare le classi di divergenza che vivono SOLO
  flag-on (ordine dei warning Undef con entrambi gli slot Undef in SS;
  fold/normalizzazione const-lhs su op non commutativi e di segno;
  store attraverso Ref-dst in reg_store_slot; fedeltà `lowered()`/dump) e
  mappare ciascuna su un gate esistente o una fixture nuova — il diff
  riga-per-riga (A-KL-100-2) NON fabbrica copertura che il corpus non ha.

## Kill-switch

- **KS-ST-101-1**: VIETATO il flip del default finché le sette fixture
  AssignOp non esistono e non passano byte-identiche nei DUE modi.
- **KS-ST-101-2**: ogni specializzazione compare sul percorso pila scritta
  PRIMA del suo perimetro dichiarato + fixture è VOID.
- **KS-ST-101-3**: il rollout nelle forme registro resta chiuso salvo
  MISURA con D_registro ≥ 0,7 ns/occ — un nuovo argomento statico non lo
  riapre.

════════════════════════════════════════════
## FILE: verbali/verbale-9-gregg.md

# Verbale sedia 9 — Brendan Gregg (metodologia di misura, attribuzione) — Concilio WP-101

## §FONDAMENTALI

**Avanzamento oggetto (mandato inverso — che cosa sappiamo OGGI che ieri non sapevamo, per NOME):**
1. **D decomposto**: 6,27 ns/occ = 3,60 call/marshalling (57%) + 2,67 traffico Vec (43%), misurato con tripletta C0/INT1/C2 stessa finestra, R=5, raw pubblicati (`premisura-rollout99.out`). Conoscenza nuova e ben costruita.
2. **D_registro ≈ 0** (banda [0, 0,5] ns/occ, pre-registrata): le forme registro inlineano già `binary_fast` — il rollout Add lì è refutato PRIMA di scrivere codice. Nuovo.
3. **Emissione flag-on bit-identica sotto H-B2** (arith on 5,43→5,44): prima fotografia delle due gambe INSIEME post-H-B2; vantaggio composto −26,9%. Nuovo.
4. **Il pin server 365f4d40 non poteva funzionare** (feature `axum-server` assente): refutato a tavolino, sostituito e collaudato rc=0. Nuovo (sul perimetro, non sul motore).
5. **Parità WP chiusa per NOME sull'emissione H-A2+H-B2** (debito regola-n.2 saldato). Nuovo come fatto di collaudo.

**Solo ri-fotografato**: la coppia peak/CPU (gamba phpr piatta) e le sei categorie (17,5/13,8/8,6/6,9/4,9/3,8 — stesso ordinamento e grandezza di S-97.0; "H-C/H-D rianimate" è contabilità della gamba oracle sanata, non scoperta). Corretto che sia così: era l'ordine del concilio.

**Contatore misure**: azzerato con merito — sessione di SOLE misure, tre .out macchina con spread e pavimenti.

**Rischio trascurato**: la gamba ORACLE peak si muove del **+28,7% (media: 346,3→445,8 MB)** e **+12,1% (full: 745,6→836,0 MB)** tra WP-94 e S-99 — stessa ricetta, stesso brew 8.5.7 — e nessun file ne dà una causa.

## VERDETTO: PASS con tre refutazioni (nessuna capitale) — la sessione ha prodotto conoscenza vera; il rischio vive nel modo in cui le fotografie peak verranno usate.

## Refutazioni

**R1 — «i rapporti peak si muovono per la gamba oracle» era inferenza non pubblicata.** L'ho verificata io dai raw (`pair94-ratios.out` vs `pair99-ratios.out`): VERA (oracle +28,7%/+12,1%; phpr +2,7%/−0,45%). Ma i .out di S-99 la asseriscono senza la tabella a quattro gambe; e la gamba phpr media peak NON è piatta (+2,7%), è "piccola". La verifica era a un `paste` di distanza e non è stata fatta.

**R2 — l'anomalia oracle resta senza nome.** Un peak che oscilla ~29% tra due sere a ricetta identica dice che O il peak oracle ha varianza run-to-run enorme O qualcosa nell'ambiente è cambiato (brew bump? cache? ASLR/allocator?). Finché non è misurato, OGNI rapporto peak porta un'incertezza potenziale di quella taglia: 2,374× e 2,698× sono fotografie con barra d'errore ignota su un lato.

**R3 — «stessa finestra» ha una cucitura misurabile ma non dichiarata.** arith flag-off netto = 7,52 in `micro-rebaseline99.out` (R=3) e 7,44 in `premisura-rollout99.out` (R=5): 1,1%, FUORI dallo spread pubblicato (0,06). Il rapporto 12,7 (flag-on) incrocia gambe di due blocchi. Innocuo su effetti 13×; da dichiarare quando un rapporto attraversa i file. La parità server con build cargo concorrente è ACCETTABILE: giudice bit/fail, l'errore possibile è solo un falso FAIL da timeout — direzione sicura; quel run non diventi mai cronometro.

## Bozza S-100

Il giudice della promozione (punto 2: coppia stessa-sera in modo nuovo + corpus due modi + parità server) è giusto, ma **manca il criterio di accettazione pre-registrato**: nessuna banda entro cui la coppia flag-on deve cadere per non bocciare il flip. H-C: lo strumento c'è (tavola arith-decomposition S-97 + op census dietro feature); il «confronto col profilo oracle» non nomina lo strumento del lato oracle.

## Emendamenti
- **A-GR-101-1**: pubblicare in S-100 la tabella 2×2×2 delle gambe raw WP-94/S-99 (fatta qui, va nei .out) ogni volta che si attribuisce un movimento di rapporto a una gamba.
- **A-GR-101-2**: PRIMA della coppia flag-on, misurare lo spread run-to-run del peak ORACLE (R≥2 sul media group) e dare un nome all'anomalia +28,7%; fino ad allora i rapporti peak si pubblicano come banda.
- **A-GR-101-3**: pre-registrare le bande di accettazione del flip (full CPU e peak flag-on entro spread di WP-94/99 lato phpr) PRIMA del run.
- **A-GR-101-4**: per H-C, nominare lo strumento della gamba oracle (census opcode o profiler) nel programma, non in sessione.

## Kill-switch
- **KS-GR-101-1**: se la coppia flag-on viene eseguita SENZA banda pre-registrata (A-GR-101-3), il suo verdetto sulla promozione è VOID: resta fotografia, non gate.
- **KS-GR-101-2**: se una gamba oracle di una coppia si muove >10% vs il riferimento senza causa nominata, i rapporti di quella coppia non sono confrontabili cross-sessione finché lo spread oracle non è misurato.

════════════════════════════════════════════
## FILE: verbali/team-flip-contratto.md

# Team «flip-contratto» — Concilio WP-101 (sedie 1 Hoare, 2 Matsakis, 4 Hejlsberg)

Tema: contratto di modo di `PHPR_REG_LOWER` e specifica del flip a default (bozza §S-100).

## Convergenze (unanimi)

1. **Contratto di modo PRIMA di ogni riga del flip.** `enabled()` è presence-based (`is_some()`: anche `=0` ACCENDE) — trappola già latente. L'opt-out post-flip è **value-parsed e nominato** (forma proposta da Hoare: assente→ON, `=0`→OFF); vietato riusare la presence o invertirne il significato (KS-MA-101-1). Col contratto nuovo vanno ri-derivati: i due bracci del dente anti-putenv, la premessa M5 (`reg_lower.rs:597`, pena cifra 1726/0 non valida — KS-HO-101-3), i tre launcher, i bracci del funnel/`s99-corpus-gate.sh` (rischio falso verde stesso-modo = forgia silenziosa, R1 Hejlsberg). Flip senza contratto = VOID (KS-HO-101-1, KS-MA-101-1, A-HE-101-1).
2. **Bit-identità = SOLO diff del dump, mai timing.** "5,43→5,44 ⇒ bit-identico" è inferenza invalida (A-HO-101-3, R2 Hejlsberg, KS-HE-101-4/KS-HO-101-2). Prerequisito: sanare il dump cieco sugli hook (A-HE-100-4) — è lo STRUMENTO, va primo. Dump-diff off/on sullo stesso albero candidato; i due bracci del funnel devono provare emissione DIVERSA su una probe (hash pubblicati, KS-HE-101-1) e la probe esce da `{main}` (A-HE-101-4).
3. **Wildcard su `Op` = bloccanti.** `visit_addrs _ => {}` (R3 Hejlsberg, KS-HE-101-2) e `bin_op_of _ => None` (R3 Matsakis, ledger leak A-MA-100-2): match esaustivi prima del flip, stessa classe S-96.

## Conflitti / posizioni

- **H-B2 sui siti stack non fusi (R2 Matsakis)**: il flip ritira il −16,2% dove l'emissione on non copre. Il team lo registra come **precondizione d'ordine, non conflitto col flip**: prima del flip si decide CON MISURA (estendere BinaryAdd ai residui della pipeline on, o rinuncia pre-registrata) e il giudice del punto 4 si sposta sui residui post-flip (A-MA-101-3). Hoare/Hejlsberg: nessuna obiezione, non coperto dai loro perimetri.
- **Unit-cache key**: Hoare la vuole nel contratto (continua a distinguere i modi); Hejlsberg la dice cintura ridondante mai collaudata (A-HE-101-3). Riconciliazione: un controllo positivo decide — provata o riclassificata, mai contata "soddisfatta per documentazione".
- **Profondità del sigillo**: Hoare chiede chokepoint di TIPO (token di boot stile VmGate, A-HO-101-1) prima del flip; Matsakis/Hejlsberg si fermano al ri-collaudo del dente sulla semantica nuova. Dissenso registrato: minimo comune = dente per ogni `[[bin]]`; il chokepoint resta emendamento Hoare.

## Priorità proposte per l'ordine S-100

1. Contratto di modo value-parsed + ri-derivazione dente/M5/launcher/funnel (A-HO-101-2, A-HE-101-1).
2. A-HE-100-4 (dump sanato) — sblocca tutto il resto.
3. A-HE-100-2 + A-MA-101-2 (match esaustivi, NON COMPILA).
4. A-HE-100-1 (tripwire col dump sanato) + A-HE-100-3 (differenziale BinaryAdd≡Binary(Add)).
5. Decisione misurata H-B2 sui residui on (A-MA-101-3).
6. Flip; poi KS-HE-101-3: ogni rotazione pin post-flip collauda il braccio OFF (RC-1 invertita, R4).

════════════════════════════════════════════
## FILE: verbali/team-evidenza-server.md

# Team «evidenza-server» — Klabnik (3) + Pedersen (6) — Concilio WP-101

## Nota di fatto (relatore)
Il morso Klabnik R1 (dente anti-putenv: chunk del `{main}` agganciato per
substring, entrambi i bracci vacui) è stato CONFERMATO A MACCHINA e RIPARATO
in chiusura di S-99: match sull'HEADER del chunk; i due bracci ri-passano sul
bersaglio giusto. Commit in main. A-KL-101-1 è quindi ESEGUITO; resta aperta
la parte «assert che il chunk non sia il {main}» come dente permanente.

## Convergenze
1. **«Collaudato: sì» è più largo dell'evidenza, per entrambe le sedie.**
   Klabnik R2 e Pedersen R1 dicono la stessa cosa da due lati: la gamba
   server poggia su DUE richieste sequenziali identiche (sentinella), mentre
   option 413 + restapi 3508 girano via phpr CLI — collaudano l'EMISSIONE,
   non il lifecycle. Riconciliazione: il registro GRADUA per gamba
   (A-PE-101-1: emissione-CLI / server-smoke / server-N-req /
   server-HTTP-suite). «Server collaudato largo» = N≥16 richieste, payload
   interleaved diversi, workers>1, almeno una restapi-shaped via HTTP
   (A-PE-101-3). Nessuna cifra WP attribuibile a un pin senza gamba
   esplicita (KS-PE-101-2).
2. **Il server non ha MAI servito flag-on: gate bloccante prima del flip.**
   KS-KL-101-1 e KS-PE-101-1 coincidono: flip VOID finché sentinella +
   option + restapi non passano col registro ACCESO, e la parità server nei
   DUE modi non passa sulla STESSA rotazione. Serve il launcher bimodale
   (A-KL-101-6): la lista chiusa `env -i` oggi non sa esprimere i due modi.
3. **Il flip inverte la semantica di «flag assente» e i denti vanno
   riscritti PRIMA.** Klabnik R7(i) e Pedersen R2 convergono: opt-out con
   grafia definita e pinnata (`PHPR_REG_LOWER=0`), bracci anti-putenv
   riscritti nel modo nuovo (assente⇒on, =0⇒off, putenv impotente in TUTTE
   le direzioni), commento-garanzia dello script di parità aggiornato
   (A-PE-101-4 ⊇ A-KL-101-5). Chi lo chiede: entrambi; ordine: opt-out →
   bracci → script → solo poi flip.
4. **Strumenti che quadrano su sé stessi**: assert conteggi↔nomi
   (A-KL-101-3, KS-KL-101-3) come precondizione di ogni gate citato al flip.

## Conflitti
Nessun conflitto sostanziale; una differenza d'enfasi: Klabnik vincola il
flip anche alla definizione operativa di A-KL-100-2, Pedersen no. Il team la
adotta (è il solo giudice dell'identità off↔on): **diff normalizzato
per-test dei dump/output off↔on, criterio pre-registrato = zero differenze**
(A-KL-101-4); il cronometro non giudica mai l'identità (KS-KL-101-2).
Secondo scarto: Pedersen aggiunge identità di ricetta (`--build-info`,
A-PE-101-5) che Klabnik non tratta — adottata senza obiezione.

## Priorità per l'ordine S-100
1. Grafia opt-out definita e pinnata nel funnel (pre-flip, bloccante).
2. Riscrittura bracci anti-putenv + funnel/M5 per il mondo post-flip
   (A-PE-101-4; sopra il fix header già in main).
3. Launcher parità bimodale + collaudo server flag-ON: sentinella estesa
   (N≥16, interleaved, workers>1, restapi via HTTP) + option + restapi.
4. A-KL-101-4: diff per-test off↔on, zero differenze, pre-registrato.
5. Registro pin: campo `collaudato:` graduato per gamba + `--build-info`.

════════════════════════════════════════════
## FILE: verbali/team-coda-stack.md

# Team coda-stack (Bak 5 · Stogov 8) — Concilio WP-101

Tema: coda H-B2 sul percorso pila (Sub/Mul/cmp int-int); criteri, perimetri, ordine S-100.

## Convergenze

1. **Criterio 0,7 ns non aggiudicabile — le due sedie chiudono la stessa porta da due lati.**
   Bak (R2, KS-BA-101-1): un criterio SOTTO il pavimento dichiarato della sonda (1,0 ns)
   non può né mordere né assolvere; ogni sonda futura pubblica BANDA + pavimento, e se
   criterio < pavimento la sonda è VOID e il criterio si rialza al pavimento — quindi
   D_registro ≥ 1,0 ns/occ finché la sonda non migliora. Stogov (KS-ST-101-3) tiene chiuso
   il rollout registro salvo MISURA che superi il criterio: nessun argomento statico lo
   riapre. Regola composta: soglia = max(0,7; pavimento sonda), scritta nel .out.
2. **La decomposizione va pubblicata come BANDA hit-only, non come quanto fisico.**
   Bak (R1, A-BA-101-1): 57/43 è uno split dipendente dall'ordine di rimozione; banda
   call/marshalling 52–62% col termine d'interazione NOMINATO; INT2 solo se decisionale.
   Stogov ri-perimetra il controfattuale (A-ST-101-1): banda valida solo guard-always-hit,
   rimovibile per-FORMA (BinarySC/SCDst pagano `consts[cidx].to_zval()` sul hit — non ~0
   per ZStr, refcount bump), conclusione D≈0 ristretta ad Add int-int. Le due restrizioni
   si sommano: «banda 52–62%, hit-only, per-forma» è l'enunciato da mettere in NEXT.
3. **Census delle frequenze = precondizione della scelta dell'occorrenza.** Bak
   (R5, A-BA-101-3, KS-BA-101-2): census sull'emissione POST-FLIP (frequenze su emissione
   in pensione = void), forme fuse CmpJmp/CmpJmpConst (run.rs:982-1004) contate A PARTE
   — sono il consumatore più caldo del plumbing; atteso scalato per frequenza PRIMA di
   scegliere; se atteso < risoluzione della coppia, l'occorrenza cade a tavolino.
4. **cmp non è fungibile a Sub.** Stogov R2/A-ST-101-2 (perimetro: __toString/ordine dei
   side effect, smart_streq «10»=="1e1", NaN/-0.0 nelle forme swapped, segno di Spaceship
   sotto swap, const-lhs; dichiarato PRIMA del controfattuale + una fixture per classe,
   pena VOID per KS-ST-101-2) e Bak R5b (le forme fuse vanno censite come classe propria)
   sono lo stesso vincolo visto da semantica e da microarchitettura.

## Conflitti

Nessun conflitto sostanziale. Una tensione d'ordine: per Bak il primo atto empirico di
S-100 include il profilo a campioni co-equale (A-BA-101-2); per Stogov nulla si promuove
finché le sette trappole AssignOp (A-ST-99-3) non sono gate PER NOME (R1, A-ST-101-3,
KS-ST-101-1: flip VIETATO senza fixture byte-identiche nei due modi). Compatibili: gate
prima del flip, profilo dentro la prima misura H-C.

## Priorità per l'ordine S-100

1. Sette trappole AssignOp PER NOME nel punto 1 come gate della promozione (bloccante,
   non backlog) — KS-ST-101-1.
2. Criterio D_registro rialzato al pavimento della sonda; banda+pavimento obbligatori
   (KS-BA-101-1) prima di ogni riapertura registro.
3. Census post-flip per-op con CmpJmp* a parte; atteso×frequenza decide l'occorrenza,
   soglia di caduta a tavolino (KS-BA-101-2).
4. Se il census indica cmp: perimetro compare SCRITTO prima del controfattuale + fixture
   per classe (KS-ST-101-2).
5. Ri-enunciato in NEXT: banda 52–62% hit-only per-forma; profilo a campioni co-equale
   alla tavola conteggio×costo nella prima misura H-C (A-BA-101-2); enumerazione classi
   flag-on (A-ST-101-4).

════════════════════════════════════════════
## FILE: verbali/team-misura-attribuzione.md

# Team «misura-attribuzione» — Concilio WP-101
Relatore: team Leijen (7) + Gregg (9). Fonti: verbale-7-leijen.md, verbale-9-gregg.md.

## Convergenze

1. **(a) Correzione dei rapporti peak — le DUE gambe, sempre.** Entrambi hanno verificato i raw (pair94/pair99-ratios): la frase «i rapporti peak si muovono per la gamba oracle» è vera sul FULL (phpr −0,45%, oracle +12,1%) ma INCOMPLETA sul MEDIA: phpr 1.170,8→1.202,7 MB (**+2,7% = +31,9 MB**, non piatta), oracle +28,7%. Correzione da scrivere in GAP_TREND e REPORT_GAP: sanare la riga media-peak con entrambe le gambe (A-LE-101-4) e, in forma, pubblicare la **tabella 2×2×2 delle gambe raw** (motore × workload × sera, A-GR-101-1) ogni volta che un movimento di rapporto viene attribuito a una gamba. «Piccolo» ≠ «piatto»: +31,9 MB senza banda è NON MISURATO.

2. **(b) L'anomalia oracle (+28,7% media, +12,1% full) va NOMINATA prima di riusare il metro.** Convergenza piena Leijen-R2 / Gregg-R2+A-GR-101-2: doppio run oracle media adiacente (R≥2, stessa sera) per misurare lo spread run-to-run; cause candidate da discriminare: varianza intrinseca del peak oracle, brew bump/ambiente, cache/ASLR/allocator. Fino ad allora i rapporti peak si pubblicano **come banda**, e KS-GR-101-2 vale: gamba oracle che si muove >10% senza causa nominata ⇒ rapporti non confrontabili cross-sessione.

3. **(c) Gate peak del flip = phpr-off vs phpr-on stessa-sera, MAI vs oracle** (A-LE-101-1), **con bande di accettazione pre-registrate PRIMA del run** (A-GR-101-3: full CPU e peak entro spread WP-94/99 lato phpr). I due kill-switch si compongono: senza banda pre-registrata il verdetto è VOID (KS-GR-101-1); sopra lo spread bandato la promozione si FERMA finché la crescita non è attribuita per owner (KS-LE-101-1).

4. **(e) Shape nuove (punto 4, Sub/cmp):** ogni variante Op porta nel commit N_OPS aggiornato (gate ≤255, const-assert) + `size_of::<Op>()` invariato dichiarato (A-LE-101-3); se lo stride cresce, la variante è RESPINTA qualunque sia il suo D (KS-LE-101-2). Nessuna obiezione di Gregg.

## Conflitti

Nessun conflitto sostanziale; due differenze di enfasi. (d) **Strumento H-C**: Gregg chiede di nominare nel programma lo strumento della gamba oracle (census opcode o profiler) co-equale alla tavola arith-decomposition lato phpr (A-GR-101-4); Leijen non copre H-C ma aggiunge l'asse mancante del flip: **sonda taglia-unità off vs on** (A-LE-101-2), perché la coppia peak da sola può mascherare un calo unit-cache con crescita runtime. Le due sonde sono complementari, non alternative. Su R3-Gregg (cucitura 1,1% tra blocchi «stessa finestra»): solo obbligo di dichiarazione, nessun gate.

## Priorità per l'ordine S-100

1. Sanare i file di rotazione: riga media-peak a due gambe + tabella gambe raw (A-LE-101-4, A-GR-101-1).
2. Spread oracle media R≥2 e nome all'anomalia; rapporti peak come banda fino ad allora (A-GR-101-2).
3. Bande di accettazione del flip PRE-REGISTRATE, gate phpr-off/on stessa-sera (A-GR-101-3, A-LE-101-1, KS-GR-101-1, KS-LE-101-1).
4. Sonda taglia-unità off/on al collaudo flag-on (A-LE-101-2).
5. Punto 4: vincoli N_OPS + size_of::<Op>() nel commit (A-LE-101-3, KS-LE-101-2).
6. H-C: nominare lo strumento lato oracle nel programma (A-GR-101-4).

════════════════════════════════════════════
## FILE: verbali/SYNTHESIS.md

# Concilio WP-101 — SINTESI DI CONVERGENZA (su S-99.0 e programma S-100)

## §FONDAMENTALI (prima di tutto, regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: il più alto da molte
sessioni — Gregg (mandato inverso) dà PASS. Nuovo per NOME: collaudo
WordPress full+media CHIUSO PER NOME sull'emissione H-A2+H-B2 (debito
regola-n.2 SALDATO); sei rapporti freschi sui DUE motori nella stessa
finestra (gamba oracle risanata; H-C 13,8 e H-D 8,6 rianimate); D
decomposto con una build di misura (call/marshalling vs traffico Vec);
predizione pre-registrata del percorso registro (rollout Add lì = a
tavolino); pin phpr E php-server COLLAUDATI (era: due rotazioni server
senza un solo collaudo, e il pin dichiarato era INCOLLAUDABILE — mancava
la feature axum-server); sigillo eager + dente anti-putenv.

**(b) Contatore sessioni-senza-misura**: full/media = **WP-99 = QUESTA
sessione (0)** — azzerato dopo 4 sessioni. Sei categorie: fresche di oggi
sui due motori.

**(c) Rischio d'oggetto più trascurato**: la gamba ORACLE del peak media
è salita del **+28,7% senza un nome** (R=1 per lato) — finché non è
spiegata con spread R≥2, ogni rapporto peak è una banda, non un gate. E
il CONTRATTO del flag è scoperto solo ora: `enabled()` è presence-based,
**`PHPR_REG_LOWER=0` ACCENDE il pass** — la bozza del flip poggiava su un
contratto che non esiste.

## Verdetti di fase 1 (9/9: nessun MI OPPONGO al lavoro fatto; la BOZZA
## §S-100 è refutata nei punti 1-2 così com'era scritta)

Verbali VINCOLANTI in `verbali/verbale-*.md`; note di team in
`verbali/team-*.md`. Refutazioni capitali per convergenza:

1. **Il contratto di modo non esiste** (5 sedie: Hoare, Matsakis,
   Hejlsberg, Pedersen, Klabnik): `enabled()` fa `is_some()` ⇒ `=0`
   accende; il flip a default esige un opt-out VALUE-PARSED nominato
   PRIMA di ogni riga, con ri-derivazione dei bracci anti-putenv, M5,
   launcher e funnel per il mondo post-flip (KS-MA-101-1, A-HO-101-2,
   A-HE-101-1, A-PE-101-4, A-KL-101-5).
2. **La bit-identità non si prova col cronometro** (Hoare, Klabnik,
   Hejlsberg): «5,43→5,44 ⇒ emissione bit-identica» è inferenza invalida;
   ogni claim d'identità d'emissione = SOLO diff di dump/output
   (KS-HO-101-2, KS-KL-101-2, KS-HE-101-4).
3. **Il server non è MAI girato flag-on e «collaudato: sì» era più largo
   dell'evidenza** (Klabnik, Pedersen): la gamba server = sentinella da 2
   richieste; il registro pin passa a `collaudato:` GRADUATO PER GAMBA
   (A-PE-101-1) + sentinella estesa (N≥16, interleaved, workers>1,
   A-PE-101-3); VIETATO il flip senza parità server flag-on nei DUE modi
   (KS-KL-101-1, KS-PE-101-1).
4. **Le cifre della pre-misura sono BANDE, non tariffe** (Matsakis, Bak,
   Stogov, Hoare): 57/43 → banda 52–62% (C0 è cross-tree: serve C0'
   same-tree per ereditarla, A-MA-101-1/KS-MA-101-2); la banda [0, 0,5]
   del registro è hit-only e per-forma (BinarySC paga `to_zval` sul hit,
   A-ST-101-1); il criterio 0,7 ns delle occorrenze future siede SOTTO il
   pavimento della sonda (1,0 ns): inaggiudicabile — soglia ≥ pavimento
   (KS-BA-101-1).
5. **Il flip RITIRA H-B2 sui siti stack non fusi** (Matsakis R2):
   l'emissione on non ha BinaryAdd; dove le finestre non coprono, il flip
   perde la specializzazione — la sorte si decide CON MISURA pre-flip
   (A-MA-101-3, arbitrato team flip-contratto: precondizione d'ordine).
6. **Morsi confermati a macchina e SALDATI in chiusura di S-99**: il
   dente anti-putenv giudicava il chunk del `{main}` (path incluso come
   COSTANTE nel dump — Klabnik R1; riparato: match sull'HEADER, bracci
   ri-verdi); la riga media-peak diceva «si muove l'oracle» ma la gamba
   phpr media è **+2,73% (+31,9 MB), NON piatta** (Leijen; REPORT_GAP_99
   e GAP_TREND sanati con la tabella delle gambe raw, A-LE-101-4 ✓,
   A-GR-101-1 ✓).

## Ordine DEFINITIVO S-100 (regola di ammissione applicata; le
## precondizioni del flip BLOCCANO l'oggetto promozione, quindi entrano)

1. **Contratto di modo** (primo atto, prima di ogni riga del flip):
   grafia dell'opt-out value-parsed nominata e documentata; bracci
   anti-putenv/M5/funnel ri-derivati per il mondo post-flip; launcher
   parità BIMODALE (A-KL-101-6).
2. **Strumenti sanati, in quest'ordine** (team flip-contratto):
   sanatoria dump/`lowered()` (A-HE-100-4) in TESTA; match ESAUSTIVI su
   `visit_addrs` e `bin_op_of` (variante nuova ⇒ NON COMPILA,
   KS-HE-101-2/A-MA-101-2); funnel-probe oltre `{main}` (A-HE-101-4);
   POI la definizione operativa del diff riga-per-riga (A-KL-101-4) +
   assert conteggi↔nomi nei gate (A-KL-101-3).
3. **Sette trappole AssignOp PER NOME** (A-ST-101-3; KS-ST-101-1 blocca
   il flip senza di esse) + **decisione H-B2-sotto-flip con misura**
   (A-MA-101-3).
4. **Collaudo flag-on completo PRE-flip**: diff per-test off↔on dei
   corpus (zero differenze attese, A-KL-101-4); parità server flag-on
   (launcher bimodale + sentinella estesa); spread oracle peak R≥2 +
   nome all'anomalia +28,7% (A-GR-101-2); coppia WP full+media + peak IN
   MODO on con BANDE PRE-REGISTRATE (A-GR-101-3; KS-GR-101-1: senza
   banda = fotografia, non gate; gate peak = phpr-off vs phpr-on
   stessa-sera, KS-LE-101-1).
5. **Flip del default** + rotazione pin con braccio OFF collaudato a
   ogni rotazione (KS-HE-101-3; campo `modo:`/gambe nel registro,
   A-PE-101-1).
6. Se il timebox regge: **H-C prima misura** (conteggio×costo + profilo
   a campioni CO-EQUALE, A-BA-101-2; strumento lato oracle NOMINATO:
   census opcache, A-GR-101-4).

**BACKLOG per NOME** (non slot di sessione): A-HO-101-1 (sigillo di
tipo), A-HO-101-4 + A-PE-101-5 (`--build-info`: identità pin oltre
l'hash churnante), A-PE-101-2 (census env PHPR_* lazy), A-MA-101-1 (C0'
same-tree), A-BA-101-3 (census post-flip, CmpJmp* a parte), A-LE-101-2
(sonda taglia-unità off/on), A-LE-101-3 (N_OPS const-assert +
`size_of::<Op>()` invariato, KS-LE-101-2), A-ST-101-2 (perimetro compare
PRIMA del controfattuale cmp, KS-ST-101-2), A-ST-101-4 (mappa divergenze
solo-flag-on), A-HE-101-3 (controllo positivo chiave unit-cache),
A-GR-101-4 già in ordine al punto 6. KS-ST-101-3: il registro resta
chiuso salvo misura ≥ pavimento.

L'ordine NON è solo apparato: i punti 3-6 sono misure e collaudi
dell'oggetto; i punti 1-2 sono le precondizioni che il flip esige per
non essere VOID (5 sedie convergenti).

## Conflitti registrati

- team flip-contratto: unit-cache key (Hoare: fidarsi del fp; Hejlsberg:
  mai collaudata) → RISOLTO con controllo positivo (A-HE-101-3, backlog).
- team flip-contratto: H-B2-sotto-flip (Matsakis: estendere; altri:
  misurare la perdita) → precondizione d'ordine CON misura, non veto.
- team coda-stack: ordine AssignOp-vs-profilo → compatibili (trappole
  pre-flip; profilo dentro H-C).
- team evidenza-server e team misura: nessun conflitto, solo enfasi.

