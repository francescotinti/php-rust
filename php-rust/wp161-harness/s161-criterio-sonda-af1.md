# s161-criterio-sonda-af1.md — sonda surplus L-AF1 (orologio §4, dovuta S-161; rev. S-160 #5: reperto UB «AL BORDO non robusto») — scritto PRIMA del run

1. Oggetto: risolvere il reperto UB di L-AF1 (D_R5=+16,0 vs UB 12,0±2,5+rumore
   — fuori di 0,5 con la formula del criterio; conferma +15,0 DENTRO):
   CONTEGGI census sui due bracci + RIMISURA su stash FERMI + DECISIONE sul
   modello di famiglia. Copione `s161-sonda-af1.sh` (nuovo; pezzi DICHIARATI:
   struttura e judge math COPIA di s159-sonda.sh — mediane, floor3, rumore
   drop-1 —; interfaccia census s156 `PHPR_MEM_CENSUS` + righe
   s149name/s149sum; parser python adattato ai nomi arrfilter).
2. Bracci CONTEGGI: probe-A = build census (`-p php-cli --features
   mem-census`, target dedicato /private/tmp) della COPIA del tree HEAD con
   patch L-AF1 INVERSA (`patch -R -p1`, SOLO host.rs dal manifest
   `wp160-harness/s160-af1-copia.diff`: patching=1 ESATTO, zero .rej, pena
   rc=6; il commit dcab1059 tocca anche loc_dente.rs ma è un TEST, non
   compilato in `-p php-cli` — esclusione DICHIARATA); probe-B = build census
   della copia del tree HEAD; hash MISURATI nel verdetto; build fuori dal
   target canonico (pin intatto).
3. Driver conteggi: `m-arrfilter-census.php` = m-arrfilter.php con chiamate
   10000→200 (adattamento DICHIARATO; 200×1.000 = 200.000
   invocazioni-elemento); R=2 repliche/probe; MARCATORE preteso
   `AF-OK 100000` ESATTO pena rc=8 (emenda §3 proposta S-160: mai il solo
   A==B); identità s149sum (hostcall_n==sum_name, unnamed=0, overflow=0)
   pena census NULLO rc=5.
4. Attese CONTEGGI pre-registrate (tolleranza ZERO): Δ(A−B) name=array_filter
   = 200000 ESATTO (1 passaggio vec-args/elemento: `call_callable(c,
   vec![v])` nel loop generico, assente nel fast path `call_closure_one`) ·
   Δ = 0 su OGNI altro nome · Δ hostcall_n = 200000 ESATTO · repliche
   identiche o differenze DICHIARATE. Un Δ fuori attesa ⇒ dichiarare e
   tornare al sorgente, NESSUNA taratura (rc=5).
5. RIMISURA: A/B R=5 ABAB, bracci = stash FERMI `phpr-s160-gemelloA` (atteso
   f2d17f18, ==pin s159) vs `phpr-s160-af1-B` (atteso ceeb6e76, ==pin s160),
   pena rc=9; giudice m-arrfilter N=10M dichiarato dal sorgente; floor3 per
   binario su empty.php; mediane + rumore drop-1 (matematica s158/s159
   INVARIATA); output: MARCATORE `AF-OK 5000000` preteso su ENTRAMBI i bracci
   E diff A==B, pena rc=2; quiescenza (s129) + sentinelle LS inizio/fine nel
   `.out`; lock della SESSIONE solo verificato; verifica POSITIVA
   dell'esistenza di TUTTI i path d'ingresso PRIMA del run (emenda §3
   proposta), pena rc=7.
6. DECISIONE (solo se p.4 ESATTO e D_rimisura > rumore con segno +): la cifra
   a registro per L-AF1 diventa D_rimisura ± rumore su stash FERMI (da
   riportare in PERF_MAP). Con Δ census = 1 passaggio/elemento, il costo del
   passaggio CLOSURE-vec (vec![v] + dispatch call_callable/invoke_value +
   match mode + Rc-bump `c.clone()`) = D_rimisura ± rumore ns. Bivio
   PRE-REGISTRATO sul modello di famiglia: (a) se |D_rimisura − 12,0| ≤ 2,5 +
   rumore ⇒ il coeff UNICO 12,0±2,5 (s159) REGGE, il reperto «al bordo» si
   risolve DENTRO; (b) altrimenti la famiglia si SDOPPIA: cammino
   hostcall-vec = 12,0±2,5 (taratura s159, invariata) · cammino closure-vec =
   D_rimisura ± rumore (taratura s161) — e l'UB della prossima leva usa il
   coeff del SUO cammino (leve p.4 NEXT_SESSION: autoload k=1 → hostcall;
   array_map string-callable → da classificare al criterio). Drift dichiarato
   vs finestra s160 (+16,0) e post-pin (+15,0) col rumore drop-1 della
   finestra nuova.
7. Esiti a FILE: verdetto `s161-sonda-af1-verdetto.out`; rc SOLO da
   `sonda-out/sonda.done` (0=valido · 2=parità/marcatore rimisura · 5=identità
   census/fuori attesa · 6=patch/guardia · 7=setup/build/path/quiescenza ·
   8=marcatore census/probe muto · 9=lock/stash). OUT in directory NUOVA
   `wp161-harness/sonda-out` (emenda §3 proposta: mai clobber di out altrui).
   FINESTRA: nessun altro run/build in volo; CI ferma sul lock misura.
