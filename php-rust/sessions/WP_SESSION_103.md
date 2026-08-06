# WP_SESSION_103 — S-103: l'ordine WP-104 consumato intero — l'audit ribalta il verdetto meccanico, i prefissi H-C2 contati esatti, la cifra H-D dimezzata dal denominatore

**In una frase**: abbiamo rifatto con più rigore una misura di memoria che
sembrava «tutto a posto» ma era inaffidabile (ora sta girando la versione
buona), verificato anche via web che le correzioni recenti non cambiano
nessun comportamento visibile, contato con precisione dove il motore spreca
lavoro nelle proprietà e nelle chiamate (scoprendo che una stima precedente
era il doppio del reale), e dimostrato con un test che gli oggetti in
circolo attraverso un generatore non vengono mai liberati.

**Data**: 2026-08-06 (13:0x–16:3x). **Modello verificato all'apertura**:
Fable 5. **Ordine eseguito**: Concilio WP-104 §S-103 punti 1-5 TUTTI
(igiene 8/8 nella finestra d'attesa dell'A/B — timebox rispettato).
**Commit**: a4a53bb → 56a2174 (19 commit su main, tutti pushati).

## Ordine eseguito

| # | Esito |
|---|---|
| **1a · Verdetto A/B peak** | La riga meccanica del launcher diceva «RUMORE, voce chiusa» — **l'audit finestra WP-104 la RIBALTA: RUN VOID** (spread 90,38/99,44 MiB > tetto 51,96 = 1,5× fase-1; il launcher usava lo spread intra-run come banda). Δ mediane +51,95 in zona marginale ⇒ R≥7 comunque; 5/5 coppie stesso segno B>A = indizio di crescita reale; coppia 1 ~85 MiB sopra le altre su ENTRAMBI i bracci (warmup freddo). **Rerun R=7 con warmup pre-registrato IN VOLO** (daemonizzato 16:23, verdetto in `wp103-harness/peak-ab-out/ab-verdetto.out`, launcher emendato: banda fase-1 e tetto NEL verdetto). Audit archiviato: `peak-ab-audit-verdetto.out` |
| **1b · Collaudo pin server NUOVO** | Launcher EMENDATO (`s103-collaudo-server.sh`, KS-PE-104-1): braccio **warning-line cb2** (2 warning undef-prop, righe 36/41 = oracle AL CONFINE HTTP — il caso multi-riga discrimina il §3.13: riga della LETTURA, non del flush), **cbE errore-poi-successo** (marca pendente + fatal ⇒ la richiesta dopo è byte-id), **interleaving cross-fixture workers=2**, PIN_SRV_ATTESO da file fail-closed, DOC canonico. Pin **31aa7c2eef899cce** @ HEAD 37312e8 (ricetta piena) **GRADATO fails=0×2 + cross-mode byte-id**; stash `php-server-s103`. Riga NON-pin 49a91e4d a registro. ⚠️ il pin di metà sessione da5c2948 (gradato 15:32) è stato SOVRASCRITTO dalle build workspace prima dello stash — superseded, lezione a registro |
| **2 · Pacchetto ricevitore** | **Fixture 19a** (soglia ESATTA mid-arm: lo slot muore dentro `__get`, collector sul temp in self-cycle) e **19b** (base=1: `(new C)->x`) — attese PRIMA, **PASS al primo colpo byte-id oracle ×2 modi** (correzione cosmetica echo registrata). Tavola INV-RECV-1 EMENDATA (esito ristretto a slot-held + righe nuove arbitrate) + **marcatori stabili OBS-1..12 nei sorgenti**. **A-ST-104-4 con CORREZIONE DI ROTTA documentata**: la premessa «il chiamante scartoccia sempre» è REFUTATA dal codice (`gc_note` instrada i Ref via il predicato; le catture by-ref sono Ref legittimi) — l'assert nel braccio Ref avrebbe morso sentieri sani; atterrato invece sull'invariante vera («i Ref non si annidano») nel descend; il contratto wrapper-mai-root è già imposto PER TIPO ai sink |
| **3 · Prefissi H-C2 (sequenza vincolata)** | (i) **leva-nulla**: feature `null-lever`, semantica nulla PROVATA (dump-diff bimodale zero), __text shift reale −308 B ⇒ **banda-LAYOUT = 0,67 ns/iter**, rumore run-to-run ~3 ns/iter dominante; (ii) **drop-census** (`dcn!` DropS/DropC col predicato unico): attesa v2 pre-registrata (raffinata LEGGENDO i corpi: gli overwrite droppano) **CONFERMATA ESATTA — 11 DropS + 3 DropC/iter**, linearità 300:1; la stima «~11» combacia coi SOLI scalari; (iii) **`hc2-criterio.out`**: pavimento 4 ns/iter (sopra layout E rumore: coerente), banda [8,22] confermata dal contato, **`size_of::<Zval>()==16` pinnato A COMPILE-TIME**, fast-out solo via `is_gc_container`, Δ per sito×specie. La LEVA stessa non aperta (tempo): tutto pronto per S-104 |
| **4 · Cifra netta H-D** | Realloc DISAGGREGATO (`grealloc_note`) + istogramma size-class in memcensus (righe nuove, storica invariata). 🔵 **CORREZIONE DI DENOMINATORE**: calls.php fa **20M iterazioni**, non 10M — la cifra S-102 «2 alloc/chiamata» era GONFIATA 2×. Cifra vera: **1 alloc × 32,0 B ESATTI + 1 free per chiamata, realloc ≡ 0** (2 eventi totali = avvio), TUTTO nel bucket (16,32] B, linearità 199:1 alla quarta cifra. Anche i «10 gc_note/iter» S-102 ⇒ ~5/chiamata. Leva GATED sul tag per-sito (residuo≡0, SiteTag a S-104); UNA sola alloc da 32 B restringe gli indiziati (ret_cell Rc O args Vec, non entrambi). `hd-cifra-netta.out` |
| **5 · Igiene (8/8, nella finestra d'attesa)** | Regola **set-che-scende** SCRITTA (4 condizioni) · **hash fail-closed + mode-probe nei 3 fixture-gate** (VOID provato su assente E mismatch) · dente sottoprocesso emendato (**braccio `=0` discriminante + controllo positivo `Z::{prop-init}` + residuo `Binary(Add)` ==1 ESATTO**) · 🔵 **fixture generator-in-cycle MORDE** (oracle raccoglie, phpr no: dtor solo a shutdown — buco A-HO-103-2 PROVATO, terza deroga negata nei fatti, fixture rossa fuori dai gate) · 🔵 **censimento §3.11/§3.12 MISURATO** (§3.12 rititolata typed-LVALUE: azzera anche la prop diretta SENZA ref, 4/4 specie; §3.11 = tutto il canale RMW incluso ++/--) · §3.13 chiusa col claim ridimensionato (famiglia PropGet, 5/~435) · banda tra-sere: protocollo + **sera-2 stesso-pin** (2/3 punti; Δ entro ±0,3) · gh-status-sync (corpus 2650/1417 pubblicato) |
| **Gate cumulativi (pin chiusura f45a5d19)** | Batteria **1739/0** · fixture **13/13 + 5/5 + 2/2** (coi denti nuovi) · corpus **1417 per NOME ×2 modi** (`wp103-harness/corpus-gate/`) · server 31aa7c2e collaudo **fails=0×2** · micro sera-2 su d0b01362 stashato: **12,3 / 11,5 / 7,4 / 6,6 / 4,4 / 3,7** (spread ≤0,06). Il runtime S-103 è INTENZIONALMENTE nullo in parità (commenti, debug_assert, census-gated): nessuna leva spedita, i rapporti NON cambiano per costruzione — coppia WP non rieseguita (runtime parity-null + corpus/batteria/server ×2 verdi; il debito è NOMINATO, non nascosto) |

## 🔵 Scoperte

1. **Il verdetto meccanico può essere GIUSTO per la regola sbagliata**: il
   launcher confrontava |Δ| col proprio spread intra-run (99 MiB) — con
   quella banda QUALUNQUE Δ era «rumore». L'audit pre-registrato (banda
   fase-1) l'ha ribaltato in VOID.
2. **La cifra S-102 di calls aveva il denominatore doppio**: calls.php fa
   20M iter (l'`out=20000000` è la somma, non un indizio di 10M). La firma
   della cifra è passata dal RILEGGERE il sorgente del giudice.
3. **Il generator-in-cycle perde davvero**: ciclo mai raccolto, dtor a
   shutdown — ora c'è la fixture rossa che lo prova (e resta l'arbitro del
   fix).
4. **§3.12 non è dei ref**: Zend azzera il typed-LVALUE dopo OGNI AssignOp
   fallito, anche `$t->i += "abc"` senza ref (4/4 specie divergenti).
5. **La premessa di un emendamento di concilio può essere falsa nel
   codice**: l'assert Ref «il chiamante scartoccia sempre» avrebbe morso
   `gc_note` e le catture by-ref — verificare i CHIAMANTI prima di
   piazzare un assert.
6. **php -S canonicalizza il docroot via symlink, phpr no** (osservazione
   dal collaudo: /tmp vs /private/tmp nei path dei warning).

## ⭐ Lezioni

- ⭐⭐ **Una banda si pre-registra o non è una banda**: il verdetto
  meccanico che sceglie la propria banda dal run stesso si assolve da solo.
- ⭐⭐ **Il denominatore si legge dal sorgente del giudice, mai dalla
  memoria di sessione** (2× di errore sopravvissuto a un concilio).
- ⭐⭐ **Un assert si piazza dove il contratto vive**: prima si enumerano i
  chiamanti reali (Serena), poi si sceglie il punto — il tipo del sink può
  già imporre il contratto meglio dell'assert.
- ⭐ **Lo stash si fa NEL momento del grading**: il pin da5c2948 gradato e
  mai stashato è stato sovrascritto dalla build successiva.
- ⭐ **Gli output di collaudo restano fuori dal repo** (26 MB di srv.log
  committati per sbaglio e rimossi: la convenzione va guardata PRIMA del
  primo `git add -A`).
- ⭐ **L'attesa pre-registrata si può RAFFINARE prima del dinamico**
  (v1→v2 dei drop leggendo i corpi): è ancora pre-registrazione, e il
  dinamico l'ha confermata a zero scarti.

## Stato binari e processi

- **phpr pin chiusura: f45a5d199ab34132** @ HEAD 56a2174 (fa fede HEAD;
  hash churna col relink) — DEFAULT flag-ON; contiene marcatori OBS,
  debug_assert nested-Ref, dcn!/null-lever/memcensus H-D (tutto
  parity-null). Stash ADDITIVO `phpr-s103`. Batteria 1739/0 · fixture
  20/20 · corpus 1417×2 per NOME.
- **php-server pin: 31aa7c2eef899cce** @ HEAD 37312e8, ricetta piena,
  GRADATO minimo-emendato fails=0×2; stash `php-server-s103`.
- **A/B peak R=7 IN VOLO** (daemonizzato 16:23, warmup non misurato +
  7 coppie ABAB ≈ fine ~21:00): verdetto MECCANICO con banda fase-1 in
  `wp103-harness/peak-ab-out/ab-verdetto.out` + flag `ab.done`.
  **Lettura del verdetto (con audit finestra) = primo atto S-104.**
  Nessun'altra run pesante finché non chiude.
- MySQL wp8 su; uploads sotto guardia (backup 16:23). Harness di
  sessione: `wp103-harness/`.
