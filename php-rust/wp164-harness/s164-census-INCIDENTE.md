# INCIDENTE S-164 #1 — census rerun eseguito col path del harness CHIUSO (dichiarato, contato)

**Difetto**: in `s164-census-au1-rerun.sh` la riga `H="$REPO/wp163-harness"` è
sopravvissuta alla copia (il sed `s163→s164` non tocca la stringa `wp163`, che
non contiene `s163`); il mio copia-gate ha ispezionato le sole righe attese e
NON la riga `H` ⇒ **verifica riga-per-riga INCOMPLETA** (REGOLE §3: la copia
dichiarata si verifica PRIMA del run — qui la verifica c'era ma era parziale).

**Effetto**: la riesecuzione (01:23–01:30) ha scritto in
`wp163-harness/census-out/` SOVRASCRIVENDO gli artefatti locali del run census
S-163 (census-{A,B}-r{1,2}.raw/.txt, run-*.out, build-*.log, patchR.log e
`census.done` — quest'ultimo era il file scritto A MANO oggetto dell'incidente
S-163: l'originale è perso come artefatto; la sua storia resta nei verdetti e
in revisione.md). Violazione OPERATIVA del veto «harness chiusi congelati».

**Merito NON toccato**: il rerun è valido — attesa EMENDATA 600000 NEL
sorgente, rc=0 emesso DALLO script, Δ hostcall_n=600000 ESATTO, Δ
class_exists=600000, altri nomi zero, repliche r1==r2 su entrambi i probe.
La cura dell'incidente S-163 è ASSOLTA nel suo canale autoritativo.

**Bonifica (S-164, ore 04:2x)**: artefatti del rerun SPOSTATI in
`wp164-harness/census-out/` (32 file) + verdetto in `wp164-harness/`;
`wp163-harness/census-out/` resta vuota (gli originali s163 non sono
recuperabili); riga `H` corretta nello script col marcatore del fix; il
manifest `s164-census-copia.diff` in git documenta la versione ESEGUITA.

**Lezione**: il copia-gate si verifica sul DIFF INTERO più un grep dei path
assoluti/di harness nel risultato (`grep -n 'wp1[0-9][0-9]-harness'`), non
sulle sole righe che ci si aspetta di aver cambiato.
