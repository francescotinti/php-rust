# Revisione S-170 — lente MISURA (revisore singolo)

**VERDETTO: REGGE CON RILIEVI** — le cifre dirette (m9 +5,64; m12 +17,16; m13 +22,20; dq 2,84×) sono A/B interni con A stabile (e2A 3,67-3,69 s su 30 run: nessun bias termico), crates identici 6845173c..1c929f33, bl run_loop 6032..6038; ma il fast path non è provato, i bracci non sono additivi, il tetto è modellato sul driver.

## Rilievi
1. **Fast path provato solo per magnitudine** (s170-m12/m13.patch: guardia tupla + fallback al corpo originale). Output = atteso non distingue ramo veloce da lento; nessun contatore/abort. Per m8 non si sa se ENTRAMBI i handler colpiscono il Long.
2. **Non additività**: m11 store +1,20 e bounds e2 (m9−m8) +1,56, ma m13−m12 = +5,04 (s170-verdetto-b.out r.7): nel corpo grasso la latenza di store/bounds è nascosta dall'OoO, nel magro emerge. «Store sotto pavimento» vale in m0, NON nella forma sigillata, che DEVE includerlo.
3. **Cifre da contrasto** (REGOLE §3): D_m12−D_m123 = 9,16 confronta bracci di sessioni diverse; «corpo fuso ≈5,07» e «residuo 2,75/op» sottraggono Sweep 2,9 e dispatch 1,75 di S-169; l'estrapolazione 2,19× somma D di due giudici. Dichiarate, ma sono direzione, non magnitudine.
4. **m8 a 2 tick**: +4,08 con tick 0,04 (`time -p`); drop-1 su 5 valori quantizzati (2,65/2,66) sottostima; A deriva 14,64↔14,72 fra sessioni. Il claim sui banali regge su m9, non su m8. N=250e6 cablato nel giudice (s170-ab-mock.sh r.59), non «emesso dal sorgente».
5. **Tetto modellato sul driver**: la tupla (Mul,Shr,Sub,Add) è cotta nel patch ⇒ m12/m13 incorporano gratis il «BinOp cotto» di m2/m7 (5,2-5,6). Una leva generica paga un match per op o pretende opcode specializzati: 22,20 è limite superiore, atteso ≈17.
6. **xctrace**: c2 delivery 15,8 vs 0,84 ns/iter misurata sul PIN, mai su m13. Se il −22 è front-end (footprint) la leva deve tenere il ramo lento fuori linea o non riprodurrà il mock.

## Azioni S-171
1. Mutante «ramo lento abortivo»: m13 (e m8) con fallback → `unreachable!()`; output = atteso ⇒ fast path esclusivo provato.
2. xctrace m0 vs m13 su dq: la colonna che cala (c0 o c2) fissa c2 senza circolarità: istruzioni o footprint.
3. Leva «forma sigillata Long» come codice: opcode specializzato, slow path `#[cold]` outlined, store in place; soglia contro m13 (tetto) oltre che contro m0.
4. Rimisurare m8 a N=1G (tick 0,01); N letto dal driver nel giudice.
5. Nessuna cifra composta (Sweep/dispatch/m123) nel criterio S-171: solo D interni al run.
