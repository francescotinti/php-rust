# NEXT_SESSION_WORDPRESS.md — S-95.0: IL CONTATORE HA DETTO SÌ → WP-96(sessione)

**Ultima sessione**: S-95.0 (2026-08-04, pomeriggio) — **A-ZV2 F1+F2
ESEGUITE, entrambe in SOLA MISURA**. F1: analisi di ultimo uso per slot
(`vm/liveness.rs`, dietro `zval-census`), contatore `would_take` sul media
group strumentato — la regola a tre bande di design95-liveness.md §P1 esce
in **banda ALTA su entrambi gli estremi** (cifre e derivazioni in
`wp95-harness/zvalcensus-f1.out`). F2: perimetro conservativo (rinunce per
funzione/slot/regione) — la prudenza taglia molto meno del 40% → **P2
SODDISFATTA**; il nucleo stringhe da solo sta in banda MEDIA
(`wp95-harness/zvalcensus-f2.out`). Determinismo pieno fra i due run.
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

## §WP-96(sessione) — F3: l'opcode `TakeSlot`, coi denti già pronti

**P0**: pre-flight standard + `--all` PASS a HEAD + pin phpr
d5ce86e3342f3926 invariato + **apparato A-SK-93..97 in timebox** (mezza
sessione MASSIMO, regola permanente; se sfora, si spedisce l'oggetto e
l'apparato torna in coda).

### L'OGGETTO: A-ZV2 fase F3 (+F4 nella stessa sessione se il tempo regge)

Contratto in `wp95-harness/design95-liveness.md` (§Le fasi, §P3 DERIVATA,
§P4). I numeri che giustificano l'ordine stanno nei due raw di S-95.0
(`zvalcensus-f1.out`, `zvalcensus-f2.out`); la P3 per F4 è la banda
derivata scritta in fondo a §P1 del design. Punti fermi per F3, decisi
dalle misure e dagli smoke di S-95.0:

1. **Il guard è a RUNTIME oltre che statico**: l'emissione usa
   `movable_safe` (F2), ma il handler di `TakeSlot` DEVE guardare il tipo
   della cella — un `Zval::Ref` si de-referenzia (fallback clone), mai si
   sposta (lo smoke ha mostrato che il lato interno di una closure by-ref
   sfugge alla rinuncia statica).
2. **La scelta del perimetro di tipo è una decisione di design da
   prendere A INIZIO F3**: perimetro F2 intero (banda ALTA, ma un take di
   un oggetto/array può ANTICIPARE un `__destruct` osservabile rispetto
   all'oracle Zend, che i CV non li consuma mai) vs nucleo stringhe
   (banda MEDIA, rischio distruttori ZERO per costruzione). Il conto per
   confrontare le due opzioni sta nei raw; la trappola distruttori è
   descritta in design95-liveness.md §Perché e §punto 10.
3. **Gate di parità COMPLETI nello stesso commit dell'opcode** (corpus
   1418 + refl 290 + ORM + hk + battery61) + i test delle trappole:
   `$a .= $a`, distruttore che osserva l'ordine, generatore sospeso,
   `compact()` dopo l'ultimo uso apparente, `use (&$x)` letto dopo il
   take apparente.
4. **F4 = coppia oracle-vs-phpr della stessa sera** con oracle RIMISURATO
   (mai denominatore congelato) + controllo positivo del meccanismo:
   `slot_reads_avoided` deve muoversi della quantità predetta dai
   contatori F2, o il Δ tempo viene da altro (Bak).

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
