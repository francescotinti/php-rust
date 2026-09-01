# WP_SESSION_166 — PROMOZIONE L-MCk (pin s166); due coppie assolte (t15@s165, t16@s166); fixture semantica fx-mc2
**In una frase**: il fast path delle chiamate a metodo aperto ieri ha perso il
tetto di arità (ogni chiamata semplice ora salta l'imbuto, −10,6% sul micro a
tre argomenti), la leva di ieri è stata ricontrollata a fondo dove il revisore
aveva puntato (identica al byte anche negli errori), e il motore promosso ha
passato due giri completi di collaudo sulla suite WordPress e su Doctrine.
**SCOREBOARD** (pin NUOVO **s166 phpr 092dcff431bef876 + server caa4e4b2638686a9**):
arith 5,4 = · prop 5,5 (5,6↓tick) · calls 4,8 (4,9↓tick) · str 4,2 = · arr 3,2 =
· re 2,5 (2,6↓tick) · giudici propri: **mc3 202,5→181,0 (−10,6%)** · mc2 ~155
conservato · **WP t15 1,746 (banda_ON 0,005 RECORD) e t16 1,749, entrambe
COMPATIBILI** (assetto post-leve da 1,761) · ORM registrato **[7,023;7,053]**
(finestra valida @ s165) · corpus 1412×2 · **leve spedite: 1 PROMOSSA** ·
incidenti: 1 (+2 recidive di classe copia: 1 dichiarata, 1 trovata dal revisore).

## Esiti secchi
1·fx-mc2 (az.rev.1 S-165): pin==funnel BYTE-ID su error-path/__get/deref=false/
  dtor/Fiber ⇒ rilievi semantici NON osservabili, az.2-3 non scattate; catalogo
  +§3.28 (ordine valutazione SEND_VAR_EX + dtor temp) +§3.29 (Fiber non final).
2·Coppia t15@s165: WP 1,746 COMPATIBILE (banda_ON 0,005 RECORD) · ORM cifra
  valida al 3° aggancio (2 finestre contaminate DICHIARATE dalla sentinella E2
  — emende E1 ictx-per-motore/E2 banda collaudate sul campo) [7,023;7,053].
3·L-MCk: cade il cap argc≤2 (edit 1 predicato); smoke rc=0 al PRIMO colpo
  (mc3 +20,0), R=5 +21,5 riconc. |1,5| dopo 1 tentativo-gate flare; catena §6
  piena ⇒ PIN s166. Churn batteria: ipotesi-env REFUTATA (intrinseco a cargo
  test), cura = rebuild ricetta, tornato al byte ×2.
4·Coppia t16@s166: WP 1,749 COMPATIBILE 6/6 · ORM parità valida, cifra non
  giudicante per 1 TICK (sentinella lato veloce 4,82) — riferimento resta
  [7,023;7,053]; apertura ri-fondazione banda PRE-registrata prima della
  prossima coppia. Incidente #1 (lanci senza copia-gate) + recidiva VERD-naming.

## ⭐ Lezioni (max 3)
- ⭐⭐ i copioni GENERATI (sed/python) falliscono sulle sostituzioni OMESSE, non
  su quelle fatte: il copia-gate deve grep-are TUTTI i nomi di scrittura
  (.done/VERD/log) del file base, non solo i path già noti.
- ⭐⭐ una banda sentinella fondata su N=6 morde 3 finestre su 4 in giornate
  cariche: fa il suo lavoro (mai cifre sporche) ma il costo è rinvio seriale —
  la ri-fondazione periodica va nel criterio, pre-registrata.
- ⭐ il churn di cargo test è INTRINSECO (l'env non c'entra): l'ordine
  batteria→rebuild-ricetta→re-hash→stash è la forma stabile del §6.
