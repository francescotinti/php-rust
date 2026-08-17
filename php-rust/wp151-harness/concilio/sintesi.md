# CONCILIO S-151 — SINTESI DI CONVERGENZA (i 9 verbali individuali restano la fonte VINCOLANTE; qui si cita per emendamento)

Verdetti: **9× CONCORDO CON EMENDAMENTI, 0 opposizioni** (Pedersen condizionato
ai suoi R2/R3 — recepiti sotto). Fase 2: tre note di team in `verbali/`.

## §FONDAMENTALI
- **Oggetto**: parità 1× — fronte ORM 7,10–7,15. La rotta Zval-first è
  **RATIFICATA CON EMENDAMENTI**. S-151 spedisce 0 leve: anomalia DICHIARATA,
  coperta dalla rotta utente 2026-08-17.
- **Sessioni-senza-misura**: 0 a ingresso (S-150 ha misurato). Il census A1 è
  esso stesso misura di conteggio; i TEMPI di suite si rifanno solo a pin nuovo.
- **Mandato inverso (Gregg)** — cosa sappiamo OGGI che ieri non sapevamo:
  (i) BT1 era un outlier ALGORITMICO pescato per NOME, non un fatto di
  memory-model ⇒ la pesca-outlier è la leva più economica e va RIPETUTA dentro
  A1 (teste: none.other 94,6M · class_exists 9,7M · __reflect_* 12,4M);
  (ii) TUTTE le cifre census s147–s149 sono STALE post-BT1 — incluso il tetto
  movimenti 1,27 s, declassato a IPOTESI direzionale non citabile nei criteri
  finché tranche-5 @ s150 non lo rifonda (governa Gregg R5);
  (iii) il gap ORM residuo non ha oggi NESSUN canale Object contato E
  prezzato ⇒ **A3 è oggi INDECIDIBILE: A1 esiste per questo**.
- **Rischio d'oggetto più trascurato**: il refcount MANUALE (un dec mancato =
  distruttore mai eseguito = bug semantico SILENZIOSO), non il layout.

## §RATIFICA — sequenza emendata
- **A1 (subito)**: census tranche-5 @ s150 RIFONDATO — spec ESATTA in
  `verbali/team-misura.md §Spec`: canali C1–C5 per TIPO (Object ≠ Str/Array),
  chiavi SIMBOLICHE mai file:riga (o A2 invalida A1), identità Σsiti==tot +
  overlap==0 STAMPATO + legge di conservazione per classe (violazione ⇒
  census NULLO), sonde-prezzo per canale (senza prezzo il canale non entra in
  banda), micro-mock del costo sostitutivo store-indicizzato, numeri Leijen
  N1–N6 (footprint, distribuzione #props pesata p50/p90/p99), testa hostcall
  per-NOME + scan outlier, istruzione scarto +3,2%, ricetta probe con env +
  path-length + stato fusione dichiarati (Gregg R7), gamba SERVER
  multi-richiesta col teardown prezzato (Pedersen R4).
- **A2**: perimetro **CHIRURGIA-FIRST** (convergenza cross-team unanime:
  SOLO i moduli della touch-map che A3 toccherà; host.rs e il resto della
  mappa Gemini = backlog non bloccante; ~3 sessioni invece di 4–6 ⇒ anomalia
  RIDOTTA — ratifica utente al prossimo contatto). Tranche = commit move-only
  revertibile, zero rinomini, same-crate VINCOLANTE, `#[inline]` preservati
  (nm-census: run.rs ha never×3/always×6 misurati), freddi→caldi, run_loop
  ULTIMO o MAI e NON spezzato (veto su exec/ops_* di Gemini §4.3). Gate per
  tranche = FASCIO (sostituto della byte-identità vietata): batteria rc=0 ·
  corpus 1412×2 **ZERO-FLIP per NOME (anche un PASS nuovo è rosso)** ·
  fixture bilaterali · micro R=5 solo-regressione soglia max(4; rumore;
  banda-layout) · disasm run_loop (size/bl) con delta DICHIARATO ·
  **census-eco: conteggi per canale IDENTICI pre/post** (Klabnik R1/Pedersen
  R5) · footprint peak vmmap (Leijen R4); `--timings` companion non-gate
  (la «build dimezzata» Gemini è NON FONDATA sotto CGU=1+fat-LTO).
- **A3**: **FUORI AGENDA finché A1 non produce i numeri.** Forma
  pre-registrata: store a bucket di puntatori STABILI + free-list alla
  `zend_objects_store` (arena contigua e bump-reset REFUTATE: Stogov R2,
  Leijen R1), store = CAMPO della Vm che muore con lei (Pedersen R1),
  handle LINEARE non-Copy/non-Clone con mint solo dello store, dup solo
  incref, morte solo `Vm::release(&mut Vm)` (Hoare R1), indice GENERAZIONALE
  con check ON in release (Hoare R2), weak=(id,gen) anti-ABA, id PHP-visibile
  con riuso alla Zend senza trapelare la generazione, coda decrementi
  thread-local drenata dallo sweep esistente (Matsakis: senza coda nel
  criterio, A3 irricevibile), shadow-mode Rc↔rc + live-count==0 a fine
  richiesta, divieto di doppia rappresentazione (Hoare R3). Perimetro per
  NOME prima della chirurgia: Resource FUORI (resta Rc), Zval::Ref e catture
  Closure DICHIARATI nel criterio (D1). Vittime nominate da riprovare:
  Drop PhpArray/L-RD1, invariante ricevitore WP-102, soglie osservatore 2+extra.
- **A4**: dente righe-.rs in **BATTERIA subito, prima della tranche 1**
  (unanime; la CI è specchio TARDIVO: backlog 93 commit ≈ 3 giorni):
  `loc_dente.rs` sul telaio di rczval_funnel_dente, misura `lines().count()`
  MAI pattern componibili (lezione auto-morso bea7ea3), nuovi ≤2.000,
  allowlist (path, cap, motivo) a cap ESATTI odierni, anti-slack
  cap−n≤200 ⇒ chi snellisce abbassa il cap nello stesso commit, cap solo in
  discesa, salita = diff dichiarato; **collaudo in NEGATIVO nell'atto di
  armamento, pena dente non armato + incidente** (Hoare R7).

## §DECISIONI DI PLENARIA (dissensi a registro, non appianati)
1. **Cadenza pin (C1-struttura)**: pin per SESSIONE con 2–3 tranche
   accorpate, coppia WP a OGNI pin ⇒ regola utente INTATTA, nessuna deroga
   necessaria (maggioranza 2-1 + team-semantica conv.7; Hoare per-tranche a
   registro). Il quesito-deroga C3 decade.
2. **A3.0 timing distruttori (C2-semantica)**: adottata (a) SWEEP-PRESERVING
   (maggioranza Stogov(a)+Pedersen R8+Matsakis coda; K-M4 armato: un
   dtor-order che cambia ferma A3); l'opzione (b) Zend-immediate resta agli
   atti come possibile atto di FEDELTÀ separato, commissionabile dall'utente.
3. **Soglia GO/NO-GO A3c (C2-misura + D2)**: si pre-registrano ENTRAMBE le
   formule (Gregg R3 banda-netta-mock ≥ 2× soglia s150-riderivata ≈ ≥0,5 s;
   Bak/struttura quota-gap su binario census) coi denominatori ARMONIZZATI
   nel criterio; GO solo se TUTTE soddisfatte (la più severa governa).
   Sotto soglia ⇒ A3c chiusa stile veto NaN-boxing; restano A3a/b e leve per NOME.
4. **Ratchet A4 (C2-struttura)**: cap ESATTI odierni (2-1; Hejlsberg +50 a
   registro come minoritaria).
5. **Cifre Gemini** (30–45%, −35%, 40%, «rischio zero», «build dimezzata»):
   MAI in un criterio; tensione tetto↔Gemini sciolta CONTRO Gemini — il
   movimento va ESCLUSO dalla somma pro-A3 (unanime); **Fase 5 registri:
   veto WP-44 CONFERMATO** (riapribile solo come «true 3-address zero-enum»
   in crate isolato, fuori da questa rotta).

## §QUESITI ALL'UTENTE (non bloccanti per A1)
- Ratifica perimetro A2 chirurgia-first (~3 sessioni vs 4–6 accettate).
- A3.0: conferma di (a) sweep-preserving o commissione di (b) fedeltà-Zend.

## §ORDINE residuo S-151 (dal concilio)
1. Pre-registrare `s151-criterio-census.md` (canali, identità, soglie
   armonizzate, ricetta probe) PRIMA di ogni run.
2. Armare il dente A4 con collaudo negativo+positivo, dichiarato in batteria.
3. Probe tranche-5 a sorgente s150 (COPIA census, mai il pin) + smoke a
   esito esatto; run ORM detached se la finestra lo consente, altrimenti
   apertura S-152.
4. Leva fedeltà §3.24: NON ammessa nel ritmo S-151 (nessuna sedia la
   ammette; il ritmo è saturo di A1/A4) — resta in coda per NOME.
