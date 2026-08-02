# TEAM-CONFINE-ENGINE — Concilio WP-91 (relatore su verbali 6-Pedersen, 8-Stogov)

Fonti vincolanti: `verbale-6-pedersen.md`, `verbale-8-stogov.md`. Questo verbale è sintesi, non supersede.

## CONVERGENZE

1. **Fail-closed con riga ledgerata, mai abbandono cieco.** Pedersen (wait_up senza `server_gone=`, A-PP50/51) e Stogov (skip conservativo sul nome irrisolvibile = divergenza da dichiarare per NOME, non fedeltà) dicono la stessa cosa: ogni ramo di uscita o deviazione dall'oracle va nominato, mai lasciato implicito.
2. **Il pin/censimento in-band non basta: serve il contatore/oracle ESTERNO.** Pedersen: `grep -c` sulle sigle non morde il gutting comment-preserving → A-PP52 (contatore esterno `main_probe_fail==2`). Stogov: il .out troncato `2>&1|head -3` è byte-impossibile per costruzione → A-DS50 (pin a canali separati). Stesso principio: il bersaglio deve essere ancorato a ciò che l'oracle emette davvero, non a un artefatto del harness.
3. **Verificare contro l'oracle PRIMA di eseguire/implementare** (premessa fattuale WP-88 confermata): entrambe le refutazioni sono state prodotte a macchina, dal vivo, prima del codice.
4. **Recidiva flag-drift WP-89**: probe e braccio con flag duplicate a literal (ds35-verify:47 vs :65) → A-DS52 (PERSIST_FLAGS unico) è la stessa classe di A-PP53 (dichiarare per NOME esclusioni e vincoli del harness).

## CONFLITTI

Nessun conflitto diretto (domini disgiunti: Pedersen lifecycle/publish, Stogov LSP). Una tensione di sequenza: i kill-switch di Pedersen (KS-PP-91-1: nessuna campagna m90+ consumabile senza A-PP50) gate-ano la MISURA, quelli di Stogov (KS-DS-91-2: fixture v2 per NOME = gate di merge) gate-ano il CODICE A-DS35. Vanno onorati entrambi ma su lane indipendenti: A-PP50/51 non blocca A-DS35 e viceversa.

## PRIORITÀ per S-90.0

**La refutazione Stogov ridefinisce il primo item.** «A-DS35 fase 1 implementazione» com'era scritto è REFUTATO in tre punti: (a) lettera r2 by-ref falsa (esatta è la ref-ness, il tipo resta contravariante — A-DS48); (b) pin .out byte-impossibile (canali mescolati — A-DS50); (c) fixture v2 con 11 buchi (A-DS49) e sede singola divergente sui condizionali (A-DS51). Implementare ora significherebbe codificare un contratto falso contro un bersaglio irraggiungibile.

**Sequenza onesta proposta:**
1. **A-DS48 prima di tutto**: emendare il contratto r2 (ref-ness esatta, tipo contravariante, union in ordine canonico Zend).
2. **A-DS49 + A-DS50 PRIMA del codice**: fixture v2 (le 11 per NOME, incluse t1-t4 timing) con pin integrale stdout/stderr SEPARATI, generate dall'oracle 8.5.7; decidere per NOME se phpr modella la log-copy stderr o la cataloga.
3. **A-DS52**: ds35-verify v2 (PERSIST_FLAGS unico, header a 3 flag) — chiude la recidiva WP-89.
4. **Solo poi il codice, in sede DUALE** (A-DS51): lowering per le hoisted top-level + bind-in-registry per condizionali/dinamiche (include/eval). KS-DS-91-1 (t3 non deve fatalare, t2 non pre-output) e KS-DS-91-2 (no esenzione ctor vs iface/abstract, no fatal su widening by-ref) come gate di merge.
5. **In parallelo, lane lifecycle**: A-PP50/51 (wait_up ledgerato + escalation KILL) come gate delle campagne m90+ (KS-PP-91-1); A-PP52/53 subito dopo.

I verbali individuali restano vincolanti; KS-DS-91-3 declassa a UNANCHORED ogni consumo del pin troncato attuale.
