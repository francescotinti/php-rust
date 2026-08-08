# WP_SESSION_113 — S-113: coppia WP saldata (full ON 1,871 in banda) · leva H-P1 tentata e NON promossa (prop +3,33 < 4; guardia calls sfondata) — revert al byte

**In una frase**: il collaudo sul sito WordPress vero conferma che l'ottimizzazione
della scorsa sessione non costa nulla sul carico reale (1,87× come prima); il
ritocco tentato oggi sulle proprietà velocizza davvero quel giudice ma sotto la
soglia pattuita, e per spostamento di layout rallenta un giudice non toccato: si
torna indietro, al byte.

**SCOREBOARD** (pin s112 f71abd2a INVARIATO; micro = valori S-112, frecce =):
**arith 5,5 = · prop 7,6 = · calls 5,2 = · str 5,3 = · arr 4,2 = · re 3,4 =** ·
held-out baseline 6,4·2,5·5,6 (non rimisurati: nessuna promozione). WP RIMISURATO:
**full ON 1,871 (rif 1,867) / OFF 1,891 · media 2,632/2,603 · peak ON 1842,27 MiB
(minimo famiglia)**. **Leve perf spedite: 0 — dichiarato; ritmo rispettato: 1 leva
TENTATA con A/B pieno R=5 e verdetto.** 2026-08-08 · Fable 5 · 0df621d→bbacd0e.

## Esiti secchi
1·coppia WP bimodale (criterio PRE 0df621d PRIMA della run): **full ON 1,871
DENTRO banda [1,81;1,88] → debito icache H-A2 ASSOLTO**; OFF 1,891 (famiglia
storica); media sesta lettura 2,632/2,603 (in discesa, voce aperta); parità per
NOME ×4 gambe (sola eccezione wp_is_stream ×2, 87/88); gate_void=0 →
2·istruttoria PRIMA del criterio: (g) census call-site `binary_value_ab`
ESAURITO a sorgente (senza guardia solo BinaryDst/CmpJmpConst, fuori dai corpi
caldi); (f) prop istruita → falla nominata: clone Rc del ricevitore buttato a
ogni get IC-hit (prior art ThisPropGet) → 3·criterio PRE 40fcc80 PRIMA del
codice → 4·leva H-P1 (2 siti) → 5·admission 12/12 dump identici; run_loop
291.316→293.292 B (+1.976, bl +24); batteria rc=0, 0 fail — **1741 vs 1742
S-112: delta NON attribuito, a verbale** → 6·A/B R=5: **prop Δ mediano +3,33,
5/5 positivi, SOTTO soglia 4** E **guardia calls SFONDATA −5,50 5/5** (sentiero
non toccato: layout, segno OPPOSTO e 3,7× il +1,50 S-112) → 7·NON PROMOSSA;
revert AL BYTE VERIFICATO (release=stash f71abd2a; diff crates/ VUOTO; bbacd0e).

## ⭐ Lezioni (max 3)
- ⭐⭐ **La banda-layout «raccontata» è smentita dal terzo punto**: −5,50 su calls
  non toccato, segno opposto al +1,50 S-112 e sopra la soglia stessa (4): l'A/B
  leva-nulla di attribuzione diventa PREREQUISITO di ogni prossima soglia.
- ⭐⭐ **Una leva lifecycle da ~1 clone Rc/op vale ~3 ns/iter: sotto il pavimento
  da sola.** Le prossime leve prop compongono più siti o cambiano famiglia.
- ⭐ **Run pesanti detached DA SUBITO**: la gamba ON partita come task e fermata
  ha lasciato la guardia uploads pendente (ABORT fail-closed corretto, restore
  da manifest). Incidente di processo contato.
