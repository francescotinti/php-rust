# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-176 = micro DI RECORD al pin s175 + leva «flag gc-idle» a SOLA DIREZIONE (tenuta nel tree, NON
promossa) + census Sweep/typed su WP-ORM + canone ORM emendato.** Pin INVARIATO phpr **5de14d6856d760a8** + server
**9b9179d4dd3d95bd**; **tree = pin + flag gc-idle** (commit d1849bc: `Vm::gc_idle [bool;2]` indice main, `sweep_idle` legge
UN byte, `gc_idle_set` unico store, ricalcolo esatto a ogni uscita di `gc_sweep_impl`; mutante MF morde fx-sw2-gc/fx-sw1;
B a parità) + contatori census (feature op-census, fuori dal binario di parità). A/B R=5 a 4 bracci (s176-flag-verdetto.out):
**prop-dq 36,20→33,47 = +2,73/+2,67** (rumore 0,47, SL 0,20, attesa [2;5] centrata; ~60 % del tetto MS +4,20 = companion, non cifra: D «vale 0» a criterio) ·
arith-dq +1,04 · nessuna regressione ⇒ **rc=3**: sotto il pavimento 4, veto «promozione sotto banda», guadagno tenuto
(keep-partial-wins) con **obiettivo NOMINATO: promozione del tree COMPOSTA con la prossima leva prop-dq** (p.1). Micro di
RECORD (finestra pulita, sentinelle nel .out, incidente #1 S-175 SANATO): arith 2,4 · prop 2,6 · calls 4,6 · str 4,1 · arr 3,0
· re 2,5. gh-status-sync fatto a mano (sonda 1017/2143, corpus 2655/1412 identici; root README era fermo a S-172). **Census
«op in place + Sweep»** (s176-census-verdetto.out, parità ORM/media rc=0): Sweep = 10,88 % degli op ORM / 7,07 % media, forme
in place su Long ≈2 % dei Sweep ⇒ la coppia NON vede Sweep-in-op né il flag; predecessori reali StoreSlot/JumpIfFalse/Pop/
PropSetPop/Jump; lettura (ipotesi, non cifra): ORM ~63 ns/op vs ~6,7 nei micro ⇒ i Sweep di ORM stanno sotto la risoluzione della coppia: il 7× è nei CORPI (il flag resta però letto da ogni Op::Sweep: 59M/ORM, rilievo 4).
**Census typed** (s176-census-typed-verdetto.out): ORM 8,99M scritture prop: **miss 55,4 % · IC plain 36,5 % · typed 8,1 %**;
media plain 82,4 % · miss 17,6 % · typed 0 % ⇒ fetta 4 ammissibile (<10 %) con presidio typed; perimetro vero su ORM = IC MISS.
**ORM**: istruttoria sentinella (banda [4,83;4,94] fondata su finestra fredda; storico 4,88..5,08) ⇒ canone E3 (assestamento a
streak nel gate per gamba) + E4 (ORA_REF 4,94, banda [4,84;5,04], REF2 [34,73;34,81] — sanata in chiusura, rilievo 6) PRE-registrato: s176-orm-coppia.sh +
s176-criterio-orm.md, si esegue alla prossima coppia. Leve: 1 · A/B: 1 · incidenti: **0** · revisione S-176 (lente MISURA):
wp176-harness/revisione-s176.md: REGGE CON RILIEVI (63 % cifra indebita → companion; PREV su binario diverso; «rerun» irraggiungibile ⇒ composizione; il flag è letto da OGNI Op::Sweep ⇒ il census non lo esclude su ORM; «0,5 %» vietata da §3; E4 mediana 4,94 SANATA) — azioni in p.1/p.4 · sessioni senza misura: 0.

## Scoreboard (pin s175 INVARIATO; micro DI RECORD)
**arith 2,4 = · prop 2,6 = · calls 4,6 = · str 4,1 = · arr 3,0 = · re 2,5 =** · giudici: prop-dq pin 36,20 = 2,57× (tree con
flag 33,47 = 2,38×) · arith-dq pin 20,28 = 2,33× (tree 19,24 = 2,21×) · WP t20 1,765 [1,738;1,799] · ORM [6,936;7,014] ·
corpus 2655 (1412 congelati) · batteria 1748/0/2 (S-175, non rilanciata) · CI: REQUEUE disk-low, lock s176 rimosso in chiusura.

## §S-177 — ordine
0. **PRE-FLIGHT**: Data ≥10G E `vm.swapusage` (swap 15-19G nel container in S-176: pesi utente Chrome/Antigravity) · MySQL wp8
   con l'elenco (se giù: `daemonize.pl <log> mysqld_safe --datadir=<mysql-wp8/data esterno> --socket=/private/tmp/mysql-wp8.sock
   --port=3306 --log-error=… --tmpdir=…`, ricetta in WP_SESSION_176/pre-flight) · lock col TOKEN `s177` · pin s175 per hash ·
   tree pulito (`git diff --quiet -- crates/`) · CI feed · Serena attiva PRIMA di toccare il Rust (hook).
1. **LEVA COMPOSTA su prop-dq** (obiettivo nominato del flag): criterio PRIMA; bracci A = pin s175, Z = gemello (ab-out/phpr-C
   b6c4b587), B = tree (flag, 6c7bbb55 = 33,47 già misurato), C = tree + leva nuova; promozione del TREE INTERO se
   D(A−C) ≥ max(4, rumore, SL) con entrambi gli stimatori e guardie senza regressione (copia dichiarata di s174-promozione.sh,
   tag s177, conferma post-pin prop-dq vs stash s175) + coppia dovuta (pair t21 copia di s175-pair.sh + s176-orm-coppia.sh E3/E4).
   Candidate per la leva nuova: (a) **fetta 4 typed-skip** su `$o->x = $o->y OP C` (BinaryTCPropSetPop): ammissibile (typed 8 %
   ORM / 0 % media) con presidio bilaterale typed; quota P1 nei driver prop-dq da leggere nel census dei micro PRIMA; (b) corpi
   di PropGetSlot/PropSetPop (residuo prop-dq 33,47 vs 14,0 = 19,5 ns/iter: dispatch 6 op ~10,5 + corpi ~9 [ipotesi]).
2. **Census IC MISS su ORM per CAUSA** (55 % delle scritture prop fuori dall'IC): contatori per causa (classe non plain / lazy
   proxy Doctrine / enum / slot assente / class-id polimorfo) nella build op-census, copia di s176-census-typed.sh; la causa
   dominante è la leva con più perimetro su ORM.
3. **calls 4,6 → str 4,1** (p.5 S-176 non eseguito): census Call/BinarySS/Ret e args-Vec sui driver calls.php/str.php e su ORM
   (copia del census), poi leva sui corpi con criterio PRIMA.
4. Coppia dovuta SOLO a pin nuovo (p.1); la voce ORM riaperta (sentinella 4,97) si chiude con la prima coppia sotto E4 (sanata: 4,94 / [4,84;5,04] / REF2 [34,73;34,81]). Azioni revisore: A/B composto a TRE bracci con PREV dello STESSO binario e anti-flare micro; coppia ORM anche sul braccio flag-solo (il flag è letto da ogni Sweep); tie-break drop-1 dichiarato; batteria+corpus sul tree PRIMA di ogni A/B che lo usi come braccio (HEAD porta il flag senza batteria: CI ferma).
5. Quesiti residui: ictx oracle1 a verbale · c0 positivo · census server (26° slitt.) · ratifiche §3 · dtor-in-dtor (catalogo) ·
   «Sweep-skip esteso» a StoreSlot/Pop/PropSetPop→Sweep (56 % dei Sweep ORM) = leva SOLO per i micro (su ORM sotto la risoluzione della coppia, lettura): non prioritaria.

## Aperture per NOME
leva composta prop-dq (flag + fetta 4 typed-skip o corpi prop) · census IC miss ORM per causa · calls → str census/leva · coppia
t21 + ORM E3/E4 (a pin nuovo) · banda sentinella ORM (E4 da eseguire) · Sweep-skip esteso (micro-only) · dtor-in-dtor · §3.32 ·
§3.30 · residuo slot 2,75/op · tupla guard · F1/F2 (SOSPESE) · autoload statiche · sonda strmap · gamba server census · §3.28 ·
§3.29 · §3.27 · §3.26 · §3.25 · §3.24+§3.23 · slot-load · §3.22 · depr. float→int · warning ×2 · div. RMW · objmap → GC ·
evalcls 316,9× · refl 42,4× · re +2 · get_gc · latin1 · dbal 10 nomi · pavimento 4 ns/iter su loop corti.

## NON riproporre (i veti restano)
**S-176: promozione di una leva a sola direzione (pavimento 4) · cifre di tempo dai census (63 ns/op è lettura, non cifra) ·
edit di un copione MENTRE gira (bash legge incrementale: copia con nome nuovo) · `cargo build` sulla target canonica per
gh-status-sync (il pin cambierebbe hash: sonda sul pin per hash) · comandi Bash che citano file sorgente Rust insieme a
cat/head/sed (hook serena-vexp li blocca in blocco) · sleep in foreground (attese in background sul `.done`).** S-175: misure
con Data <10G o senza watchdog disco · find/du durante una finestra · token phpr/php-server negli argv delle attese · rilancio
ORM senza pre-attesa anti-flare (ora E3 nel canone) · cache utente bloccate oltre la finestra · demone che patcha copioni senza
dichiarazione. S-174: commenti in coda a righe con `;` · collaudo del solo blocco python · floors in una stringa (zsh) · skill
con `model:` a metà sessione · banda-layout da ricostruzione della stessa sorgente · guardia promossa a bersaglio senza rerun
completo. S-173: attese «intatte» senza reset del dst · `perl -0pi` con `\|\|` · pulire ab-out/ durante una build · copertura
del mutante senza liste a-verdetto · cifra sotto soglia anche con 5/5. S-172: push a mano nelle catene pin-*.sh · `git archive`
senza touch · fixture single-shot con IC · `a || b && c` senza graffe. S-171: lock senza TOKEN · xctrace senza trap/Data ·
build/run durante la coppia. S-170: driver senza `[ -e ]` · path non quotati · `rm -rf` di target intere · mock unsafe.
Trasversali: NaN-boxing/fn-table/arena (⚖️) · BOLT/PGO · pin senza collaudo · rc da pipe · promozione sotto banda · cifre composte.
**Riscritto** 2026-09-16 (chiusura S-176; storia in `sessions/` · `gaps/`).
Pre-flight S-177: pin phpr **s175 5de14d6856d760a8** + server **9b9179d4dd3d95bd** (tree = pin + flag gc-idle) · **Data ≥10G +
`vm.swapusage`** · MySQL wp8 con l'elenco · uploads sotto guardia · corpus 1412 · lock col TOKEN `s177` · CI feed · lettura:
REGOLE.md → QUI → wp176-harness/revisione-s176.md → s176-flag-verdetto.out + s176-criterio-flag.md → s176-census-verdetto.out +
s176-census-typed-verdetto.out → s176-istruttoria-sentinella-orm.md + s176-criterio-orm.md → WP_SESSION_176 → gaps/GAP_TREND → PERF_MAP.
