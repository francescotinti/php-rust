# WP_SESSION_118 — R3-Pedersen SALDATO 4/4 (parità WP per-request sotto A′) · treno-1 H-P1 PROMOSSO (prop +5,33 5/5, la tassa calls non esiste) · pin s118

**In una frase**: il collaudo sul sito WordPress vero e sul server persistente
conferma che la nuova fabbrica del programma non ha rotto nulla (zero oggetti
persi per richiesta, stessi identici test verdi), e il ritocco alle proprietà
bocciato tre sessioni fa è stato ripescato e promosso: ora il metro fine
mostra che velocizza davvero, e il rapporto sulle proprietà scende a 5,5×.

**SCOREBOARD** (pin s118 **15dfb6b3** @ f7f7da7, micro R=5 sul pin; frecce vs
s117): **arith 5,5 ↑lieve (5,4) · prop 5,5 ↓ (5,9) · calls 4,8 = · str 5,3 = ·
arr 3,8 = · re 3,3 ↓** · held-out 6,3·2,5·5,4 · WP RIMISURATO sotto A′ (sul pin
s117): **full ON 1,913 / OFF 1,955 · media 2,506/2,580 · peak ON 1839,2 MiB**
(nuovo riferimento; il rif 1,867 decade come pre-registrato). **Leve perf
spedite: 1 (treno-1 = H-P1a/b)**. 2026-08-08 · Fable 5 · 9cf1fcd→f7f7da7.

## Esiti secchi
1·**R3 saldato** (criterio PRE 9cf1fcd, verdetto s118-r3-verdetto.out): gate1
repin server A′ 20411ba0 (phpr INVARIATO, zero churn) · gate2 grado ×2 modi
rc=0 voids=0 (option 413 + restapi 3508 IDENTICI per NOME) · gate3 pair109
stessa sera ×2 modi (media 0 fail ×4 gambe; full diff == SOLO wp_is_stream) ·
gate4 tripla census **obj/req 0,000 spread 0,000** (KiB/req 0,0000, uc-steady
3 leg) ⇒ «parità CLI» ESTESA a «parità WP per-request»; verdetto WP di A′ CHIUSO.
2·az.rev. S-117: **corpus-gate CANONICO** (`scripts/corpus-gate.sh`, nomi E
CONTENUTO E off↔on; golden 1412 digest/modo dai raw del pin) collaudato con
replay rc=0 E mutante rc=1 col test nominato; prima esecuzione LIVE verde in
promozione · A/B eseguito nel TARGET CANONICO (niente cross-target).
3·**treno-1** (manifest 5 vagoni per NOME 3148cae PRIMA; criterio PRE 9013b2b):
H-P1 composto sopra L-A (patch 52 righe additive, conflitto PropGetSlotRecv
risolto: probe fuso invariato, ramo !fused in prestito); admission 6/6 output +
6/6 dump INTERI byte-id; A/B R=5: **prop +5,33 ≥ 4,00, 5/5; guardie TUTTE
tengono (calls +0,50)**; held-out 3/3 · §6 pieno: build riproduce il candidato
al byte ×2 (pre e post batteria 1742/0/2 inventario IDENTICO), pin via script,
corpus canonico rc=0, fixture 6/6. V3-V5 (IncDec/Isset/MCall) istruiti per S-119.
4·C-lite: disegno committato (5a834f7); esecuzione rinviata a S-119 (dichiarato).
5·⚠️ release/php-server post-promozione RIPRISTINATO allo stash gradato
20411ba0: la leva NON è ancora nel server (repin+grado = apertura S-119).

## ⭐ Lezioni (max 3)
- ⭐⭐ **Una leva bocciata è un candidato congelato, non un morto**: H-P1 (+3,33
  sotto soglia S-113) promossa al 2° giudizio quando il metro fine (banda calls
  0,50) ha mostrato che la «tassa» era layout — terza firma dello stesso reperto.
- ⭐⭐ **Il gate che non giudica il contenuto passa leve che cambiano il
  dettaglio**: il corpus-gate nomi+contenuto è nato dal buco che il revisore
  S-117 aveva coperto a mano, e ha girato live già stasera.
- ⭐ **Il determinismo paga tre volte in una sera**: riproduzione del candidato
  giudicato, neutralizzazione del relink batteria, ripristino del release al
  pin dopo l'A/B — tutti «al byte, pena STOP», tutti passati.
