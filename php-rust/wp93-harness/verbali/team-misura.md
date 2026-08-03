# Team-misura — Concilio WP-93 (fase 2)

Relatore: team-misura. Sedie: 5 Bak, 6 Pedersen, 2 Matsakis. Fonti vincolanti: verbale-5-bak.md, verbale-6-pedersen.md, verbale-2-matsakis.md. Questo verbale riconcilia; NON supersede i verbali individuali.

## CONVERGENZE

**1. La coppia di Matsakis e la finestra di Pedersen sono lo STESSO emendamento, con un pezzo in più di Pedersen.** A-MS-57 (bracket: seconda lettura POST-dump, campo `outstanding_post=` in-band sulla riga mi_quiesce) e A-PP-66 (testimone a COPPIA: seconda riga POST-dump, stesso ckpt, campi pre/post) coincidono nella sostanza: chiudere la finestra load→dump con una rilettura. Divergono solo nella forma (campo sulla stessa riga vs seconda riga) e nella sufficienza: Matsakis ritiene pre==post==0 sufficiente sotto protocollo (client unico sequenziale esclude il transito completo intra-finestra); Pedersen dimostra che a livello MACCHINA non basta — una richiesta interamente contenuta nella finestra riporta outstanding a 0, e il monotono `REQUESTS` NON compila sotto mem-census (fetch_add r.944 è census-instrumentation-only). Quindi serve anche A-PP-67 (monotono di arrivo su `any(census-instrumentation, mem-census)`, campo `arr=`).

**Forma unificata deliberata (componibile in UNA riga testimone):** riga mi_quiesce unica, stesso ckpt, campi `outstanding_pre= outstanding_post= arr_pre= arr_post=`; dec OUTSTANDING → `Release`, load testimone → `Acquire` (A-MS-56, edge formale dec→load); pre==post==0 ∧ arr_pre==arr_post ⇒ finestra pulita. Un solo commit, tre emendamenti (A-MS-56/57 + A-PP-66/67) fusi. Il commento main.rs:262-265 va riformulato: «proof» solo con edge + bracket + arr.

**2. Stessa diagnosi sul claim.** Entrambe le sedie refutano l'estensione del testimone da ISTANTE a FINESTRA; entrambe confermano che la catena sottostante REGGE (teardown Vm/RetainSet completi prima del send; direzione fail-closed). A-PP-69 (free del `Vec<u8>` body lato axum fuori-testimone, dichiarato nel commento) accolto senza obiezioni.

## CONFLITTI

**1. Grado della refutazione (registrato, non riconciliato).** Pedersen la classifica CAPITALE («garanzia di finestra» falsa); Matsakis la classifica SOVRA-DICHIARAZIONE senza refutazione capitale. Il reperto è identico; differisce il grado. Il team registra: capitale per il claim-come-finestra (Pedersen vincola), PASS per la soundness del cfg-split e della catena (Matsakis vincola). Nessuna contraddizione di merito.

**2. Sufficienza del bracket.** KS-MS-93-1 (bracket pre/post) è SUSSUNTO da KS-PP-93-1 (coppia + arr uguali): il dente più severo vince — Δ di fase verdict-grade SOLO con coppia completa incluso `arr=`; testimone a riga singola O bracket senza monotono ⇒ ADVISORY. Matsakis non obietta: il suo Q2 già ammette che il testimone non è machine-proof contro traffico estraneo.

**3. Bak ↔ .out (non intra-team):** la dicitura «pooled … CI onesto» in MEASURE90 va emendata (A-BB71); nessun dissenso interno.

## GRADO DEL TESTIMONE A-PP-63 ATTUATO

**Attuazione PARZIALE, viva.** Vale come istante: outstanding==0 al load certifica request_end + teardown Vm/RetainSet completi (Q2 Pedersen verificato; Q1 Matsakis: cfg-split coerente, ogni inc ha dec/undo). NON vale come finestra: il Δ di fase resta **ADVISORY** finché non attuati, nello stesso commit: A-MS-56 (Release/Acquire), la riga unificata pre/post (A-MS-57≡A-PP-66), A-PP-67 (arr monotono compilato sotto mem-census), A-PP-69 (buco body dichiarato). Con questi quattro, la promozione di b_work a verdict-grade resta a portata di S-92.0: la decomposizione b_boot+b_work di WP-90 non è toccata nelle cifre, solo nel grado del sigillo di fase.

## GRADO DELLE BANDE (b_peak mediana)

Cifre di testata riprodotte 20/20 al byte (b_peak(med)=20.289.946, se=1.084.655); outlier DENTRO la mediana è lo stimatore giusto (A-BB70); w16.r1 mascherato flaggato per NOME ma effetto ZERO sulle testate. **La banda pooled df=18 [19.376.426, 21.173.319] NON è citabile come 95%** (F(2,16)=6,16, ICC≈0,51): declassata ADVISORY-conditional (A-BB71). **Unica banda citabile come 95%: la banda df=2 sulle medie per-W [16.633.973, 23.915.772]** (t=4,303). Qualunque banda n>W-points richiede F-test di scambiabilità ex-ante superato (KS-BB-93-1).

## PRIORITÀ PER S-92.0

1. **Riga testimone unificata** (A-MS-56 + A-MS-57≡A-PP-66 + A-PP-67, un commit) + commento main.rs riformulato + A-PP-69 — prerequisito della promozione b_work.
2. **Emendare MEASURE90**: riga «CI onesto» (A-BB71), frase «never via min» in forma normativa + additività etichettata tautologia (A-BB72).
3. **MAD flag in forma LOO** con stato DEGENERE dichiarato (A-BB69) prima della prossima campagna.
4. **A-MS-58** (ordine A-MS-53 ≺ A-DL-52) e **A-MS-55** (req_t0 salta `__census_self`) in design.
5. **A-PP-68** (trap `gate_in_flight=`/`deferred=1`) nella battery.
6. Kill-switch armati: KS-PP-93-1 (sussume KS-MS-93-1), KS-PP-93-2, KS-MS-93-2, KS-BB-93-1, KS-BB-93-2.
