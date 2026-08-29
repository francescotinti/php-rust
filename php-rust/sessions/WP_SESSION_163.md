# WP_SESSION_163 — coppia t13+ORM assolta al primo pin; leva L-AU1 PROMOSSA → pin NUOVO s163; §3.26/3 + §3.27 a catalogo
**In una frase**: rifatta la coppia di benchmark sul motore appena promosso
(tutte le gambe buone; un indicizzatore macOS ha fatto perdere solo tempo, mai
misure), e spedita una leva vera: l'autoloader in stile Composer
(`[$loader,'loadClass']`) non alloca più nulla a ogni classe mancante — il
caso miss costa il 12,6% in meno.
**SCOREBOARD** (pin NUOVO s163 phpr fea4a2d040a0d8d0 + server 8d76d6f129bfd4af):
arith 5,5 = · prop 5,5 = · calls 4,7 ↓ · str 4,2 = · arr 3,2 ↑tick · re 2,6
↑tick (rif s162: 5,5/5,5/4,8/4,2/3,1/2,5; ⚠️ arith 5,5 PERSISTE da s162 ⇒
indagine DOVUTA S-164) · WP t13 mediana 1,763 COMPATIBILE 6/6 pulite ·
banda_ON companion 0,029 · media 2,435-2,466 · ORM [7,072;7,114] VALIDO
(assoluto MIGLIORA [+0,04;+0,21]s) · dbal [7,394;7,570] riserva ictx
RICORRENTE · corpus 1412×2 · batteria 1748/0/2 (cap 25909/7726) ·
**leve spedite: 1 (L-AU1 PROMOSSA)** · incidenti: **+1 NUOVO dichiarato**.

## Esiti secchi
1·Coppia t13 rc=0 COMPATIBILE 1,763 (6/6 pulite, quiete dichiarata); ORM rc=0
  al 2° giro (1° rc=8: flare mediaanalysisd oltre retry ×3 — zero gambe
  giudicate sporche); sentinella oracle 4,85/4,84 dal lato veloce ⇒ VALIDA;
  attesa-AM2 COMPATIBILE come pre-registrato; attesa-AF1 aperta (+2 gambe pool).
2·Rev. S-162: az.1-2 ASSOLTE (fx-sm esteso BYTE-ID: default-param, return-hint,
  namespaced, generator; §3.26/3 array vuoto). Az.3 sonda strmap NON eseguita
  (sito diverso dalla leva odierna, resta in aperture); az.4 rinviata (bracci
  morti ora DUE: call_fn_one + call_method_one).
3·Leva L-AU1 (autoload [obj,metodo] k=1): smoke +45,5 FUORI banda [12;30] ⇒
  census con ERRORE D'UNITÀ dello strumento DICHIARATO (hostcall_n = TUTTE le
  alloc) ⇒ attesa corretta **3 alloc/miss ESATTE** (bisezione a 7 forme) ⇒
  R=5 **D=+42,0** (334→292, rumore 2/4), 19/19 guardie, disasm Δ=0 ⇒
  PROMOZIONE rc=0 (claim EMENDATO post-revisione: arbitrato census emendato
  A MANO, census.done scritto dalla sessione = INCIDENTE contato); conferma post-pin +42,0 segni 5/5 CIFRA
  PIENA. Reperti: **L-AL3 candidata** (fast path closure alloca 1 Box
  FrameExt/miss) · §3.27 (self-unregister con successore) · 3 TAG A/B bruciati
  dal flare · gemello A == pin AL BYTE al PRIMO tentativo (target canonico).

## ⭐ Lezioni (max 3)
- ⭐⭐ l'attesa census si pre-registra sull'UNITÀ DELLO STRUMENTO letta dal SUO
  sorgente (hostcall_n conta TUTTE le alloc, non i soli vecargs): l'arbitrato
  ha smascherato l'errore e la bisezione a 7 forme l'ha trasformato in un
  ledger per-forma che ha già indicato la prossima leva (L-AL3).
- ⭐ un flare INTERMITTENTE si attende con quiete CONTINUA (6 campioni ×30s),
  non con campioni radi: i TAG bruciati sono il prezzo dichiarato del gate.
- ⭐ la lezione s162 paga: gemello nel target CANONICO ⇒ pin AL BYTE al primo colpo.
