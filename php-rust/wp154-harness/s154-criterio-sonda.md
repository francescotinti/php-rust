# Criterio S-154 p.2 — sonda k post-BT2 (FUORI-UB D=+266,7 > UB=160 da spiegare) — PRE-REGISTRATO prima di build/run

1. **Probe s154**: copia canonica del tree corrente (pin s153) →
   `/private/tmp/phpr-census-s154-src` (APFS) + diff census s151
   (`wp151-harness/s151-census-copia.diff`) coi path NORMALIZZATI al repo
   (collaudo `git apply --check -v`: 10/10 file, offset dichiarati host.rs +30
   / mod.rs +3 = aggiunte L-BT2); ricetta PREP s151 INVARIATA
   (SOURCE_DATE_EPOCH=0, CARGO_INCREMENTAL=0, target dedicato APFS,
   `-p php-cli --features mem-census`). GUARDIA identità-ricetta: build PULITA
   dalla copia == **8370c257ae70cc8e** (pena STOP). Smoke: `smoke151-check.sh`
   con attesi s151 INVARIATI (pena STOP). CONTEGGI, mai tempo.
1-bis. **EMENDA (dichiarata nell'atto, dopo rc=3 t1 e PRIMA di ogni riuso)**:
   la build PULITA dalla copia dà 18dee21bff221146 ≠ pin — con crates/ ==
   sorgente pin @40df12f (git diff VUOTO) e ricetta identica ⇒ **il pin s153
   NON è cold-riproducibile** (pin-phpr.sh l'ha costruito su cache canonica
   CALDA dopo i build A/B di S-153; il pin s150 nacque su cache pre-prune
   fredda, per questo PREP s151 chiudeva a hash esatto). Reperto a verbale.
   La guardia identità-ricetta si chiude a **CONTENUTO**: build fredda in
   target DEDICATO, confronto byte-a-byte col pin — PASS solo se dimensioni
   uguali E ≤64 byte diversi confinati in LC_UUID+pagina firma (meccanismo
   48 B già nominato in s150-identita-candidato.md e PREP s151) E
   strings-diff ZERO. Oltre ⇒ rc=3 (divergenza REALE, STOP).
   **Raffinamento (t2 rc=3, cluster istruiti al byte)**: i 93 B di scarto
   reale sono 4 cluster TUTTI nominati — LC_UUID 16 B @0x838 · banner
   `__DATE__/__TIME__` di mimalloc 13 B @0xbb848a (pin: «Aug 18 2026
   10:02:49» = oggetto C riusato dalla cache calda S-153, dove
   SOURCE_DATE_EPOCH non entra nel fingerprint del cc-build-script; build
   fredda: «Jan 1 1970») · 2 slot firma 32+32 B in __LINKEDIT. La guardia
   ammette ESATTAMENTE queste classi: ogni cluster o è nel LC_UUID, o è in
   __LINKEDIT, o è il banner-data mimalloc (build='Jan  1 1970'/'00:00:00');
   cap totale 160 B; il confronto strings ammette SOLO le righe data/ora.
   Qualunque cluster non classificato ⇒ rc=3 (divergenza REALE, STOP).
2. **k nuovo**: `wp152-harness/bt-count.php` a BTN=100000/300000,
   k = Δ(hostcall.debug_backtrace)/ΔN, b = Δbyte/ΔN. Riferimento pre-BT2:
   k=45 ESATTE, b=2.282 B/call (s152-pesca). **Attesa BLIND: k_new 22–28**
   (criterio bt2 p.2: −20±3).
3. **Riconciliazione FUORI-UB** (meccanica): D_alloc = (45−k_new) × miheap
   6,7–6,9 ns; residuo R = 266,7 − D_alloc. Attesa al k atteso: D_alloc
   117–159 ⇒ R ≈ +108/+150 ns NON-alloc — canali candidati da NOMINARE
   (hash `Key::from_bytes` 10 chiavi/call + memcpy chiavi + `ty.to_vec` +
   clone file), nessuna cifra per canale senza mock dedicato. Se k_new ≤ ~7
   il solo canale alloc copre D (dichiarare). k_new FUORI 22–28 ⇒ si torna
   al sorgente PRIMA d'ogni conclusione.
4. **Testa hostcall rifondata** (denominatori per p.3): census ORM col probe
   s154 = copia DICHIARATA di `s151-census-orm.sh` (lock: VERIFICA soltanto,
   adattamento dichiarato; identità §3 pena census NULLO). Attese BLIND:
   residuo non-backtrace 60,9M ±2%; debug_backtrace 21,3M → ≈0,473M × k_new
   (±10%); class_exists ≈9,7M e get_declared_classes ≈4,6M ricontati.
5. Ordine: SOLO dopo la chiusura delle finestre di misura p.1 (build vietati
   in finestra); esiti in `sonda-out/` + verdetto `s154-sonda-verdetto.out`,
   rc autoritativi da file.
