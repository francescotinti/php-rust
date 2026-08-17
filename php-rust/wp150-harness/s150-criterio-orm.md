# Criterio S-150 p.4 — coppia ORM (+dbal companion) @ pin s150 = GIUDIZIO della scommessa BT1 — scritto PRIMA del run

1. Scommessa PRE-REGISTRATA (s149-decisione-bt1.md p.4, criterio bt1 p.6):
   al pin nuovo la coppia ORM deve dare direzione ↓ fuori dal rumore;
   attesa dichiarata ↓ 0,8–3,1 s sul phpr user netto (≈2–7% di ~42 s);
   denominatore/risoluzione KS-146-1 = 0,293 s. **La scommessa è l'ARBITRO
   del valore-suite della leva (az.rev.5); la promozione (fedeltà) non
   dipende da questo esito.**
2. Metodo: `s150-orm-coppia.sh` = copia DICHIARATA di s147-orm-rimisura.sh
   (manifest `s150-orm-copia.diff`): N=2/lato, oracle prima (`memory_limit=-1`
   §3.14), workspace ri-untarrati, watchdog, `/usr/bin/time -l`, pavimenti
   med3 per-binario PER WORKSPACE (misurati, MAI ereditati — az.rev.4),
   gate ictx/s, lock di sessione VERIFICATO. Pin atteso = **s150
   cbbe71735effb165** (pena rc=9).
3. Riferimento @ s145 (s147-orm-rimisura-verdetto.out): phpr ORM net
   [41,60; 42,22] s (user − floor 0,07) · rapporto 8,370–8,427 ·
   oracle net ~5,0 s. dbal companion: phpr net [9,02; 9,04] · 8,20–8,37.
4. Giudizio pre-registrato (meccanico, su Δ = ref_net − new_net, intervalli):
   Δ_min = 41,60 − max(new_net) · Δ_max = 42,22 − min(new_net);
   - **VINTA** se Δ_min ≥ 0,293 (↓ fuori rumore su tutto l'intervallo);
     **CENTRATA** se inoltre [Δ_min, Δ_max] ∩ [0,8; 3,1] ≠ ∅; oltre 3,1 =
     oltre-attesa (dichiarare);
   - **PERSA** se Δ_max < 0,293 (fermo dentro il rumore, o ↑).
5. Parità: ORM fail-set per NOME == baseline 16 (pena cifra NULLA); dbal
   stabile tra gambe. dbal = companion, NON arbitra la scommessa (attesa non
   pre-registrata; un ↓ eventuale si dichiara come reperto).
6. Rapporto bilaterale citabile SOLO come intervallo net; nessuna
   attribuzione oltre «BT1 = unica leva spedita tra s145 e s150» (REGOLE §4:
   direzione+meccanismo firmati dall'A/B, magnitudine ripartita SOLO se il
   Δ cade nell'attesa).
