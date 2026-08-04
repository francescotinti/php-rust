# SINTESI DI CONVERGENZA — Concilio WP-97 (su report S-95.0 + programma WP-96)

## §FONDAMENTALI (in testa per regola utente 2026-08-03)

(a) **Avanzamento dell'OGGETTO in S-95.0**: fatti NUOVI e falsificabili sul
meccanismo di phpr — la frazione di letture di slot che sono ultimi usi, il
costo del perimetro conservativo, la scoperta che uno slot può reggere un
`Ref` a runtime invisibile alla rinuncia statica (cifre nei raw
`wp95-harness/zvalcensus-f1.out`/`-f2.out`, contatori esatti). Gregg
(mandato inverso): «OGGETTO avanzato». Nessun cronometro in sessione: il
cronometro è F4.
(b) **Contatore sessioni-senza-misura**: ultima misura full/media = WP-94
(1 sessione fa) · ultima campagna sull'oggetto footprint = m90 (5 sessioni
fa). Il metro è fresco; la rotta corrente è CPU-VM.
(c) **Rischio d'oggetto più trascurato**: costruire F3 su un moltiplicatore
NON rimisurato — il valore del canale in §P1 viene da un profilo R=1: ogni
banda derivata è SCREEN finché F4 non la misura. Secondo rischio: l'apparato
env-git (A-SK-93..97) ora BLOCCA l'oggetto (KS-SK-97-1: senza ambiente
COSTRUITO i PASS di parità di F3 non sono verdict-grade) — entra nell'ordine
di WP-96 per la regola di ammissione, in timebox.

## Verdetti (9 sedie, NESSUNA benedizione)

Hoare CON EMENDAMENTI · Matsakis CON EMENDAMENTI · Klabnik PASS CON RISERVE
VINCOLANTI · Hejlsberg: F3 come scritto NON eseguibile, S-95.0 regge · Bak
PROCEDI CON EMENDAMENTI · Pedersen RESPINTO IN PARTE · Leijen NON REFUTATO,
prosecuzione CONDIZIONATA · Stogov REFUTA la scelta «banda ALTA» · Gregg
APPROVATO CON DECLASSAMENTO.

## Refutazioni capitali (riprodotte nei verbali, fonte vincolante)

1. **Hoare A-TH-97-1**: `movable_safe` è INSOUND per emissione — la def è
   sottratta anche sul contributo dell'arco eccezionale (un catch potrebbe
   vedere `Undef`). Bande F1/F2 dichiarate salve; F3 BLOCCATA finché il
   transfer non è corretto e i conteggi rifatti (KS-TH-97-2/3).
2. **Stogov**: in Zend i CV non si consumano MAI; la morte anticipata è
   osservabile anche senza `__destruct` (spl_object_id, WeakReference,
   risorse). L'F3 fedele = move SOLO Str ⇒ banda MEDIA, P3 ri-derivata.
3. **Bak**: «non aggiunge opcode al percorso caldo» è FALSO — `TakeSlot` è
   un braccio nuovo e paga il tetto WP-39..44; banda ALTA = SCREEN-grade.
4. **Gregg (grado)**: righe `guadagno_*` marchiate VERDICT ma il canale è
   SCREEN — il prodotto eredita il fattore più debole. [APPLICATO IN
   CHIUSURA: `grade-per-campo` nei due raw.]
5. **Pedersen (provenienza)**: header del raw F2 dichiarava un HEAD nato
   DOPO l'avvio del run (build da albero non committato). [APPLICATO IN
   CHIUSURA: header trascritto dall'identity, corrispondenza dichiarata-non-
   provata; «determinismo pieno» declassato a riproduzione N=1.]
6. **Hejlsberg**: F3 col riuso dell'analisi lazy/pointer-key sarebbe
   corruzione semantica — l'analisi va nel COMPILATORE, identità
   strutturale, mai puntatori; banda P3 lorda del corpo caldo nuovo.

## Ordine WP-96 emendato (recepito in NEXT_SESSION §WP-96)

1. Apparato A-SK-93..97 in timebox (ora precondizione del grado di parità).
2. Fix soundness (A-TH-97-1) + match esaustivi senza wildcard (A-TH-97-2 ≡
   A-SK-97-2) + varianti mancanti (A-SK-97-1 NewAnonDeferred, A-DS-97-5 /
   A-MS-97-5 CallBuiltinRefCell + debug_zval_refcount) + contatore
   WOULD_TAKE_SAFE_REF (A-MS-97-1) → RICONTEGGIO F1/F2; se P2 scende sotto
   la soglia, stop e confronto piano B (KS-TH-97-3).
3. Perimetro: whitelist Str-first (A-MS-97-2 ≡ A-DS-97-1); P3 RI-DERIVATA
   sul nucleo; banda attesa MEDIA ⇒ confronto ESPLICITO col piano B
   (A-TH-97-3) al NETTO del corpo caldo (tetto A-LB-97-1, nm -S predetto).
4. Se la strada lunga vince il confronto: emissione SOLO compile-time
   (A-TH-97-5 ≡ A-AH-97-1, identità strutturale A-AH-97-3, assert taglia Op
   A-AH-97-4), contratto Undef+warning (A-DS-97-2), gc_note sul valore
   preso (A-TH-97-4 ≡ A-MS-97-3), trappole come test COMMITTATI (A-SK-97-3
   ≡ A-PP-97-4), gate completi nello stesso commit.
5. F4: census su binario separato, controllo positivo a tre contatori
   (A-LB-97-2), coppia A/A con tetto spread ex-ante (A-BG-97-3), sanity
   ns/evento (A-BG-97-2), predizione footprint firmata (A-DL-97-1; peak
   R=1 resta SCREEN, KS-DL-97-2), suite per NOME (A-PP-97-3).

Conflitti registrati (non appianati): ammissibilità futura del perimetro
F2-intero (Matsakis sì-dopo, Stogov pre-refutato, Hoare via confronto §P1);
sufficienza del guard di tipo (Stogov sì, Hoare no); obbligo del peak in F4
(Leijen sì, altri cronometro-first). Fonte vincolante: i verbali individuali.
