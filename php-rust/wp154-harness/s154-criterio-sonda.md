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
