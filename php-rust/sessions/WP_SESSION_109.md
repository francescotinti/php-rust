# WP_SESSION_109 — S-109: coppia WP al minimo storico (1,842), server regradato POST-lotti, LOTTO-3 str (6,2→5,3)

**In una frase**: la suite WordPress completa conferma che il codice aggiunto
nei lotti non rallenta il motore (full 1,842×, il miglior rapporto mai
registrato), il server è stato ri-collaudato a pieno grado, e un terzo lotto
di ottimizzazioni ha reso le operazioni sulle stringhe il 15% più veloci —
con tutti i collaudi (batteria, corpus, fixture, doctrine, symfony) verdi.

**SCOREBOARD** (micro R=5 sul pin finale 92909544, N emessi):

| giudice | S-108 | S-109 | trend |
|---|---|---|---|
| **aritmetica** | 9,4 | 9,3 | ~ (banda tra-sere) |
| **proprietà** | 8,0 | 7,9 | ~ |
| **chiamate** | 5,3 | 5,1 | ~↓ |
| **stringhe** | 6,2 | **5,3** | ↓↓ (lotto-3; A/B Δ=+37,5 5/5 SOPRA soglia, −15% sul giudice) |
| **array** | 3,9 | 3,9 | = |
| **regex** | 3,5 | 3,5 | = |

WordPress: **full ON 1,842× / OFF 1,911× RIMISURATI OGGI** (ON = minimo
storico, sotto banda [1,86;1,93]: debito icache lotto-2 ASSOLTO) · media
2,707/2,634 (off AL riferimento 2,64: la voce aperta perde monotonia, resta
aperta solo-direzione) · parità per NOME ×2 (solo wp_is_stream) · full peak
phpr 1845,49 MiB (minimo osservato). **Leve perf spedite: 1** (lotto-3).
run_loop 287.944 B (−976). Contatore sessioni-senza-Δ-rapporti: 0.

**Data**: 2026-08-07 (18:1x–20:5x). **Modello**: Fable 5. **Commit**: 5b7ea5b→(chiusura) pushati.

## Esiti secchi
1·coppia bimodale VERDE ×2 (criterio PRE 5b7ea5b) → 2·server **443ae42f GRADATO PIENO ×2 POST-lotti** (sentinella PASS, option 413 + restapi 3508 per NOME; debito «dde2a64d PRE-lotti» chiuso) → 3·census TERZO giro (str unico giudice non toccato: 2 bigrammi fusabili) → criterio lotto-3 PRE (6534584) → F1 Neg-fold (consts-append) + F2 ConcatNConst (helper condiviso) → admission v2 PASS ({main}-only: emendamento v1 DICHIARATO — il dump copre il prelude) → **A/B str +37,5 ns/iter 5/5 PROMOSSA** (245→206) → **PIN S-109 = 929095448e823cb5** (batteria 1742/0 rc=0; corpus 1415×2 IDENTICO; fixture 6/6; ORM 16 nomi = baseline ESATTA; hk 0E/0F; micro R=5) → azioni revisore S-108 SALDATE ×4 (guardia W13 estesa allo specchio + dente + fixture w9a/w9b VERDI con probe di finestra). 🔵 NUOVE per NOME: **§3.16** (riga errata warning undef-var del ricevitore prop-assign, bilaterale, esposta da w9a caso B, parcheggiata) · **xctrace ASSENTE** sulla macchina (solo CLT) ⇒ contatori L1I infattibili senza Xcode (prerequisito tooling della tesi threaded). Incidente sfiorato: rc-da-pipe INTERCETTATO al lancio batteria (4ª morsicatura evitata); 1° giro batteria annullato (test lotto3 con builtin host nel run() di batteria).

## ⭐ Lezioni (max 3)
- ⭐⭐ **Il perimetro di un giudice d'emissione è il {main}, non il dump intero**: il prelude fonde legittimamente in OGNI giudice — un'admission «i cinque restano identici» sul dump intero è falsa per costruzione (emendamento v1→v2 dichiarato).
- ⭐⭐ **Una fixture nuova è anche un detector**: w9a caso B non collaudava la finestra, ha SCOPERTO §3.16 (bilaterale, pre-esistente). La via giusta è catalogo per NOME + parcheggio, non forzare il gate.
- ⭐ **W13 assorbe il PushConst di un Neg dopo LoadVar foldabile** (F1 non lo vede; valore identico, Neg resta a runtime): le finestre nuove vanno pensate CONTRO l'ordine delle esistenti — il test del lotto deve replicare la forma del giudice, non una forma comoda.
