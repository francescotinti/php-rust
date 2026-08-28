# WP_SESSION_160 — istruttoria phpr1 CHIUSA; coppia t10+ORM @ s159 (AL1 CHIUSA); fx-am v2; leva L-AF1 PROMOSSA → pin NUOVO s160
**In una frase**: trovata e curata la causa delle misure ORM «sporche»
(transitorio di primo-run: ora si scarica con un giro di rodaggio), rimisurato
tutto pulito sul motore nuovo, e spedita la seconda leva della famiglia
array (array_filter senza allocazioni per elemento), prevista dal modello.
**SCOREBOARD** (pin NUOVO s160 phpr ceeb6e76e4ef5ace + server 001a4b2bf04a73ae):
arith 5,4 = · prop 5,5 = · calls 4,7 = · str 4,3 = · arr 3,2 ↑tick · re 2,6
↑tick (rif s159: 5,4/5,5/4,7/4,3/3,1/2,5; arr/re = tick di denominatore,
guardie A/B verdi) · WP t10 mediana 1,760 COMPATIBILE (6/6 pulite, banda_ON
0,008 pareggia il record) · media 2,434–2,458 · ORM 7,077–7,097 · dbal
7,541–7,550 · corpus 1412×2 · batteria 1748/0/2 (cap loc 25810/7708) ·
**leve spedite: 1 (L-AF1 PROMOSSA)** · incidenti 19 (+2 in ratifica: #20, #21).

## Esiti secchi
1·Istruttoria phpr1 ictx (az.rev. #4): transitorio INTERNO di primo-run
  (leg1 598/s→leg2-3 ~120/s, istruzioni identiche, Δ-daemon≈0: Spotlight
  scagionato); RETTIFICA «3ª finestra»→2 su orm; rimedio rodaggio+quiescenza
  NEL criterio ORM → in campo: 4/4 gambe PULITE (orm phpr1 53,1 vs 160,5).
2·Coppia t10 rc=0 COMPATIBILE 1,760 ∈[1,738;1,799]; parità 6/6; peak BASSE.
3·ORM @ s159 rc=0: attesa-AM1 COMPATIBILE su scaletta a DUE estremi (az.rev.
  #1; Δ_norm [+0,04;+0,24] intervallo INTERO in banda) · **replica-AL1
  CHIUSA** (gambe pulite: attesa 0,02-0,05 sotto-risoluzione CONFERMATA).
4·fx-am v2 (az.rev. #2): 20 forme BYTE-ID (generatore, hint, func_get_args
  nel fast=1 arg ESATTO, static/bind, by-ref mascherato).
5·Leva L-AF1 (census-quota: tranche-2 COLLASSA su array_filter 1,83M; walk/
  usort/reduce assenti): smoke +14,0 in banda [8;22] → R=5 **D=+16,0 SOPRA
  SOGLIA** (m-arrfilter 196→180), riconc. |2,0|<4,0, **FUORI-UB +1,5-4,0
  dichiarato** (componenti nominate) ⇒ sonda surplus DOVUTA S-161; 17/17
  guardie (arrmap +2,0); bl 6033==6033; promozione rc=0 CON RETTIFICA.
6·RETTIFICA promo (incidente #21 PROPOSTO): copia con H stantio (wp159) —
  fx-af/conferma girati su file INESISTENTI = falso verde da errore identico;
  ri-derivati con esito ESATTO (fx-af BYTE-ID+marcatore; conferma post-pin
  D=+15,0 segni 5/5). Emenda proposta §3: copia-gate con verifica POSITIVA
  dei path + marcatore preteso in ogni gate-fixture.

## ⭐ Lezioni (max 3)
- ⭐⭐ un gate che confronta A==B senza pretendere l'ESITO può passare su due
  errori identici: il marcatore va preteso SEMPRE (forge-silent-failure, di
  nuovo — stavolta ha morso una promozione).
- ⭐⭐ la causa di una misura sporca si cerca con conteggi invarianti
  (istruzioni) + sospetti nominati (Δ-daemon): 40 minuti di istruttoria hanno
  reso pulite 4/4 gambe dopo 2 finestre di flag.
- ⭐ il gate census-quota decide la tranche PRIMA di scrivere codice: 3 nomi
  su 4 non pagavano e sono rimasti fuori dal diff.
