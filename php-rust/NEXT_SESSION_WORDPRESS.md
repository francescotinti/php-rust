# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-173 = LEVA L-SL2 fetta 3 (prop, residuo del corpo) PROMOSSA (pin NUOVO
s173)**: P3 = peephole runtime `Op::PropGetSlot`+`BinarySTDst` (guardie IC get verbatim + dominio i64
di BinarySTDst, store in place su dst Long altrimenti `reg_store_slot`, `ip+2`: niente push/pop, un
dispatch in meno) + P4 = un solo `borrow_mut` nel probe sigillato P1 quando recv == slot (guardie
get+set verbatim, solo il caso in place); zero unsafe, nessun corpo nuovo. A/B R=5 coi TRE bracci
RUOTATI per coppia: **prop-dq 52,53→44,93 (P3, D +7,60) →41,13 (P3+P4, D +11,40; 2,95× l'oracle
contro 3,77×)**, attese [2;8]/[2,5;12] CENTRATE, KILL-3 no, guardia arith-dq piatta (+0,12); **C−B
+3,80 firmato NON nominato ⇒ P4 solo direzione**; disasm bl 5974→5983→5986 (+12, dichiarato);
mutante P3/P4 rc=0 (corsa 2) · **az.rev. S-172 TUTTE CHIUSE**: mutante P1/P2 rc=0 (corsa 2; typed/
private/dynamic = fuori perimetro: IC set solo su classi plain), A/B alternato B↔C ⇒ **P2 = +4,87
CIFRA**, `--braccio` col commit sorgente, PIN_REGISTRY corretto · promozione rc=0 al tentativo 2 (t1 rc=101 = dente run.rs 7274 > cap 7200, alzato DICHIARANDO): build ricetta = candidato a contenuto (da4921a5 vs 19c0540e: soli LC_UUID/firma), batteria 1748/0 (+2 denti dichiarati), churn neutralizzato al byte, corpus 1412×2 ZERO flip, fixture chain 10/10, 19 gate byte-id (fx-sl3 NUOVA + fx-sl3-div), micro R=5, conferma post-pin prop-dq +11,67 rumore 0,93 segni 5/5 (52,93→41,27), ORM 16 nomi == baseline (3484: 3E/13F), hk 1665 0E/0F · leve: 1 ·
incidenti: 1 (log della build C cancellato durante la build) · revisione S-173 (lente MISURA,
wp172-harness/revisione-s173.md): REGGE CON RILIEVI (drop-1 0,07 = tick; banda tra run 1,14; 0,94 non ri-derivato; appaiate P2 [4,87;5,60], P4 +3,87 sotto soglia; decomposizioni = ipotesi; pre-registrazione e igiene REGGONO) · sessioni senza misura: 0.

## Scoreboard (pin NUOVO s173 phpr da4921a52eba0187 + server 2d2adc4549a820e0)
**arith 2,8 (2,7: tick) · prop 3,0 ↓↓ (3,8) · calls 4,8 (4,7: tick) · str 4,1 = · arr 3,1 (3,0: tick, guardia di P3 senza regressione) · re 2,5 =** · mc2 ~155 / mc3 181 (non
rimisurati) · arith-dq 23,8 vs 8,64 · **prop-dq 41,13 vs 13,93 = 2,95×** (conferma post-pin +11,67 rumore 0,93 segni 5/5) ·
dispatch 1,75/op · **coppia t19: t19 LANCIATA al pin s173 in chiusura S-173 (03:07, pair → orm in catena via s173-lancio-*.sh): DA LEGGERE in S-174 (attesa direzione ≤0; ORM: regola 4 se net resta >7,05)** · corpus 1412×2 · batteria 1748/0 · denti: run.rs 7274 (cap dichiarato 7200→7274)
· mod.rs 25909 · host.rs 7726 · coda CI: coda 17 job, ultimo evento «DONE af234ee33993 OK 2026-09-10 01:53:52»; runner in disk-low finché Data <10G (target canonica potata in chiusura).

## §S-174 — ordine
1. Coppia t19 (lanciata al pin s173 in chiusura S-173): leggere s173-pair-verdetto-t19.out e s173-orm-coppia-verdetto.out (rc SOLO dai .done in pair-out/orm-out); WP in banda [1,738;1,799] = compatibile, nessun claim; ORM net >7,05 ⇒ istruttoria (regola 4). ORM: se net resta > 7,05
   scatta la regola 4 (istruttoria) — voce in sospeso da t18 [7,090;7,092].
2. **Az.rev. S-173** (revisione, PRIMA di nuove leve): (a) nei `.out` la banda tra run (stesso codice, binari diversi) accanto al drop-1 e 0,94 RI-DERIVATO su prop-dq (`s173-ab-leva.sh:80` lo importa dal giudice arith); (b) il copione A/B stampi ANCHE la mediana delle differenze appaiate, cifra P2 come intervallo [4,87;5,60]; (c) decomposizioni del residuo senza cifre o marcate «ipotesi» (fatto S-173 in NEXT/REPORT_GAP/verdetto: verificare); (d) il verdetto del mutante nomini i blocchi `-each` INTATTI come prova del dominio (oggi sono solo nelle liste INT).
3. **Prop, dove sta il residuo** (prop-dq 41,13 − 13,93 = 27,2 ns/iter; IPOTESI di riparto, cifre da altri binari: dispatch 7×1,75=12,3 + Sweep×2
   ≈5,8 + CmpJmpSC/IncDec ≈7 + corpi ≈2): il corpo prop è quasi ESAURITO sul giudice — il residuo è
   dispatch+Sweep+controllo di loop (comune a TUTTE le categorie). Candidate: (a) Sweep elidibile
   quando il frame non ha temporanei vivi (census dal dump: 2 Sweep/iter su 7 op), (b) fusione
   CmpJmpSC+IncDecSlotJmp in un op di loop (bigramma 0012/0019 = loop `for` canonico); criterio
   proprio, giudice arith-dq (loop puro) + prop-dq guardia; mock magro PRIMA se il corpo non è leggibile.
4. **Perimetro typed**: IC set non riempita su classi con prop tipizzate/private (mutante S-173) ⇒ P1/P4
   non mordono sul PHP moderno (WP/ORM usano typed props): census WP/ORM delle forme `$o->x = $o->y OP C`
   su classi typed PRIMA di decidere; leva «IC set su typed» = fetta 4 candidata, solo con census.
5. Generalizzazione (calls 4,7 → str 4,1): forme DIVERSE, census dal dump prima, criterio per categoria.
6. xctrace #2 SOLO con Data ≥20G e purge ktrace in `trap EXIT`.
7. Quesiti residui: c0 positivo · Sweep/iter 2,9 · (b) T2/A2 · census server (23° slitt.) · ratifiche §3.

## Aperture per NOME
Sweep elidibile · fusione loop CmpJmpSC+IncDecSlotJmp · IC set typed (fetta 4) · census typed WP/ORM ·
calls → str · assign-form (BinarySCSC+BinaryDst) · §3.32 riga Deprecated prop dinamica · §3.30 ·
xctrace pin/B/m13 · residuo slot 2,75/op · tupla guard · F1/F2 (SOSPESE) · autoload statiche · sonda
strmap · banda sentinella ORM (4,82) · gamba server census · §3.28 · §3.29 · §3.27 · §3.26 · §3.25 ·
§3.24+§3.23 · slot-load · §3.22 · depr. float→int · warning ×2 · div. RMW · objmap → GC · evalcls
316,9× · refl 42,4× · re +2 · get_gc · latin1 · dbal 10 nomi.

## NON riproporre (i veti restano)
**S-173: attese «intatte» su forme in loop senza reset del dst (dopo la 1ª iterazione è Long) · edit
di copioni con `perl -0pi` e `\|\|` nel needle (alternanza regex: ha riscritto la riga 1) · pulire
`ab-out/` mentre una build vi scrive (log della build C perso) · copertura del mutante senza le liste
a-verdetto (rc=7 ×3) · cifra a P4 sotto soglia anche con direzione 5/5.** S-172: push a mano durante
una catena con pin-*.sh · `git archive` in target condiviso senza touch · fixture single-shot con IC ·
righe dopo `unset($r)` sulle stesse variabili · `a || b && c` senza graffe · `git show <sha>:crates/...`
(usare `:./`). S-171: lock senza TOKEN · `phpr` nell'argv delle attese (symlink neutri) · xctrace senza
trap EXIT e Data ≥20G · build/run durante la coppia. S-170: driver senza `[ -e ]` · patch senza
`--relative` · path non quotati · `rm -rf` di target intere · mock unsafe. Trasversali: NaN-boxing/
fn-table/arena (⚖️) · BOLT/PGO · pin senza collaudo · rc da pipe · promozione sotto banda · cifre
composte tra binari diversi.
**Riscritto** 2026-09-10 (chiusura S-173; storia in `sessions/` · `gaps/`).
Pre-flight S-174: README/COVERAGE pubblici sono al pin s172 (docs af234ee3): risincronizzare via gh-status-sync a inizio S-174 (corpus 20 min, non durante la coppia). Pre-flight S-174: pin phpr **s173 da4921a52eba0187** + server **2d2adc4549a820e0** (SOLO via pin-*.sh; stash bracci
`phpr-s173-f3-B/C` NON pin) · Data ≥10G (≥20G se xctrace; corpus-gate ~5G transitori) · MySQL wp8 con
l'elenco (S-173: era GIÙ, riavviato col daemonizer perl double-fork+setsid sul datadir esterno) ·
uploads sotto guardia · corpus 1412 · lock misura da CREARE COL TOKEN `s174` · CI: coda 17 job, ultimo evento «DONE af234ee33993 OK 2026-09-10 01:53:52»; runner in disk-low finché Data <10G (target canonica potata in chiusura) · coppia
dovuta SOLO se il pin cambia · lettura: REGOLE.md → QUI → wp172-harness/s173-verdetto.out +
revisione-s173.md → s173-f3-verdetto.out → s173-criterio.md → s173-azrev-criterio.md → WP_SESSION_173 →
PERF_MAP.
