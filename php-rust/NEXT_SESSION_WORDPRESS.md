# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-177 = leva composta prop-dq «L-CM1» costruita a parità e misurata in finestra CONTAMINATA (rerun
cm1b in coda) + census IC miss ORM per CAUSA (private/readonly) + CI potata.** Pin INVARIATO phpr **5de14d6856d760a8** +
server **9b9179d4dd3d95bd**; **tree main (d3d3fbc5+) = pin + flag gc-idle (S-176) + L-CM1** (`Vm::ic_epoch` cached: PropIc/
MethodIc get+fill con epoch esplicita, 30 siti; Add-first in `long_arith_i64`; semantica identica SOTTO A-DS15, altrimenti più restrittiva); braccio C 62d1a2e3 a PARITÀ (fx-sw2-gc
invarianza, fx-sw1/sl1/sl2/sl3 byte-id). Fetta 4 typed-skip ESCLUSA (classe plain: P4 prende sempre; census micro typed 0).
Disasm del pin (wp177-harness/disasm-pin-run_loop-stats.txt): 6 `blr` tutti nel prologo ⇒ «TLV a ogni hit» REFUTATA; ogni hit
IC = reload TLV dallo stack + load epoch + cmp (30 siti). **A/B cm1 rc=4 SENZA VALORE**: finestra contaminata (A stesso binario
54,67 vs PREV 36,20, oracle 22,60 vs 14,07, +60 % uniforme, Chrome ×4 a 100 % col gate s129 PASS) ⇒ emende E1 (banda
same-binary |A−PREV| > 4 ⇒ rc=8) ed E2 (calma CPU totale <150 % ×4) in s177-ab-cm1b.sh / s177-lancio-cm1b.sh; **rerun cm1b
LANCIATO detached (attende la calma, tetto 90 min): esito in wp177-harness/s177-cm1b-verdetto.out + ab-out/cm1b.rc,
ab-out/lancio-cm1b.done** (rc=0 promozione · 3 direzione · 4 KILL L-CM1 (revert al byte di d3d3fbc5, flag resta) · 5 regr. ·
8 nessun verdetto). **Census miss per causa** (ramo `s177-census` c639748a; rc=0 parità): ORM 4,98M miss = **ic_empty 95,4 %**,
ic_class 4,6 %, lazy 0; esito **private_mangled 85,0 % · readonly 59,2 %** · np_nofill 10,3 % · np_fill_typed 3,8 % ·
plain_fast 0,8 %; media 975k = ic_empty 97,5 %, private_mangled 62,5 %, np_nofill 23,3 %, plain_fast 8,9 %, dynamic 2,9 %.
CI: 40 job = replay S-174 col dente storico (batteria-FAIL ATTESI), coda potata a HEAD (queue-pruned-s177); il runner
attende la rimozione del lock ⇒ batteria+corpus sul tree girano a chiusura (leggere CI_FEED in apertura). Leve: 1 · A/B: 1
(+1 rerun in coda) · incidenti: **1** (gate cieco ai pesi utente) · revisione S-177 (lente SEMANTICA): wp177-harness/revisione-s177.md.

## Scoreboard (pin s175 INVARIATO; micro NON rimisurati, riferimenti S-176)
**arith 2,4 = · prop 2,6 = · calls 4,6 = · str 4,1 = · arr 3,0 = · re 2,5 =** · giudici: prop-dq pin 36,20 = 2,57× (tree con
flag 33,47) · arith-dq pin 20,28 = 2,33× (tree 19,24) · WP t20 1,765 [1,738;1,799] · ORM [6,936;7,014] · corpus 2655 (1412
congelati) · batteria 1748/0/2 (S-175; tree NON collaudato: CI a chiusura) · CI: coda 1 (HEAD) + replay archiviato.

## §S-178 — ordine
0. **PRE-FLIGHT**: Data ≥10G E `vm.swapusage` (16-17G in S-177: Chrome/Antigravity) · **CPU utente**: `ps -Ao %cpu` somma
   <150 % prima di OGNI misura (E2: Chrome ×4 a 100 % ha contaminato cm1) · MySQL wp8 con l'elenco · lock col TOKEN `s178` ·
   pin s175 per hash · tree pulito (`git diff --quiet -- crates/`), ramo `main` (il ramo `s177-census` porta SOLO contatori
   op-census) · **CI_FEED**: esito del job 710d823c (tree + flag) e dei successivi (d3d3fbc5 = + L-CM1) · **LEGGERE
   s177-cm1b-verdetto.out / ab-out/cm1b.rc**: se rc=0 ⇒ p.1a; se 4 ⇒ `git revert` di d3d3fbc5 (L-CM1 al byte), tree = flag
   solo, p.1b; se 8/assente ⇒ rilanciare s177-lancio-cm1b.sh (token: il copione esige `s177` nel lock — rilanciare col
   lock s177 PRIMA di crearne uno s178, o copiare con token s178) · correggere il commento del lanciatore cm1b (cita `s177-criterio-cm1b.md` inesistente: è s177-criterio-cm1.md) a run finito · Serena attiva PRIMA del Rust.
1a. (cm1b rc=0) **PROMOZIONE del tree** con `PROMO_SP=/private/tmp/phpr-promo-s178 s177-promozione.sh 62d1a2e3b47a0d5a`
   (registro braccio: `scripts/pin-phpr.sh --braccio s177-cm1 wp177-harness/ab-out/s177-leva/phpr-C d3d3fbc5`) → coppia t21
   (`PIN_ATTESO/SRV_ATTESO` dal pin nuovo in s177-pair.sh: s177-lancio-pair.sh → s177-lancio-orm.sh, canone ORM E3/E4).
1b. (cm1b rc=3/4) **LEVA S-178 = fill dell'IC per gli SLOT PRIVATI nello scope dichiarante** (perimetro = TETTO 4,2M scritture
   ORM: private∩readonly ≥44 % del miss, PHPUnit 13 `readonly class` ×523 domina ⇒ PRIMA contatori private∩readonly /
   private∩¬readonly + top-10 classi (copia di s177-census-miss.sh); se PHPUnit domina, dichiararlo nel giudice ORM): oggi `prop_set_entry` riempie NP solo con `key == name`; la cella
   è già (classe, scope)-keyed (WP-35) e `write_property_at(…, Some(slot))` usa il nome solo nel fallback a indice stantio
   (impossibile a class_id combaciante) ⇒ fill con la chiave mangled ammesso nello scope dichiarante, guardie per-oggetto
   dell'hit INVARIATE; readonly (59 %) SOLO con bit RO che conserva `mark_readonly_init`/`readonly_write_error` (o fuori
   perimetro, dichiarato); VINCOLI (rilievo 6): fill solo con obj_class == scope (un figlio con layout spostato ha
   `slot: None`), fallback di `write_property_at` per KEY mangled (oggi scrive `name`), mutante «slot stantio su figlio». Giudice: nuovo micro `prop-priv` (`$this->x = $this->x + 1` in metodo, classe con `private`),
   criterio PRIMA; guardie prop-dq/arith-dq a sola regressione; fixture bilaterale NUOVA (private/protected/readonly/
   scope figlio/Closure::bind/__set/hook) + mutante «fill privato in scope sbagliato» che DEVE mordere. Coppia ORM attesa
   direzione ≤0 con quota 47 % delle scritture (SOTTO-risoluzione dichiarata finché non censita nel tempo).
2. Composizione col flag: dopo 1b, A/B a 4 bracci (A pin, Z gemello, B tree, C tree+leva) con E1/E2; promozione del tree
   intero se D(A−C) ≥ max(4, rumore, SL) su prop-dq (e prop-priv nominato).
3. calls 4,6 → str 4,1 (p.3 S-177 non eseguito): census Call/BinarySS/Ret e args-Vec sui driver e su ORM.
4. Quesiti residui: ictx oracle1 a verbale · c0 positivo · census server (27° slitt.) · ratifiche §3 · dtor-in-dtor ·
   Sweep-skip esteso (micro-only) · 59 % readonly ORM: chi scrive readonly 2,95M volte? (istruttoria breve: sonda per classe).

## Aperture per NOME
cm1b (verdetto da leggere) · promozione tree (flag+L-CM1) o revert L-CM1 · leva IC slot privati (S-178) · readonly bit RO ·
coppia t21 + ORM E3/E4 (a pin nuovo) · banda sentinella ORM (E4 da eseguire) · calls/str census · batteria+corpus del tree
(CI a chiusura) · Sweep-skip esteso (micro-only) · dtor-in-dtor · §3.32 · §3.30 · residuo slot 2,75/op · tupla guard · F1/F2
(SOSPESE) · autoload statiche · sonda strmap · gamba server census · §3.28 · §3.29 · §3.27 · §3.26 · §3.25 · §3.24+§3.23 ·
slot-load · §3.22 · depr. float→int · warning ×2 · div. RMW · objmap → GC · evalcls 316,9× · refl 42,4× · re +2 · get_gc ·
latin1 · dbal 10 nomi · pavimento 4 ns/iter su loop corti.

## NON riproporre (i veti restano)
**S-177: misure col gate s129 solo (cieco ai pesi utente: E1/E2 obbligatorie) · ipotesi di costo su TLS/thread-local senza il
disasm (LLVM issa il thunk nel prologo) · leve «fetta 4 typed-skip» su prop-dq (perimetro zero) · edit del sorgente sul
tree mentre un braccio/promozione lo usa come sorgente (ramo separato) · lanciatori con token cablato senza copia a token
nuovo · verdetti rc≠8 letti come cifra quando |A−PREV| same-binary > 4.** S-176: promozione a sola direzione · cifre di
tempo dai census · edit di un copione MENTRE gira · `cargo build` sulla target canonica · Bash con file Rust + cat/head/sed
· sleep in foreground. S-175: misure con Data <10G o senza watchdog · find/du durante una finestra · token phpr/php-server
negli argv delle attese · rilancio ORM senza anti-flare · cache bloccate oltre la finestra. S-174: commenti in coda a righe
con `;` · floors in una stringa (zsh) · skill con `model:` a metà sessione · banda-layout da ricostruzione · guardia
promossa a bersaglio senza rerun. S-173: attese «intatte» senza reset del dst · `perl -0pi` con `\|\|` · pulire ab-out/
durante una build · cifra sotto soglia anche con 5/5. S-172: push a mano nelle catene pin-*.sh · fixture single-shot con IC
· `a || b && c` senza graffe. S-171: lock senza TOKEN · xctrace senza trap/Data · build/run durante la coppia. S-170:
driver senza `[ -e ]` · path non quotati · `rm -rf` di target intere · mock unsafe. Trasversali: NaN-boxing/fn-table/arena
(⚖️) · BOLT/PGO · pin senza collaudo · rc da pipe · promozione sotto banda · cifre composte.
**Riscritto** 2026-09-17 (chiusura S-177; storia in `sessions/` · `gaps/`).
Pre-flight S-178: pin phpr **s175 5de14d6856d760a8** + server **9b9179d4dd3d95bd** (tree = pin + flag + L-CM1, salvo revert) ·
**Data ≥10G + swap + CPU utente <150 %** · MySQL wp8 con l'elenco · uploads sotto guardia · corpus 1412 · lock col TOKEN `s178`
· CI feed (job 710d823c e seguenti) · lettura: REGOLE.md → QUI → wp177-harness/s177-cm1b-verdetto.out (+ cm1) →
revisione-s177.md → s177-criterio-cm1.md → s177-census-miss-verdetto.out + s177-criterio-census-miss.md → WP_SESSION_177 →
gaps/GAP_TREND → PERF_MAP.
