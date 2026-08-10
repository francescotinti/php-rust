# s127-criterio-ab.md — A/B L-OL1-F1 «stampo» (ri-commit del criterio s126-leva-nominata.md CON la forma) — PRE-registrato

1. Forma: template Props per classe (`OnceCell` su CompiledClass; alloc = template.clone(), default COW). Ammissione della forma PRIMA dell'A/B: arbitro census `s127-admission.sh` rieseguito sul binario-leva — predizione alloc/free/iter **−1,00 ESATTO** (tol 0,05) su objalloc/objallocni/objdatains/objdropdef/objchurn, **0,00** su objmap; parità stdout col pin su tutte; bl-count `alloc_object` in calo (disasm a verbale). Fuori predizione ⇒ diagnosi PRIMA dell'A/B.
2. Giudice A/B: `wp127-harness/micro-orm/objalloc.php` (N=3.000.000 emesso dal sorgente), R=5 ABAB interleaved (pin s125 002e6cc1 vs leva), user CPU netto-pavimento PER-binario, timer /usr/bin/time -p.
3. Segno: phpr objalloc ns/iter DIMINUISCE. Soglia: max(4 ns/iter, rumore R=5 = spread della gamba leva, SL prop = max(0,80 s123; 0,40 s125) = 0,80). Smoke R=2 con early-stop a segno opposto, lettore proprio dell'arbitro (famiglia = 1 categoria, MINFAM rispettato).
4. Guardie NON-bersaglio a SOLO-REGRESSIONE (stesse run R=5): le sei micro storiche (arith prop calls str arr re, giudici `wp97-harness/micro/`) + objchurn/objmap; soglia guardia per categoria = max(4 ns/iter, SL s123/s125 della categoria); morde solo il PEGGIORAMENTO.
5. Gate promozione (tutti, sul candidato): batteria `cargo test --release` (rc dal comando) · corpus fail-set CONGELATO per NOME ×2 modi (1415) · fixture bilaterali · ORM 16 nomi == baseline · hk 0E/0F · pin SOLO via `scripts/pin-phpr.sh` (collaudo-nell'atto).
6. Nessuna predizione di magnitudine oltre il segno (prima leva su questa categoria); il profilo indica gruppi spalmati — l'A/B decide.

## EMENDA p.4 (dopo smoke-1, PRIMA di ogni nuova esecuzione — REGOLE §3/§5)
La prima esecuzione (s127-smoke-verdetto.out, a verbale) ha morso arr/re per
METRICA, non per regressione: i giudici `wp97-harness/micro/` hanno file annidati
(arr: N letto 100k, iterazioni vere 6M ⇒ 35 ms di rumore letti come −350 ns/iter)
e pavimenti che pesano 10-60× per iter su N corti, mentre le bande SL provengono
dai giudici SCALATI. Guardia RI-COLLOCATA dove il sito sopravvive: le sei storiche
girano sui giudici `wp123-harness/scaled/` (N fissi nell'arbitro: arith 150M,
prop 90M, calls 60M, str 28M, arr 30M, re 12M — i file nativi delle bande SL
s123/s125). Giudice e guardie objchurn/objmap invariati (loop piatti, N dal
sorgente). Smoke R=2 rieseguito col criterio emendato prima dell'A/B R=5.
