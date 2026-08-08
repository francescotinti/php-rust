# s118-criterio-treno1.md — treno-1 = V1+V2 (H-P1a/b, manifest 3148cae): rigiudizio sotto A′/banda-v2 (PRE-registrato)

1. Candidato B: HEAD + patch composto H-P1 (8bb395c risolto sopra L-A: probe fuso INVARIATO, ramo !fused col probe in prestito; 52 inserzioni additive, `hp1-composto.patch`). Build nel target CANONICO con ricetta A′ (az. rev. n.3: niente cross-target sulle guardie fini). A = stash `phpr-s117` 1656580e.
2. Admission (pena STOP): parità output 6 giudici A↔B; dump `PHPR_DUMP_OPS` INTERO byte-id A↔B sui 6 giudici (H-P1 è runtime-only: emissione INVARIATA, forma S-113 12/12).
3. A/B `s118-ab-treno1.sh` = macchina s117-ab-la INVARIATA (floor mediana-3 per binario, famiglia 1,3×min, coppie extra max 6, tie 2 dec): smoke R=2 early-stop a segno opposto, poi full R=5 ABAB.
4. Bersaglio: prop — PASS se D_med ≥ max(4,00; spread_A; banda-v2 3,33) con segni 5/5.
5. Guardie non-bersaglio a SOLO-REGRESSIONE: D_med ≥ −max(2×spread_A; banda-v2(cat); 2×quanto) su arith/calls/str/arr/re (banda-v2: 0,80·0,50·7,50·6,67·0,00 — re protetto dal pavimento 2×quanto).
6. Esito: PASS ⇒ candidato CONSERVATO (`phpr-s118-treno1`) e promozione §6 PIENA (batteria+corpus nomi E contenuto via `scripts/corpus-gate.sh`+fixture+micro+held-out) in questa sessione se il muro orario regge, altrimenti S-119 col candidato congelato — mai promozione parziale. FAIL/segno-opposto ⇒ verdetto a verbale, revert del tree e release ripristinato con build ricetta H1==1656580e AL BYTE, pena STOP.
7. Bisezione (solo se guardia sfondata): stacca l'ULTIMO vagone (V2) e riesegui UNA volta; max 2 giri.
8. Nessuna cifra fuori da `.tsv`/verdetto emessi dallo script; il tree torna SEMPRE al pin a fine giro.
