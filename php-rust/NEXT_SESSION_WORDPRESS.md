# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-180 = ratifica delle sessioni diagnostiche S-178/S-179 (ChatGPT Astra 6: la leva «fill IC slot privati»
CADE su misura di tempo, ≈1,1 % della suite ORM) + revert L-CM1 al byte + dente LOC sanato (dichiarato) + RI-PIN s180 sotto toolchain
Rust 1.98.1 con TUTTI i gate rc=0 + coppia t21/ORM lanciata in catena.** **PIN NUOVO phpr `884399fc52277119` + server
`045fe03356ee73e5`** (sorgente c2b572ef: pin s175 + gc-idle S-176 + toolchain 1.98.1; L-CM1 revertata b3e48919); tree == pin.
Ratifiche: licenza **PHP-3.01** · `AGENTS.md` (mappa delle fonti, non stato) · `rust-toolchain.toml` 1.98.1 (da qui OGNI braccio è
same-toolchain col pin s180; i numeri ≤s175 restano attribuiti a 1.96.0). **Fill privato = DECLASSATO a bassa priorità su misura di tempo**
(wp179-harness/REPORT-resume.md, ricalcolo S-180 dai raw confermato 0,4176/0,4316 s: 1,795M private¬RO = 36 % dei miss, 68 % ammissibili
alle 7 guardie, cammino lento ≈0,42 s / 37 s INCLUSIVO, non tetto rigoroso; **readonly NON misurato**): NON riproporre senza una
cifra di tempo del SOLO sottoinsieme. Leve: 0 (anomalia dichiarata; **serie senza leva S-178/S-179/S-180 = 3: S-181 DEVE spedirne una**) · A/B: 0 · misure di record: micro s180 +
conferma prop-dq · incidenti: 0 · revisione S-180 (lente PROCESSO): wp180-harness/revisione-s180.md.

## Scoreboard (PIN s180, micro R=5 di record 2026-09-22 00:0x, E2 PASS t3, quiescenza t1)
**arith 2,2 ↓ · prop 2,4 ↓ · calls 4,5 ↓ · str 4,1 = · arr 2,9 ↓ · re 2,5 =** (vs s175: direzione toolchain+gc-idle, NON ripartita) ·
giudici: prop-dq pin s180 34,20 (s175 36,20: D +2,00 5/5 rumore 0,40, sola direzione) · arith-dq NON rimisurato (s175 20,28) ·
WP t21: **COPPIA IN CORSO/DA LEGGERE** (pair-out/pair180-t21.done, s180-pair-verdetto-t21.out; riferimento t20 1,765 banda
[1,738;1,799]) · ORM: **DA LEGGERE** (wp176-harness/orm-out/rimisura.done + s176-orm-coppia-verdetto.out → copiare in
wp180-harness/s180-orm-coppia-verdetto.out; riferimento [6,936;7,014]) · corpus 2655 (1412 congelati) · batteria 1748/0/2 (s180) ·
CI: coda 3 job (4c2d3c4b, bffaaf8f, c2b572ef) in attesa del lock; ATTESI verdi (dente sanato, batteria del tree rc=0).

## §S-181 — ordine
0. **PRE-FLIGHT**: pin s180 per hash (phpr 884399fc52277119, server 045fe03356ee73e5) · `rustc --version` = 1.98.1 nel
   workspace · Data ≥10G + `vm.swapusage` · **CPU totale <150 % ×4 (E2) prima di OGNI misura** · MySQL wp8 con l'elenco · lock
   col TOKEN `s181` · tree pulito · **CI_FEED**: esiti dei job 4c2d3c4b/bffaaf8f/c2b572ef (attesi OK; se batteria-FAIL leggere
   `phpr-ci/out/<sha12>/batteria.log`) · **coppia t21/ORM**: se non chiusa in S-180, leggere i `.done` PRIMA di tutto: WP fuori
   banda o ORM >7,05 ⇒ istruttoria (regola 4) prima di ogni leva · Serena attiva PRIMA del Rust.
1. **LEVA S-181 = calls 4,5× (peggior rapporto) → str 4,1×**: census Call/Ret/args-Vec/SEND sui driver micro `calls` e su ORM
   (feature op-census su ramo separato, copia di s177-census-miss.sh); criterio PRIMA; bersaglio = il CORPO del handler di chiamata
   (rotta S-169: dispatch = oracle, i corpi pesano); giudice nuovo `calls-dq` (N dal driver) + guardie prop-dq/arith-dq a sola
   regressione; A/B a 3 bracci (A pin s180, Z gemello same-toolchain, B leva) R=5 ABAB con E1 (|A−PREV| ≤4) + E2; soglia
   max(4 ns/iter, rumore, banda-layout). Promozione via copia di s180-promozione.sh (tag s181, candidato = braccio B).
2. **L-CM1 (opzionale, solo se la finestra è calma e p.1 è chiuso)**: il meccanismo (una load in meno per hit IC, Add-first) è a
   verbale in wp177-harness; ricostruire come braccio C same-toolchain sopra il pin s180 e misurare con E1/E2; attesa piccola
   (sotto 4 ns/iter probabile ⇒ solo direzione).
3. **Coppia**: dovuta a ogni pin nuovo — se t21/ORM di S-180 sono chiuse e compatibili, nessuna coppia in S-181 salvo pin nuovo.
4. Quesiti residui: readonly write-once (49 % dei miss ORM) NON MISURATO (nessuna cifra di tempo: sorte da misurare, non presunta) ·
   ictx oracle1 · c0 positivo · census server (28° slitt.) · ratifiche §3 · dtor-in-dtor · Sweep-skip esteso (micro-only) ·
   sito phprust.com ancora «MIT» (allineare nel suo progetto) · gh-status-sync a mano (skill con `model:` — solo a inizio sessione) ·
   **licenza PHP-3.01 ratificata col testo integrale: clausole 4 («PHP» nel nome del prodotto: php-rust/phpr) e 6 non adattate —
   DECISIONE UTENTE richiesta** (rilievo 8) · copia S-181 della catena: HEAD + `git status --porcelain` PRIMA della build, `hk.rc`
   nel verdetto (rilievi 3/4) · purge `._*` in wp180-harness/{promo,pair,orm}-out a coppia chiusa (rilievo 9).

## Aperture per NOME
coppia t21 + ORM E3/E4 (in corso/da leggere) · CI 3 job (attesi verdi) · leva calls (S-181) · str 4,1 · L-CM1 come braccio C
same-toolchain (opzionale) · readonly write-once senza cifra · arith-dq da rimisurare sul pin s180 · Sweep-skip esteso (micro-only) ·
dtor-in-dtor · §3.32 · §3.30 · residuo slot 2,75/op · tupla guard · F1/F2 (SOSPESE) · autoload statiche · sonda strmap · gamba
server census · §3.28 · §3.29 · §3.27 · §3.26 · §3.25 · §3.24+§3.23 · slot-load · §3.22 · depr. float→int · warning ×2 · div. RMW ·
objmap → GC · evalcls 316,9× · refl 42,4× · re +2 · get_gc · latin1 · dbal 10 nomi · pavimento 4 ns/iter su loop corti.

## NON riproporre (i veti restano)
**S-180: chiudere un fronte su una misura singola altrui (S-179 = tetto inclusivo: «bassa priorità», non «caduta») · leve scelte su census di FREQUENZA senza cifra di TEMPO del cammino (S-179 ha ridotto «47 % delle scritture» a ≈1 % della
suite) · fill IC privato/readonly senza una misura di tempo nuova · A/B braccio-vs-pin s175 (toolchain diversa: solo direzione) ·
cambio di toolchain o ricetta a metà arco di misura · tenere nel tree una leva senza verdetto oltre due sessioni · file `._*`
AppleDouble in phpr-ci (uccidono il runner) · daemonize.pl senza `mkdir -p` della cartella del log · `.rs` nel testo di un comando
git (hook).** S-177: misure col gate s129 solo (E1/E2 obbligatorie) · ipotesi di costo su TLS senza disasm · fetta 4 typed-skip ·
edit del sorgente sul tree mentre un braccio lo usa · lanciatori con token cablato senza copia · verdetti rc≠8 letti come cifra con
|A−PREV| >4. S-176: promozione a sola direzione · cifre di tempo dai census · edit di un copione MENTRE gira · `cargo build` sulla
target canonica fuori catena · sleep in foreground. S-175: misure con Data <10G o senza watchdog · token phpr/php-server negli argv
delle attese · cache bloccate oltre la finestra. S-174: skill con `model:` a metà sessione · banda-layout da ricostruzione. S-173..170:
attese senza reset del dst · `a || b && c` senza graffe · lock senza TOKEN · `rm -rf` di target intere · mock unsafe. Trasversali:
NaN-boxing/fn-table/arena (⚖️) · BOLT/PGO · pin senza collaudo · rc da pipe · promozione sotto banda · cifre composte.
**Riscritto** 2026-09-22 (chiusura S-180; storia in `sessions/` · `gaps/`).
Pre-flight S-181: pin phpr **s180 884399fc52277119** + server **045fe03356ee73e5** (tree == pin) · toolchain 1.98.1 · **Data ≥10G +
swap + E2 CPU totale <150 %** · MySQL wp8 con l'elenco · uploads sotto guardia · corpus 1412 · lock col TOKEN `s181` · CI feed ·
lettura: REGOLE.md → QUI → wp180-harness/s180-promo-verdetto.out → s180-pair-verdetto-t21.out (+ ORM) → revisione-s180.md →
s180-criterio-ripin.md → wp179-harness/REPORT-resume.md (perché il privato è caduto) → WP_SESSION_180 → gaps/GAP_TREND → PERF_MAP.
