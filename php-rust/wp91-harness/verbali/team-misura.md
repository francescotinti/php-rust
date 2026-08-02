# TEAM-MISURA — Concilio WP-91 (relatore; fonti: verbale-5-bak.md, verbale-7-leijen.md — i verbali individuali restano vincolanti)

## CONVERGENZE
1. **Cifre riprodotte**: Bak ricomputa dai 40 raw al byte (b_base=19.575.603,2, b_ret0=19.329.843,2, OLS dof=2 corretto); Leijen non contesta i numeri ma la loro semantica.
2. **Esito P-RET0**: entrambi confermano che la refutazione REGGE (b_ret0≈b_base, ratio 0,987, insensibile alla soglia in (0,5; 0,987]).
3. **Evidenza a macchina, mai fuori banda**: Bak esige robustezza emessa in-band (A-BB61/62, KB-91-1/2); Leijen esige read-back e pin (heaps_total==W+1, commit==peak su ogni raw). Stessa dottrina.
4. **Clamped dt 1-5**: stessa lettura causale — decommit sincrono al free (purge_delay=0) di un lato dentro la finestra dell'altro; dt5.bfirst (Bak) e arena.c:2059-2067 (Leijen) puntano allo stesso meccanismo. I due esperimenti proposti sono UNO: rerun con purge_delay≥finestra (A-BB63) + riga Δpurged/Δpurge_calls con pin «clamp ⇔ Δpurged>0» (A-DL50).

## CONFLITTI
**La riqualifica di Leijen cambia la lettura del VATTR/P-RET0 di Bak? SÌ, nella causa; NO, nell'esito.** Leijen dimostra (prim.c:504→os.c:591) che su macOS il contatore committed è monotono: b è la pendenza del **PICCO** di commit, non del trattenuto; i knob retain/abandon/purge agiscono POST-picco. Quindi b_ret0≈b_base non è una scoperta discriminante ma una **conseguenza quasi tautologica della metrica**: la robustezza statistica di Bak (2σ-floor 16.691.057 ≥ 0,8·b_base, anti-moda, tie-alt) è aritmeticamente valida ma misura la solidità di un negativo che la metrica garantiva in partenza. Tensione formale: **KB-91-1** (verdict-grade ammesso con robustezza in-band) vs **KL-91-2** (ogni VATTR su knob post-picco è VOID finché b è definita su committed_postcollect_win0). KL-91-2 è più restrittivo e prevale finché la metrica non è riqualificata: la citazione corretta diventa «P-RET0 NOT-attributed; robustezza mostrata (Bak); braccio strutturalmente non discriminante per b-come-PEAK (Leijen)». Le due sedie sono **compatibili** — Bak giudica lo stimatore, Leijen la metrica — ma la composizione declassa la classe VATTR-post-picco, non solo l'etichetta g3 (che per Bak era già ADVISORY-inherited). Rafforzo incrociato: Q2 di Leijen (retain solo small, page.c:749) spiega perché il braccio era mezzo-morto anche a prescindere dal picco.

## PRIORITÀ ITERAZIONE 2 (attribuzione di b)
1. **A-DL48 subito** (pre-condizione di tutto): dichiarare commit≡peak su macOS; giudice verifica commit==peak_commit su ogni raw (KL-91-1). Costo nullo, riorienta la lane.
2. **Census+collect IN-REQUEST al picco** (A-DL49, worker vivi, pin heaps_total==W+1; gate copertura ≥90% del commit o ADVISORY, KL-91-3). Unico braccio che decompone ciò che b misura.
3. **Δcommitted per finestra disgiunta** (contatore monotono ⇒ Δ esatto, lezione WP-88): attribuzione per FASE, complementare al census.
4. **Esperimento clamped unificato** (A-BB63+A-DL50): purge_delay≥finestra con read-back, predizione ex-ante nel header, riga Δpurged/Δpurge_calls.
5. **Robustezza VATTR in-band** (A-BB62, +A-BB61 tie, +A-BB64 sanatoria doc): necessaria per ogni FUTURO negativo, ma da sola non attribuisce nulla — dopo i bracci 1-3.
6. **Ritiri**: ord44 fuori dalla lane commit (A-DL51); banda VATTR post-picco congelata da KL-91-2.
