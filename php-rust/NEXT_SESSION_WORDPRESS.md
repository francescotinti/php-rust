# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-170 = DELIBERA R4 dell'utente = (i) campagna «corpo del
handler», eseguita come SOLA MISURA con mock magri su ENTRAMBI i giudici: handler
banali (e2) m8 +4,08 (a filo) / m9 +5,64 ⇒ per-op 7,32→4,50 vs 1,76; handler fuso
BinarySCSCDst (dq) m12 funnel i64 +17,16 / m13 corpo magro totale +22,20 ⇒ dq 46,8→24,6
= 2,84× (pin 5,4×), corpo fuso 27,5→5,1; guardie (+1,44) e store (+1,20) da soli NON
pesano: pesa la FORMA (Zval temporanei, funnel Option<Zval>, clone/drop, bounds).
Kill pre-registrati NON scattati, soglia alta (≥20) superata ⇒ la «forma sigillata
Long» (regola 8) è la PROSSIMA LEVA DA MISURARE COME CODICE** · xctrace-3: c0
positivo fallito 2×, c0 per esclusione, c2 delivery 15,8 vs 0,84 ns/iter indiziata ·
leve: 0 (sanzionato) · incidenti: 2 (pavimenti S-167..169 su file inesistente, ≤0,02;
target gads-mcp cancellata) + 4 difetti di copione curati in corsa · revisione S-170:
lente MISURA (vedi wp170-harness/revisione.md) · sessioni senza misura: 0.

## Scoreboard (pin INVARIATO s166 phpr 092dcff431bef876 + server caa4e4b2638686a9)
**arith 5,4 · prop 5,5 · calls 4,8 · str 4,2 · arr 3,2 · re 2,5** · mc2 ~155 /
mc3 181 · arith-dq 46,6 vs 8,64 (mock m13 24,6) · E2 14,64 vs 3,3-3,6 (mock m9 9,00)
· dispatch 1,75/op · WP 1,746-1,749 · ORM [7,023;7,053] (RIF) · corpus 1412×2 ·
batteria 1748 · denti: run.rs 6917 · mod.rs 25909 · host.rs 7726 · coda CI: 1 (HEAD).

## §S-171 — ordine
1. **LEVA CODICE «forma sigillata Long», fetta 1** (prima leva vera dopo 3 sessioni di
   sola misura): criterio PRIMA (≤10 righe, soglia max(4, rumore, banda-layout),
   giudici arith-dq E arith-e2, R=5, guardie non-bersaglio a sola regressione);
   (a) BinarySCSCDst: catena i64 con guardia tupla, fallback al corpo esatto su
   overflow/shift/tipi (forma m12, SAFE, niente get_unchecked); (b) CmpJmpSC +
   IncDecSlotJmp: cammino Long senza to_zval/guardia ridondante/Zval::Bool,
   `checked_add` con fallback ESATTO (forma m8 semantica-preservante). Attesa dai mock (tetto
   m13 = limite superiore, revisione S-170): dq −17..−22 + e2 −4 ⇒ arith-dq ≈ 25-30
   ⇒ ~2,9-3,4×: la TAPPA ≤3× su arith è possibile, NON garantita dalla fetta 1.
   Gate pieni: batteria, corpus 1412 per NOME ×2, fixture bilaterali, micro R=5,
   **coppia WP+ORM DOVUTA al pin nuovo**; disasm bl run_loop prima/dopo.
2. **Se (1) nominata e promossa**: piano di generalizzazione a forme (census S-164:
   quali handler caldi delle categorie prop/calls/str hanno la stessa forma Zval-temp
   + funnel) — misura per categoria, non aggregato.
3. Az.rev. revisione S-170 (lente misura, REGGE CON RILIEVI): (a) xctrace m0 vs
   m13 su dq — la colonna che cala (c0 o c2) fissa c2 senza circolarità; (b) leva
   con slow path `#[cold]` outlined e store in place (non additività: m13−m12 +5,04);
   (c) soglia della leva contro m0 E contro m13 (tetto = limite superiore: BinOp
   cotto incluso, atteso generico ≈17 ⇒ ~3,0-3,4×: tappa NON garantita); (d) m8 a
   N=1G (tick 0,01), N letto dal driver nel giudice; (e) nessuna cifra composta
   (Sweep/dispatch/m123) nel criterio: solo D interni al run.
4. c0/c2: mutante di DELIVERY non circolare (corpo gonfio vs magro su E2) prima di
   contare c2.
5. Quesiti residui: (b) T2/A2; (c) census server (20° slitt.); (d) ratifiche §3.

## Aperture per NOME
forma sigillata Long (fetta 1: BinarySCSCDst + CmpJmpSC/IncDecSlotJmp) ·
generalizzazione a forme · residuo 2,75/op accesso slot (doppia indirezione
Vec→Frame→Vec: misurabile solo con cambio di forma) · tupla guard 0,4 · mutante
delivery · c0 positivo (kernel ILP-ricco) · Sweep/iter 2,9 · F1/F2 (SOSPESE: tetto
1,75/op) · tetto-fuso · autoload statiche · sonda strmap · banda sentinella ORM · gamba
server census (20°) · §3.28 · §3.29 · §3.27 · §3.26 · §3.25 · §3.24+§3.23 ·
slot-load · §3.22 · depr. float→int · warning ×2 · div. RMW · objmap → GC · evalcls
316,9× · refl 42,4× · re +2 · get_gc · latin1 · dbal 10 nomi.

## NON riproporre (i veti restano)
**S-170: driver o pavimento senza `[ -e ]`/`[ -s ]` E senza collaudo contro l'atteso
· patch da `git diff` senza `--relative`/path check (git root ≠ repo dir) · wrapper
con path non quotati · `rm -rf` di una target Rust intera (tenere i binari) · mutante
c0 a catena seriale (crc32, array_sum) · promuovere un mock unsafe (m9/m13 = misura).**
S-169: A==B senza atteso · copia-gate per riga · xctrace senza purge ktrace · sessione
senza pre-flight. S-168: mock senza dump · catena sotto timeout tool · output di run
nel repo · soglia che scala col rumore · kill senza banda. Trasversali:
NaN-boxing/fn-table/arena (⚖️) · BOLT/PGO · pin senza collaudo · rc da pipe ·
promozione sotto banda.
**Riscritto** 2026-09-07 notte (chiusura S-170; storia in `sessions/` · `gaps/`).
Pre-flight S-171: pin phpr **s166 092dcff4**31bef876 + server **caa4e4b2**638686a9
(SOLO via pin-*.sh; stash bracci `phpr-s168-*`/`phpr-s169-*`/`phpr-s170-*` NON pin) ·
Data ≥10G (oggi 20G; CI target + build canonica ~10G: controllare `$DARWIN_USER_TEMP_
DIR/instruments*.ktrace` e CI_FEED) · MySQL wp8 con l'elenco · uploads sotto guardia ·
corpus 1412 · lock misura da CREARE (oggi RIMOSSO in chiusura) · **coppia DOVUTA se il
pin cambia** · CI: runner rilanciato su HEAD, leggere il feed · lettura: REGOLE.md → QUI
→ wp170-harness/s170-verdetto.out + s170-verdetto-b.out + revisione.md →
s170-criterio.md/-b.md (forme dei mock = specifica della leva) → WP_SESSION_170 → PERF_MAP.
