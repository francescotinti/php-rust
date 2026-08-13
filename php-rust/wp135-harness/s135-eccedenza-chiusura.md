# S-135 — chiusura az.rev. S-134 #1+#5: l'eccedenza +101,3 è ATTRIBUITA a conteggi

Fonti: `s135-sonda-verdetto.out` (rc=0, parità stdout 8/8, conteggi
deterministici tra run) + disasm pin stashato s134 (61896da1, run_loop
0x2667b0–0x2abf60, 71.148 istr = +673 vs s133 70.475).

## (a) resolve 2→0/iter a regime — VERIFICATO

objalloc gamba B (s134): `resolve=2` ASSOLUTE su 6.000.000 scritture (=solo i
2 fill della prima passata, uno per sito; `NPfill=2`, `NPhit=5.999.998`).
objdatains: resolve 4→2/iter — restano ESATTAMENTE le 2 del cammino dim-write
(`$e->data['k']=$i`), fuori perimetro della IC di PropSet: contabilità
del residuo ri-derivata sul pin s134 (voce per la scelta leva).

## (b) ripartizione per NOME (a eventi, non a prezzi — REGOLE §4)

48/48 celle conformi alle predizioni pre-registrate. I canali dichiarati
senza prezzo nel criterio icnp p.2.3 vanno TUTTI 2→0/iter a regime:
magic-probe · hook-lookup · enum-borrow · asym · readonly. Il canale
«deprecation» (depr-contains) è 0→0: era nominato ma NON contribuiva —
falsificazione misurata di un componente dichiarato. Il canale typed resta
pagato (coerce 2=2/iter). L'eccedenza +101,3 è quindi la somma dei 5 canali
sopra + costo di sito della resolve rimossa; la magnitudine PER canale resta
non ripartita (nessun prezzo per componente, vietato).

## (c) canale layout — ESCLUSO nei termini pre-registrati

Criterio: «escluso se i conteggi B confermano i salti E il ramo hit non
contiene bl verso i canali contati». Entrambe le condizioni valgono:
1. conteggi B: eventi dei 5 canali = 2 assoluti (≡0/iter) — il lavoro
   sparisce a runtime, non è un artefatto di misura;
2. disasm del hit NP sul PIN s134: le 3 sequenze di decodifica/guardia
   (0x27bb14 · 0x284258 · 0x2842f4, siti PropSet/PropSetPop) sono PRIVE di
   `bl`; i 3 blocchi TY (0x294504 · 0x2946dc · 0x2949cc) contengono SOLO
   `bl coerce_typed_prop_write` + `bl typed_ref_assign` (canali pagati
   dichiarati). Nessun bl verso resolve/magic/hook/asym/readonly nel ramo hit.
Il +673 istr di run_loop resta churn di layout FUORI dal cammino hit (già
agli atti: nessun flip inliner, resolve 21=21).
