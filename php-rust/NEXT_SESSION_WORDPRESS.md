# NEXT_SESSION_WORDPRESS.md — S-95.0: IL CONTATORE HA DETTO SÌ → WP-96(sessione)

**Ultima sessione**: S-95.0 (2026-08-04, pomeriggio) — **A-ZV2 F1+F2
ESEGUITE, entrambe in SOLA MISURA**. F1: analisi di ultimo uso per slot
(`vm/liveness.rs`, dietro `zval-census`), contatore `would_take` sul media
group strumentato — la regola a tre bande di design95-liveness.md §P1 esce
in **banda ALTA su entrambi gli estremi** (cifre e derivazioni in
`wp95-harness/zvalcensus-f1.out`). F2: perimetro conservativo (rinunce per
funzione/slot/regione) — la prudenza taglia molto meno del 40% → **P2
SODDISFATTA**; il nucleo stringhe da solo sta in banda MEDIA
(`wp95-harness/zvalcensus-f2.out`). Contatori F1 riprodotti identici nel
run F2 (riproduzione su UNA coppia, N=1 — declassato da A-PP-97-5).
Binario di parità INVARIATO a ogni passo. Dettaglio:
`sessions/WP_SESSION_95.md`.

**⏱ FONDAMENTALI (regola utente 2026-08-03, aggiornare a OGNI rotazione)**:
ultima misura full/media = **WP-94 (1 sessione fa)** · ultima campagna
sull'oggetto = m90 in WP-90 (5 sessioni fa). S-95.0 non ha cronometrato:
ha contato il MECCANISMO della leva CPU scelta (conteggi esatti, non
campioni) e la decisione di prosecuzione è DERIVATA, non voluta. Il
cronometro torna in F4 (coppia stessa-sera, obbligatoria nel §WP-96).

## Stato gate

- **phpr (CLI, parità release)**: **d5ce86e3342f3926 INVARIATO** (nessuna
  ricompilazione di parità in S-95.0; le build strumentate vivono in
  `phpr-census-target/`, sha in banda nei `.identity` dei run). Corpus
  Zend per NOME 1418 + refl 290 (non rimisurato: nessun cambio al binario).
- **php-server**: **f8f4295a1dcdb627** (invariato, non toccato in S-95.0).
  ⚠️ Il pin storico d45b57843eeb1375 resta NON riproducibile — voce APERTA.
- **Gate cifre v3+A1**: `--all` **PASS a HEAD** (verificato dopo OGNI
  commit di S-95.0; budget corpus alzato con delibera due volte, nello
  stesso commit dei nuovi raw — mai un PASS pre-commit letto come
  post-commit). ⚠️⚠️ **Resta APERTO il canale env di git** (Concilio
  WP-96/Klabnik): la cura è COSTRUIRE l'ambiente (`env -i` + allowlist
  chiusa, A-SK-93..97, denti T27-T30) — **prima voce d'apparato di
  S-96.0, NON eseguita in S-95.0** (dichiarato: non bloccava l'oggetto,
  i PASS di sessione sono nati in ambiente pulito).
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).

## Permanent Binding Rules (da S-94.0, invariate)

**Un privilegio che vale per il processo non vale per la sua discendenza**
(sanificare l'AMBIENTE CONSEGNATO, non la shell). **Un dente che smette di
mordere non lo annuncia** (pretendere l'rc ESATTO). **Un predicato non deve
dipendere da ciò che esso stesso introduce.** **Un confronto identico non è
valido se entrambi i lati stanno fallendo.** **Il rc del runner non è il
giudice di una coppia.**

## ⚖️ Concilio WP-97 ESEGUITO (2026-08-04, verbali VINCOLANTI): `wp97-harness/COUNCIL_WP97_REVIEWS.md`

9 sedie, protocollo due fasi (3 team: engine, misura, catena), NESSUNA
benedizione. **Sei refutazioni capitali**, tre già APPLICATE in chiusura:
(1) Hoare: `movable_safe` INSOUND per emissione (def sottratta anche sul
contributo dell'arco exc — un catch può vedere `Undef`); bande F1/F2 salve,
**F3 bloccata finché transfer corretto e conteggi rifatti**; (2) Stogov: in
Zend i CV non si consumano MAI e la morte anticipata è osservabile anche
senza `__destruct` → F3 fedele = **move SOLO Str, banda MEDIA, P3
ri-derivata**; (3) Bak: «non aggiunge opcode al percorso caldo» è FALSO —
`TakeSlot` è un braccio nuovo, tetto WP-39..44; (4) Gregg: le righe
`guadagno_*` erano VERDICT ma il canale è SCREEN [APPLICATO: grade-per-campo
nei raw]; (5) Pedersen: header del raw F2 con HEAD nato dopo l'avvio del run
[APPLICATO: provenienza trascritta dall'identity; determinismo declassato a
N=1]; (6) Hejlsberg: F3 col riuso dell'analisi lazy/pointer-key = corruzione
semantica → **analisi nel COMPILATORE, identità strutturale**. Sintesi
§FONDAMENTALI + ordine emendato in `wp97-harness/verbali/SYNTHESIS.md`.

## §WP-96(sessione) — F3 EMENDATA dal Concilio WP-97: prima la soundness, poi il perimetro, poi l'opcode

**P0**: pre-flight standard + `--all` PASS a HEAD + pin phpr
d5ce86e3342f3926 invariato + **apparato A-SK-93..97 SUBITO in timebox**
(mezza sessione MASSIMO; ora è precondizione del grado di parità di F3 —
KS-SK-97-1: senza ambiente COSTRUITO i PASS futuri non sono verdict-grade).

### L'OGGETTO: A-ZV2 verso F3, nell'ORDINE del Concilio WP-97

Contratto in `wp95-harness/design95-liveness.md` + ordine emendato in
`wp97-harness/verbali/SYNTHESIS.md` (vincolante). In sequenza:

1. **Fix di soundness PRIMA di tutto** (A-TH-97-1): la def NON va sottratta
   sul contributo dell'arco eccezionale; match ESAUSTIVI senza wildcard in
   `effect()`/`renounce()` (A-TH-97-2 ≡ A-SK-97-2); varianti mancanti:
   `NewAnonDeferred` (A-SK-97-1), `CallBuiltinRefCell` + `debug_zval_refcount`
   (A-DS-97-5 ≡ A-MS-97-5); contatore `WOULD_TAKE_SAFE_REF` (A-MS-97-1).
   Poi **RICONTEGGIO F1/F2** (run media strumentato): se P2 scende sotto la
   soglia del design, stop e confronto piano B (KS-TH-97-3).
2. **Perimetro: whitelist Str-first** (A-MS-97-2 ≡ A-DS-97-1 — mai oggetti
   né array). P3 RI-DERIVATA sul nucleo (banda attesa MEDIA) ⇒ per la regola
   a tre bande: **confronto ESPLICITO col piano B** (A-TH-97-3), al NETTO
   del corpo caldo nuovo (tetto A-LB-97-1: Δ corpi caldi ≤0 o compensato,
   taglia `nm -S` predetta prima).
3. **Solo se la strada lunga vince il confronto**: `TakeSlot` con emissione
   SOLO compile-time (A-TH-97-5 ≡ A-AH-97-1; identità strutturale, mai
   puntatori, A-AH-97-3; assert sulla taglia di `Op` a 48 byte nello stesso
   commit, A-AH-97-4), contratto Undef+warning (A-DS-97-2), `gc_note` sul
   valore preso (A-TH-97-4 ≡ A-MS-97-3), trappole promosse a TEST COMMITTATI
   (A-SK-97-3 ≡ A-PP-97-4: `$a .= $a`, distruttore che osserva l'ordine,
   generatore sospeso, `compact()` dopo l'ultimo uso apparente, `use (&$x)`),
   gate di parità COMPLETI nello stesso commit (corpus 1418 + refl 290 +
   ORM + hk + battery61).
4. **F4** (se il tempo regge): coppia oracle-vs-phpr stessa sera con oracle
   RIMISURATO; census su BINARIO SEPARATO; controllo positivo a TRE
   contatori (A-LB-97-2: takes + fallback = safe predetto); coppia A/A con
   tetto spread ex-ante (A-BG-97-3); sanity ns/evento (A-BG-97-2);
   predizione footprint FIRMATA (A-DL-97-1; il peak a R=1 resta SCREEN,
   KS-DL-97-2); suite per NOME (A-PP-97-3).

### Dopo A-ZV2, per NOME (non «più avanti»)

Denominatore omogeneo in GAP_TREND (KS-BG-96-3) · leva arene per-file del
preludio con α RI-DERIVATO (Leijen) · probe slope v2 fuso · attribuzione
dello slope · il pin php-server che non torna.

### BACKLOG PER NOME (invariato da WP-95, più le voci nuove)

A-AH-78/79 · A-MS-65/66 · A-DS-96-1/2/3 (registry wrapper) · A-PP-83
(battery61 senza reset fra le gambe) · A-SK-92-PROBE · A-AH-70/74/75 ·
A-AH-73 · audit A-BG-72 · debito WP-94 non-A (ancoraggi campo, perimetro
root, sigilli E1→E3, checker LSP D1→D4) · residui A-DS51 fasi 2-3 ·
`stream_get_wrappers` incompleto · **gc_note_frame bitmask per-funzione
(Stogov §3, consulenza S-95.0)** · **hash cachato in PhpStr (Stogov §4)**.

### Criteri di CHIUSURA del fronte Axum/php-server (invariati da WP-94)

1. Slope attribuito per NOME — PARZIALE. 2. Leva per-file eseguibile.
3. Parità + ricevuta pin — APERTA (pin php-server). 4. Apparato CONGELATO
fuori quota. 5. Batteria riproducibile — SODDISFATTO con riserva
(KS-DS-96-3: predicato positivo + reset fra le gambe).

**NON riproporre**: tutti i NON-riproporre WP-83..95 restano; in più —
**«un PASS del gate letto PRIMA del commit vale per lo stato DOPO»** (il
corpus si legge da HEAD: la sequenza è commit locale → morso → budget nello
stesso commit → push a PASS); **«spostare un valore da uno slot che a
runtime regge un Ref»** (si de-referenzia, mai si sposta); «leggere un
miglioramento in un rapporto»; «una soglia di rinuncia non derivata»;
«ottimizzare il dispatch» (tetto calcolato, consulenze Bak/Stogov);
«run_loop non ci sta in i-cache» (refutato per misura).

---
**Chiusura**: 2026-08-04. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
