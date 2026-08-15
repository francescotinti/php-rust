# WP_SESSION_143 — CONCILIO a 9 deliberato + ISTRUTTORIA eseguita: quota oggetti 1,4% ⇒ la regola pre-registrata dice B (l'arena-oggetti cade)

**In una frase**: i nove revisori hanno bocciato la scommessa «oggetti in
arena» e ordinato prima le misure; fatte in giornata, dicono che gli oggetti
sono solo l'1,4% delle allocazioni (le stringhe il 27,6%) ⇒ la via è la B.

**SCOREBOARD** (pin s142 bba8a734+eeb284b6 INVARIATO, micro non rimisurate =
per costruzione): **arith 5,5 · prop 5,6 · calls 4,7 · str 4,2 · arr 3,2 ·
re 2,6** · WP rif 1,765–1,788 (fermo, non rimisurato) · **leve perf spedite: 0
(DICHIARATO: sessione concilio+istruttoria su rotta utente)** · incidenti 14.

## Esiti secchi
1·**CONCILIO a 9** (`COUNCIL_S143_REVIEWS.md`, 2 fasi, 13 agenti): 9/9
  concordo-con-emendamenti, 0 opposizioni; ISTRUTTORIA-PRIMA 7/9; **A RIFONDATA
  unanime** (Zend NON azzera il refcount → §3.22 sistemica; 29,4 GB/run
  refutano l'arena senza riuso; binding output-capture NON emendabile; A =
  pool+refcount+handle-generazione); 6 veti confermati 9/9; a verbale: la
  scommessa compra la TAPPA ≤3×, non la parità (~15 s residui).
2·**Regola di decisione PRE-REGISTRATA** (`s143-criterio-istruttoria.md`,
  committata PRIMA dei dati): obj ≥40% ⇒ A-poi-B · <25% ⇒ B · 25–40% ⇒ riconvoca.
3·**Census CH_* eseguito** (probe b57c8183, ×2, r1==r2 ±1, parità 16 nomi
  rc=0): **quota_obj 1,38%** (box 0,69 + propsbuf 0,68) · **quota_str 27,6%
  (129,9M creazioni!)** · quota_arr 9,4% · other 61,7% (ref/vecargs, dichiarato)
  · galloc_n 471,3M == dossier (tripla conferma) · **ESITO REGOLA: B sola /
  B-poi-A** — sotto ogni kill-switch delle sedie (15/25/30/40%).
4·**Reperti collaterali**: `size_of::<Zval>()=16` — la Zval è GIÀ 16 B come
  Zend (B va rimirata su Rc-traffic/niche, non sulla taglia) · bilancio bytes
  CHIUSO per ispezione (realloc disaggregato: gfree−galloc ≈ Σ(new−old)=4,76 GB
  ≈ 4,33 osservato) · emenda v2 parser (tag=exit_mi doppiava; quote invariate).
5·Az.rev. S-142: #3 FATTA (s143-promozione.sh, copia-gate rc=0) · #2 vincolo
  attivo · #5 coda CI drena · #4 all'utente (near-miss = incidente 15?).

## ⭐ Lezioni (max 3)
- ⭐⭐ Una regola di decisione firmata PRIMA dei dati trasforma un esito
  spiazzante (1,4% vs atteso ~40%) in delibera automatica: nessuna
  ri-litigazione possibile a valle.
- ⭐⭐ Un parser di census si collauda sul TAG esatto: `exit_mi` conteneva
  `exit` e raddoppiava ogni chiave — il fattore 2,00 esatto era la firma.
- ⭐ Dichiarare `size_of` nel dump costa una riga e ha rimirato un'intera
  opzione strutturale (B non è «riduzione taglia»: è già 16 B).
