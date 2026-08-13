# WP_SESSION_135 — LEVA AP1 fast-path SPEDITA (pin s135) + eccedenza S-134 attribuita + rimisura dbal/ORM

**In una frase**: scrivere in un array (`$a[k] = v`, l'operazione più comune di
WordPress e Doctrine) ora salta un lungo giro di corridoio interno — il caso
tipico è ~32% più veloce (objmap 17,3→11,7) — e i sospesi del revisore S-134
sono stati chiusi con misure, non con dichiarazioni.

**SCOREBOARD** (pin NUOVO **s135 6518a1e1**4a266d52 + server s135 e2efdf15; micro gate promozione):
**arith 5,4 = · prop 5,5 = · calls 4,9 ↘ · str 4,2 = · arr 3,2 = · re 2,5 =**
· rif WP resta **1,769 on-only @ pin s134** (non rimisurato; coppia N≥3 DOVUTA
in S-136 sul pin s135) · **leve perf spedite: 1 (AP1)** · incidenti: 0 nuovi.

## Esiti secchi
1·**Az.rev. S-134 5/5**: sonda conteggi **48/48 conforme** (resolve 2→0/iter;
  eccedenza +101,3 attribuita per NOME ai 5 canali 2→0; depr 0→0 = componente
  FALSIFICATO; layout escluso col disasm del ramo hit) · UB falsificabile e
  banda submicro↔A/B pre-registrate e RISPETTATE nella leva di oggi ·
  pin-phpr.sh scrive il registro fail-closed (primo uso vivo in promozione).
2·**Rimisura dbal/ORM @ s134**: dbal 8,36–8,45 · ORM 8,43–8,56 net, parità ok
  — **REPERTO: rapporti FERMI** (phpr −4/−5% assoluto, oracle −3/−4% drift):
  6 leve object non muovono le suite ⇒ leva scelta sul profilo, non sul micro.
3·**LEVA AP1 SPEDITA** (istruttoria in 2 stadi, criteri PRIMA): bisezione
  objmap → dominante = macchineria dim-set (183,3 = 77%); modello tempo
  AssignPath (arm 66,8 · path_op 78% · walk-plumbing 38,4 · chiusura 86%
  INCOMPLETO dichiarato) → fast-path 1-chiave/Array. Fedeltà: fixture 14
  sezioni BYTE-ID al pin. A/B r1 rc=5 AGLI ATTI (guardia objalloc, banda
  sotto-fondata) → emenda rev. S-112 (guardie = formula del giudice) → r2
  **objmap D=+56,7 ≤ UB 57,7**, riconc. smoke 6,7≤10, guardie 9/9 →
  promozione rc=0 → **pin s135**. Submicro: **objmap 11,7** (riconc. A/B
  0,1≤20) · objchurn 7,0 (collaterale) · allocni 8,1 (+13,3, osservazione).
4·CI: FAIL `dbff54a` = infrastrutturale (SIGTERM disk-low) · lock misura
  rimosso dal trap EXIT della rimisura → finestra scoperta (smoke1 rc=1 agli atti).
## ⭐ Lezioni (max 3)
- ⭐⭐ Una banda di guardia presa da uno strumento DIVERSO (spread submicro) può
  essere più stretta del rumore vivo dell'A/B: la guardia si fonda anche sul
  drop-1 del run stesso (formula del giudice), e l'emenda si fa rieseguendo.
- ⭐⭐ La UB falsificabile da prezzi MISURATI (modello del tempo) tiene: D=56,7
  dentro 47,7+10 — prima leva della serie senza eccedenza non attribuita.
- ⭐ Un lock condiviso non si affida a un trap EXIT altrui: la riserva di
  finestra è di chi la tiene (ri-creare il lock dopo ogni orchestratore).
