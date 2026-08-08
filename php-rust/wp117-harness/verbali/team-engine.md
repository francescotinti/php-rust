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
