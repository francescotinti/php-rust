# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-181 = leva L-CR1 «Call/Ret magri» PROMOSSA, PIN NUOVO phpr `19a2faa83a492745` + server `b2802f08c5887e77`**
(sorgente leva 172814ae, HEAD alla build f5f746cf, tree == pin; toolchain 1.98.1). Micro s181: **arith 2,1 · prop 2,3 · calls 4,0 · str 4,1 · arr 2,8 · re 2,6**
(calls 4,5→4,0 = leva; il resto tick/trasversale). **calls-dq** (giudice nuovo, N=60M): 98,83→90,83 ns/iter (oracle 21,67; 4,56×→4,19×),
CIFRA [+8,00;+8,17] (cr1c di record; cr1/cr1b repliche concordi a guardia per un campione di GoogleUpdater), conferma post-pin +8,00 5/5
su layout diverso. Guardie: **prop-dq +2,40/+2,47 (5/5, replicata post-pin)** e arith-dq +0,72: effetto TRASVERSALE deterministico
(sp_refs di run_loop −8 %), quota del meccanismo NON ripartita (rilievi 1-4 revisione S-181). Leve: 1 (PROMOSSA) · A/B: 3 corse ·
incidenti: 1 (lanciatore cr1b con nome del copione errato, rc=127 senza misura) · revisione S-181 (lente MISURA): wp181-harness/revisione-s181.md.
**Coppia t22 + ORM E3/E4: IN CORSO alla chiusura** (catena s181-lancio-coppia.sh partita 15:34): se non lette in S-181, leggere PRIMA di tutto
`wp181-harness/pair-out/pair181-t22.done` + `s181-pair-verdetto-t22.out` e `wp176-harness/s176-orm-coppia-verdetto.out` (copiarlo in wp181 come S-180).

## Scoreboard (PIN s181, micro R=5 di record 2026-09-22 15:2x, E2 PASS t8, quiescenza t3)
**arith 2,1 ↓ · prop 2,3 ↓ · calls 4,0 ↓↓ · str 4,1 = · arr 2,8 ↓ · re 2,6 ↑** (vs s180 2,2/2,4/4,5/4,1/2,9/2,5) · giudici sul pin s181: calls-dq 90,50
(stash s180 98,50) · prop-dq 31,60 (s180 34,07) · arith-dq B 18,24 (A 18,96, cr1c) · **WP t22: in corso** (t21 1,744, banda [1,738;1,799]) ·
**ORM: in corso** (t21 [7,008;7,060], leg2 a filo di 7,05: regola 4) · corpus 2655 (1412 congelati) · batteria 1748/0/2 (s181, denti dichiarati) ·
CI: job in coda da 172814ae a HEAD partono al rilascio del lock (ATTESI verdi: batteria del tree rc=0, cap LOC run.rs 7404 dichiarato).

## §S-182 — ordine
0. **PRE-FLIGHT**: pin s181 per hash (phpr 19a2faa83a492745, server b2802f08c5887e77) · rustc 1.98.1 · Data ≥10G + swap (S-181: 11G→5G
   in batteria per lo swap; cache Google 1,5G purgata) · **CPU totale <150 % ×4 prima di OGNI misura (E2: l'IDE + questo processo
   pesano 70-80 %: silenzio durante le finestre)** · MySQL wp8 · lock col TOKEN `s182` · tree pulito · CI_FEED (job da 172814ae) ·
   **coppia t22/ORM**: se non chiuse in S-181 leggere i `.done`; WP fuori banda ⇒ istruttoria; **ORM: se leg2 resta >7,05 a t22 senza che
   Δ_norm lo spieghi ⇒ ISTRUTTORIA (regola 4) PRIMA di ogni leva** · Serena attiva PRIMA del Rust · purge `._*` in wp181-harness/*-out.
1. **SCOMPOSIZIONE L-CR1 (az.rev. S-181, rilievi 1-4)**: bracci same-toolchain sopra il pin s181: P = placebo (modifica innocua in
   run_loop: misura la banda-layout del CANDIDATO), −a (senza split_at_mut/push diretto), −b (senza ip=1), −c (guardia Ret com'era);
   giudice calls-dq + prop-dq/arith-dq con ESITO ESPLICITO dell'attesa |D|<1 nello script (non solo regressione); attesa: (b) ≈ 3-4
   ns (un dispatch + handler), (a) ≈ 0-2 (il disasm dice che il Frame viaggia ancora per valore), trasversale ≈ 2,5 su prop-dq.
   Mutante ip=1 anche sul cammio lento di methodcall (rilievo 7). Riscrivere il meccanismo (a) a verbale con le cifre.
2. **LEVA S-182 = Frame davvero in place o CheckArity a compile-time**: (i) `CheckArity` eliminato dal bytecode per TUTTE le forme
   (bind_params/binder controllano l'arità: ORM 12,9M op, media 46k — oggi saltato solo sui fast path); (ii) Ret senza copia del
   Frame (take dei buffer in place + truncate: ordine di drop IDENTICO da provare con fx-sw2-gc/fx-cr1 + mutante); disasm bl/sp_refs
   prima/dopo. Criterio PRIMA; A/B a 3 bracci R=5 con E1 (PREV same-binary: calls-dq 90,50 · prop-dq 31,60 sul pin s181) + E2 + E3.
3. **str 4,1×** = seconda categoria peggiore (census str: concat/interpolazione, quota Zval::Str clone/alloc) — dopo calls.
4. **Coppia**: dovuta a ogni pin nuovo (t22 di S-181 in corso; t23 SOLO con pin nuovo).
5. Quesiti residui: readonly write-once NON MISURATO (49 % dei miss ORM) · ictx oracle1 · c0 positivo · census server (29° slitt.) ·
   ratifiche §3 · dtor-in-dtor · Sweep-skip esteso · sito phprust.com «MIT» · gh-status-sync a mano (skill con `model:`) ·
   **licenza PHP-3.01 clausole 4/6 non adattate — DECISIONE UTENTE richiesta** (rilievo 8 S-180, ancora aperto) · E3 (updater ≥2 campioni)
   da tenere o togliere nel lanciatore-modello (rilievo 5) · divergenza dtor del locale al ritorno = famiglia §3.28 (ii) (osservata in fx-cr1).

## Aperture per NOME
**VOCE ORM [7,008;7,060] a filo di 7,05 (regola 4 a t22)** · quota trasversale di L-CR1 (prop-dq +2,47 non ripartita) · meccanismo (a) vs disasm (bl +9) ·
CheckArity a compile-time · Frame in place al Ret · str 4,1 · readonly write-once senza cifra · voce rete Tests_Fonts (t21 leg4) · Sweep-skip
esteso · dtor-in-dtor · §3.32 · §3.30 · §3.28 (ii) dtor locale al ritorno · residuo slot 2,75/op · tupla guard · F1/F2 (SOSPESE) · autoload statiche ·
sonda strmap · gamba server census · §3.29 · §3.27 · §3.26 · §3.25 · §3.24+§3.23 · slot-load · §3.22 · depr. float→int · warning ×2 · div. RMW ·
objmap → GC · evalcls 316,9× · refl 42,4× · re +2 · get_gc · latin1 · dbal 10 nomi · pavimento 4 ns/iter su loop corti.

## NON riproporre (i veti restano)
**S-181: cifra di una leva composta senza scomposizione né placebo (la cifra è «binario vs pin») · guardie a sola regressione con attesa
pre-registrata muta · meccanismo dichiarato senza il disasm che lo conferma (bl atteso −1, misurato +9) · copie di lanciatori con sed
globale sul tag (ha rinominato il copione invocato: rc=127) · limiti cablati nei patch dei mutanti (4000 caratteri) · finestre senza E3 con
updater periodico (un campione da 30 s ⇒ guardia) · misure con l'IDE attivo sopra 150 % totale.** S-180: chiudere un fronte su una misura
singola altrui · leve scelte su census di FREQUENZA senza cifra di TEMPO · fill IC privato/readonly senza misura di tempo nuova · A/B
braccio-vs-pin s175 · cambio di toolchain a metà arco · leva nel tree senza verdetto oltre due sessioni · `._*` in phpr-ci · daemonize senza
`mkdir -p` · `.rs` nel testo di un comando git. S-177: misure col gate s129 solo · ipotesi TLS senza disasm · edit del sorgente sul tree
mentre un braccio lo usa · verdetti rc≠8 con |A−PREV| >4. S-176..170: promozione a sola direzione · cifre da census · edit di un copione
mentre gira · build sulla target canonica fuori catena · sleep in foreground · Data <10G senza watchdog · token phpr/php-server negli argv
delle attese · skill con `model:` a metà sessione · lock senza TOKEN · rm -rf di target intere. Trasversali: NaN-boxing/fn-table/arena (⚖️) ·
BOLT/PGO · pin senza collaudo · rc da pipe · promozione sotto banda · cifre composte.
**Riscritto** 2026-09-22 (chiusura S-181; storia in `sessions/` · `gaps/`).
Pre-flight S-182: pin phpr **s181 19a2faa83a492745** + server **b2802f08c5887e77** (tree == pin) · toolchain 1.98.1 · Data ≥10G + swap + E2 ·
MySQL wp8 · uploads sotto guardia · corpus 1412 · lock col TOKEN `s182` · CI feed · lettura: REGOLE.md → QUI → wp181-harness/s181-cr1c-verdetto.out
→ s181-promo-verdetto.out → revisione-s181.md → s181-criterio-cr1.md → (pair t22 + ORM) → WP_SESSION_181 → gaps/GAP_TREND → PERF_MAP.
