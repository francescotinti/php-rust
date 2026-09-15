# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-175 = PROMOZIONE della leva «Sweep-in-op» (braccio C) a PIN s175** (s174-promozione.sh
emendato per il tag, manifest s174-promozione-tag-s175.diff): pin phpr **5de14d6856d760a8** (identità a contenuto col
candidato b6c4b587: soli LC_UUID/firma) + server **9b9179d4dd3d95bd**; batteria 1748/0/2 · corpus 1412 ×2 ZERO flip ·
fixture chain 10/10 · 20 gate byte-id (fx-sw1 NUOVA) · conferma post-pin arith-dq +3,56 (5/5, rumore 0,16) = SOLO
SEGNO · ORM 16 nomi · hk 0E/0F ⇒ rc=0. **Coppia DOVUTA fatta**: WP t20 1,765 COMPATIBILE [1,738;1,799] (6/6 pulite,
banda ON 0,026) · ORM t3 rc=0 (t1/t2 rc=8 = quiescenza 3×30 s battuta da flare mediaanalysisd ⇒ pre-attesa anti-flare
s175-lancio-orm-calmo.sh): rapporto [6,936;7,014] ≤7,05, Δ abs nel rumore, **sentinella oracle 4,97 FUORI [4,83;4,94]
⇒ Delta_norm non giudicante, voce riaperta**. **Az.rev. S-174 TUTTE CHIUSE**: (a) gamba pressione GC (fx-sw2-gc.php a
invarianza pin==stash; MS rompe i siti fusi, M3 = 3ª clausola forzata rompe fused/ctl/loop/scsc ⇒ clausola VIVA nel testo
unico), (b) liste INT = soli discriminanti (corsa 5 rc=0), (c) solo segno. **Mock «flag gc-idle»** (s175-criterio-mock.md,
A=sorgente del pin, Z=pin controllo nullo, B=MS, C=M3, R=5): **arith D_B +2,36 direzione · prop D_B +4,67 CIFRA**, M3
+0,80/+1,93 ⇒ tetto NOMINATO su prop-dq: la leva si disegna. Leve: 1 · A/B eseguiti: 1 (mock) · incidenti: **1** (#1
igiene: micro e conferma post-pin nella finestra con updater Little Bird attivo e Data 0,47G, non nel .out ⇒ micro a
GUARDIA, da rimisurare) · revisione S-175 (lente PROCESSO, wp174-harness/revisione-s175.md): REGGE CON RILIEVI — (1) divergenza «momento della
raccolta GC» NON era a catalogo ⇒ SANATA in sessione: §3.33 in PHPR_DIVERGENCES (az.rev. (a) chiusa per NOME); (2) mock
lanciato dopo ORM rc=8 contro il criterio p.5 (quiete s129 PASS nel copione; tetto 4,67 a filo del rumore 0,93): rerun dopo
ORM rc=0 in S-176 col lanciatore che gata su `^rc=0`; (3) micro/conferma non di record (incidente #1); (4) catena: STOP a
commit fallito, ogni lanciatore nuovo committato PRIMA del run; (5) istruttoria drift sentinella ORM prima della prossima coppia · sessioni senza misura: 0.

## Scoreboard (pin NUOVO s175 phpr 5de14d6856d760a8 + server 9b9179d4dd3d95bd)
**arith 2,3 ↓ · prop 2,6 ↓ · calls 4,6 ↓ · str 4,0 ↓ · arr 3,1 = · re 2,5 =** (micro a GUARDIA: finestra sporca) ·
giudici puliti (mock, A=sorgente del pin): **arith-dq 20,08 = 2,32× · prop-dq 36,13 = 2,58×** (oracle 8,64 / 14,00) ·
coppia t20 WP 1,772→1,765 · ORM net [6,936;7,014] · corpus 2655 (1412 fail congelati) · batteria 1748/0/2 · denti: run.rs 7361 ·
CI: lock s175 ⇒ job in skipped-busy dalla sera del 13 (job 51ca44dbba04 in requeue disk-low).

## §S-176 — ordine
0. **PRE-FLIGHT**: Data ≥10G BLOCCANTE (a fine S-175: 5,67G; pesi UTENTE: swap fino a 9G nel volume VM dello stesso container,
   Application Support/Claude 9,6G, Google 8,1G, updater Little Bird che riscarica ~1,7G in cache) · MySQL wp8 con l'elenco
   (se giù: daemonizer double-fork su `mysql-wp8/data`, MAI start naive) · lock col TOKEN `s176` · pin s175 per hash · CI feed.
1. **Micro R=5 al pin s175 in finestra PULITA** (sana l'incidente #1: `wp97-harness/micro/run-micro.sh` con quiescenza PRIMA;
   registrare sentinelle e Data nel .out) → scoreboard di record. Poi **gh-status-sync a mano** (skill con `model:`; numeri
   corpus identici, pin/data/micro da aggiornare).
2. **LEVA «flag gc-idle»** (p.3a S-175, tetto misurato): un solo `bool` «gc idle» sul Vm mantenuto dai siti che cambiano una
   delle tre clausole (push/drain di `gc_buf` e `gc_buf_head`, insert/take di `gc_light_demoted`, `gc_enabled`, insert/retain
   di `gc_cycle_roots`/`gc_ctr_roots`, refresh di `gc_sweep_bound`); `sweep_idle` = IN_DESTRUCTOR || flag (un solo testo,
   handler + siti fusi). Criterio PRIMA: bersaglio prop-dq a nomina (attesa D ∈ [2;5] = quota del tetto 4,67), guardia
   arith-dq (attesa [1;2,5], direzione); KILL: D_prop < max(1, rumore): il costo è nei load/cmp, non nella loro somma;
   mutante = flag mai aggiornato ⇒ fx-sw2-gc + fx-sw1 devono ROMPERSI (gamba pressione + note pendenti); bracci A=pin s175,
   Z=gemello, B=flag; R=5 rotazione; promozione con copia dichiarata di s174-promozione.sh (tag s176) + coppia dovuta.
3. **Census «op in place + Sweep» su WP/ORM** (portata reale della fusione e del flag): build op-census (feature `op-census`,
   `PHPR_OP_CENSUS`, ricetta wp147-harness/s147-census-orm.sh) e conteggio dei bigrammi {BinarySCSCDst, BinarySTDst,
   P3, P1/P4}→Sweep e IncDecSlotJmp→CmpJmpSC sui workload della coppia (media WP + ORM). MAI cifra di tempo dal census.
4. Perimetro typed (invariato): census WP/ORM di `$o->x = $o->y OP C` su classi typed PRIMA di una fetta 4.
5. Solo dopo: calls 4,6 (census Call/BinarySS/Ret, args-Vec) → str 4,0 (ConcatNConst, substr).
6. **Emenda del canone ORM** (lezione 3): pre-attesa anti-flare / assestamento a streak PRIMA del gate di quiescenza per
   gamba (copia dichiarata da s175-pair.sh); banda sentinella oracle: istruttoria drift (t19 4,94, t20 4,97).
7. Quesiti residui: ictx oracle1 segnalata (a verbale) · c0 positivo · census server (25° slitt.) · ratifiche §3 · dtor-in-dtor
   (catalogo per NOME in PHPR_DIVERGENCES) · momento della raccolta GC = §3.33 CATALOGATA (S-175).

## Aperture per NOME
flag gc-idle (leva) · census «op in place + Sweep» WP/ORM · micro pulite s175 · banda sentinella ORM (4,97) · emenda canone
ORM anti-flare · dtor-in-dtor (catalogo) · IC set typed (fetta 4) · census typed · calls → str ·
assign-form · §3.32 · §3.30 · residuo slot 2,75/op · tupla guard · F1/F2 (SOSPESE) · autoload statiche · sonda strmap · gamba
server census · §3.28 · §3.29 · §3.27 · §3.26 · §3.25 · §3.24+§3.23 · slot-load · §3.22 · depr. float→int · warning ×2 ·
div. RMW · objmap → GC · evalcls 316,9× · refl 42,4× · re +2 · get_gc · latin1 · dbal 10 nomi · pavimento 4 ns/iter su loop corti.

## NON riproporre (i veti restano)
**S-175: misure di record con Data <10G o senza watchdog disco · scansioni `find`/`du` durante una finestra di misura ·
token `phpr`/`php-server` nell'argv di monitor e attese (quiescenza s129) · rilancio ORM senza pre-attesa anti-flare ·
cache/dir di app utente bloccate oltre la finestra (si ripristinano in chiusura) · un demone che patcha un copione di
misura SENZA dichiarazione in coda al criterio.** S-174: commenti in coda a righe con `;` · collaudo del solo blocco python
· floors in una stringa (zsh) · skill con `model:` a metà sessione · banda-layout da ricostruzione della stessa sorgente ·
guardia promossa a bersaglio senza rerun completo. S-173: attese «intatte» senza reset del dst · `perl -0pi` con `\|\|` ·
pulire ab-out/ durante una build · copertura del mutante senza liste a-verdetto · cifra sotto soglia anche con 5/5. S-172:
push a mano nelle catene pin-*.sh · `git archive` senza touch · fixture single-shot con IC · `a || b && c` senza graffe.
S-171: lock senza TOKEN · xctrace senza trap/Data · build/run durante la coppia. S-170: driver senza `[ -e ]` · path non
quotati · `rm -rf` di target intere · mock unsafe. Trasversali: NaN-boxing/fn-table/arena (⚖️) · BOLT/PGO · pin senza
collaudo · rc da pipe · promozione sotto banda · cifre composte.
**Riscritto** 2026-09-15 (chiusura S-175; storia in `sessions/` · `gaps/`).
Pre-flight S-176: pin phpr **s175 5de14d6856d760a8** + server **9b9179d4dd3d95bd** · **Data ≥10G (bloccante: liberare
PRIMA)** · MySQL wp8 con l'elenco · uploads sotto guardia · corpus 1412 · lock col TOKEN `s176` · CI feed · lettura: REGOLE.md
→ QUI → wp174-harness/revisione-s175.md → s175-mock-verdetto.out + s175-criterio-mock.md → s175-mutante-gc-verdetto.out →
s175-pair-verdetto-t20.out + s175-orm-coppia-verdetto.out → WP_SESSION_175 → gaps/REPORT_GAP_175 → PERF_MAP.
