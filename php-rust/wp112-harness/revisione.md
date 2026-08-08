# Revisione S-112 — lente PROCESSO (revisore singolo, REGOLE §7)

**Verifiche sui raw**: ordine commit d8d7c68 (criterio, 01:29:46) → c04c273 (leva, 01:34:58) → verdetto 01:46 → pin 02:01. Timestamp di ab-out/, heldout-out/, admission-out/, corpus-gate/ letti su disco: A/B full 01:36–01:38 e held-out 01:41–01:43 coerenti coi tempi attesi per R=5; corpus 01:45–01:58 col hash **f71abd2a in progress.txt** ⇒ corpus/fixture/micro sono girati davvero sui byte del pin. Held-out: anche con la baseline fresca (wploop 6,69+0,12=6,81) il candidato 6,76 passa — la scelta della baseline S-111 non decide il verdetto.

**Il punto che RIDIMENSIONA**: senza l'emendamento della guardia calls, il criterio pre-registrato (punto 5: «una guardia sfondata ⇒ NIENTE promozione») **refutava**. L'emendamento invoca REGOLE §5, che però disciplina i GATE (batteria/corpus/fixture/micro), non le guardie di MISURA (§3): prestito di categoria, deciso DOPO aver visto i dati. Mitiganti reali: direzione = miglioramento (guardia anti-tassa che morde un miglioramento è errore di stesura, intento S-111 documentato); dichiarato nel verdetto e nella lezione; la guardia emendata refuterebbe comunque un −1,50. Aggravante: il +1,50 viene **attribuito a layout senza misura propria** e subito arruolato come punto N=2 della banda — se la banda passa 0,67→1,50, le prossime guardie non-bersaglio si allargano 2,2× su un'attribuzione raccontata, non provata.

**Ridimensionamenti secondari.** (1) Admission del candidato già compilato alle 01:32:13, a 2,5 minuti dal commit del criterio: «committato PRIMA di ogni implementazione» non regge alla lettera; regge l'ordine che conta (criterio prima della misura, smoke 01:36). (2) Batteria mai eseguita sui byte del pin (gemella b70e049a; re-hash divergente risolto con cp, non ri-batteria): dichiarata e sostanzialmente accettabile, ma §6 formalmente non rispettato. (3) La derubricazione di (b) usa la calibrazione S-106 (+3,5 stimati) — lo stesso strumento vietato nei criteri, qui usato per scartare: asimmetria non dichiarata. La riconduzione a famiglia (c) è argomentata (collo per-op, non dispatch). (4) Igiene: `corpus-gate/` NON è nel .gitignore (output di run nel working tree); contatore morsi-pipe fermo a 4, nessun verbale in-repo del morso di stasera: se c'è stato, è un incidente non contato.

**Verdetto: la promozione REGGE nella sostanza (Δ 8,2× la soglia, held-out 3/3 su entrambe le baseline), ma è «PROMOSSA su criterio emendato in corsa», non «promossa dal suo criterio PRE».**

## Azioni
1. Rietichettare in WP_SESSION_112/GAP_TREND: «promossa su criterio emendato in corsa (guardia calls)».
2. Banda-layout resta 0,67 finché il +1,50 non ha un A/B di attribuzione (leva nulla stesso testo); non allargare le guardie su N=2 non misurato.
3. Codificare in REGOLE (sostituendo una riga): guardie di misura si emendano solo con re-run del criterio emendato; solo-regressione come stesura di default.
4. Aggiungere `corpus-gate/` al .gitignore; verbalizzare e contare il morso pipe (4→5) se avvenuto.
5. S-113: ordine §6 pieno (batteria → re-hash → stash sullo stesso binario) o dichiarare la deroga come norma, una volta.

*(Azioni 1-4 SALDATE in chiusura S-112 — stessa sessione; az.5 iscritta nell'ordine S-113. Morso pipe CONFERMATO: primo run della fixture chain col rc in coda a `tail`, rieseguito col rc dal comando — contato, 4→5.)*
