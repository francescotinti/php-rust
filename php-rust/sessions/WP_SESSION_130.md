# WP_SESSION_130 — F4 SPEDITA (pin s130) + sonda E1a: il residuo NON è la resolve

**In una frase**: la leva che salta i controlli-hook mai necessari nelle scritture
su proprietà, bocciata in S-129 per una soglia gonfiata da un valore anomalo, è
stata rimessa alla prova con la formula di rumore corretta e stavolta promossa su
tutta la catena di collaudo (micro-oggetti fino a −20%); la sonda nuova mostra
che il prossimo bersaglio grosso non è la ricerca-per-nome (~39 ns) ma il resto
del cammino di scrittura (~120 ns).

**SCOREBOARD** (pin NUOVO **s130 0fdf1c49** + server s130 7fb79069; micro gate promozione):
**arith 5,5 ↗ · prop 5,6 = · calls 5,0 = · str 4,3 ↗ · arr 3,3 ↗ · re 2,5 =**
(↗ = jitter denominatore oracle; phpr netto invariato) · WP full NON rimisurato
(resta 1,758–1,805 @ s127b) · **leve spedite: 1 (F4)** · incidenti: 0. 2026-08-11.

## Esiti secchi
1·**Criterio F4 emendato PRE-REGISTRATO** (879a38e, PRIMA di ogni run): rumore
  trimmed drop-1 SIMMETRICO su A e B; bande FONDATE objmap 6,7 · objalloc 6,7 ·
  objchurn 13,3 (range drop-1 delle 7 gambe A committate S-129). Riesecuzione,
  non ricalcolo (rev. S-112).
2·**F4 SPEDITA**: revert-del-revert (89e59ea) riproduce 0fdf1c49 AL BYTE (census
  11/11 resta valido) · smoke R=2 +80,0 PROMOSSA · R=5 D=+80,0 vs soglia 16,7
  PROMOSSA, guardie in banda · direzione B-più-veloce **14/14 cumulata**.
3·**Promozione COMPLETA rc=0**: batteria 1746/0/2 inventario identico · churn
  build neutralizzato al byte · pin s130 · corpus nomi+golden+off↔on · fixture
  rc=0 · micro R=5 · ORM 16 nomi==baseline · hk 0E/0F · server s130 grado minimo.
4·**Micro-ORM su s130**: objdatains **7,7** (1253,3 ns, coerente al ns col lato B
  dell'A/B) · churn **8,6** · dropdef 9,0 · allocni 9,8 · alloc 7,8 ≈ · objmap 17,0 ≈.
5·**Sonda E1a ACQUISITA** (az.rev. S-129 #1): k deterministici (alloc **4** a zero
  statement! datains 9 · p5/p6 14) ⇒ 4 resolve/iter vengono dal CTOR; per-statement
  **5 resolve = 39–44 ns ≈ 24% di E−E2** (la riga «67%» del verdetto è respinta in
  lettura: E1a non è sottoinsieme di E−E2); UB resolve-once statement-only **31–35
  ns**; il grosso di E−E2 (~120 ns) è prop_step NON-resolve (3× prop_key +
  contains/get_mut/replace). TOT 232,9 vs 296,7 pre-F4: il modello trasferisce.
## ⭐ Lezioni (max 3)
- ⭐⭐ Una leva con direzione firmata e criterio caduto su un artefatto della formula
  si RIESEGUE col criterio emendato pre-registrato: F4 da avversa a spedita senza
  toccare una riga di codice.
- ⭐⭐ Una sonda su funzione CONDIVISA esige il controllo a zero statement: senza
  objalloc k=4, i 111 ns di E1a passavano per quota dello statement (67% vs 24%).
- ⭐ Le bande di guardia si fondano sulle gambe del pin già committate: zero run extra.
