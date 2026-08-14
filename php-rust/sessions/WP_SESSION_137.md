# WP_SESSION_137 — coppia WP sul pin s136 (obbligo assolto) + sonda FD1 NON CHIUSA + objmap ATTRIBUITO (leva 0 DICHIARATA)

**In una frase**: la verifica completa di WordPress rifatta tre volte sul motore
nuovo conferma il rapporto (~1,77–1,78: la leva FD1 non ha rotto nulla), e le due
indagini di misura hanno inchiodato dove NON si guadagna a buon mercato — l'eccedenza
FD1 resta non ripartita (colpa dell'attrezzo di misura, dichiarato) e il sovrapprezzo
degli oggetti-in-mappa è un giro completo del garbage collector a ogni statement,
curabile solo dal piano GC a roadmap.

**SCOREBOARD** (pin INVARIATO s136 1e14793e + server 91c4e043; micro dal gate S-136, NON rimisurate):
**arith 5,5 = · prop 5,6 = · calls 4,7 = · str 4,2 = · arr 3,3 = · re 2,6 =** ·
**rif WP full ON-ONLY = 1,767–1,781 (N=3 @ pin s136, COMPATIBILE col rif S-136
1,777–1,779 su banda off 0,041**; off 1,805 N=2; media 2,445–2,529; peak 1743–1819
RIENTRATO) · **leve perf spedite: 0 — ANOMALIA DICHIARATA** (3 ragioni per NOME in
s137-istruttoria-objmap.md p.5) · incidenti: 0 nuovi (storici 13).

## Esiti secchi
1·**Coppia t1 rc=0 al primo colpo** (quiescenza 7/7 senza retry: le emende S-136
  assestamento-streak+retry reggono); lettura firma per gamba PRE-REGISTRATA
  (az.rev. S-136 #2 ASSOLTA nel criterio pair): leg1-off SPORCA (phpr ictx 2260%
  med motore) esclusa dal canone, leg1-on ELEVATA (oracle 137%) annotata dal
  giudice; parità per NOME 6/6; COPIA-GATE ricetta rc=0 (manifest 45 righe).
2·**Sonda eccedenza FD1 (az.rev. S-136 #4 ASSOLTA)**: rc=0, hit 2.999.999/miss 1,
  parità probe==pin; ma identità NON CHIUSA (UB 69,6 − 8,7 = 60,9 vs D 83,3,
  scarto −22,4 fuori banda 13,3): arm probe 56,7 vs ~34,9 implicato dall'A/B =
  artefatto del probe (timer nel call-site rompono l'inlining di
  field_assign_fast). Indizio dominante: plumbing set-entry 17,6→4,6 = **+13,0 ≈
  eccedenza +13,7**. Per p.5 pre-registrato: **blocco leve dim-write PERSISTE**.
3·**Istruttoria objmap con census** (build gc-census): m0 obj `inserted 3,0M
  (1/iter) · sweep main 3,0M (1/statement) · demoted 3,0M`; m1 int: 2. Canale
  «valore-oggetto 43,4» **ATTRIBUITO al round-trip nota→sweep→demozione**; leva
  note-time RIFIUTATA con precedente WP-21 (documentato nel sorgente); cura
  nominata = [[php-rust-gc-cycle-collector-plan]]. Az.rev. #1/#5 → criterio leva S-138.

## ⭐ Lezioni (max 3)
- ⭐⭐ Un probe che rompe l'inlining del bersaglio misura un arm che il binario
  vero non paga: l'identità di una sonda si chiude solo con artefatto omogeneo —
  altrimenti NON CHIUSA senza sconti a posteriori, e l'indizio resta indizio.
- ⭐⭐ Prima di disegnare una leva GC, leggere i precedenti NEL sorgente: la
  scorciatoia note-time era già caduta (flake WP-21) e il commento lo dichiara.
- ⭐ Un'anomalia leva-0 con tre ragioni per NOME vale più di un A/B forzato su
  un meccanismo refutato.
