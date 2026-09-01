# Revisione S-168 (revisore singolo, lente SEMANTICA)

## VERDETTO: REGGE CON RILIEVI

I numeri reggono (m0 nullo −0,24; guardia e2 ≤0,16; additività 0,36; stride con controllo positivo +270). Non regge la LETTURA «rotta esaurita»: il kill scatta per costruzione del conto.

## Rilievi

1. **Kill = somma distorta al ribasso contro soglia secca.** Il pavimento 4 ns (REGOLE §3) nasce per la promozione A/B; in una decomposizione azzera ogni componente <4: m4b +2,96 (rumore 0,64, direzione firmata) vale 0. Σ grezza m1+m2+m4b = 12,9 > 10; Σ «nominata» 9,92 < 10 per 0,08 con rumore 0,2–0,6. E m2 è CONSERVATIVO dichiarato (guardia tupla, `to_zval` presente; patch r.70-71). Sostenibile solo «esito a filo, non deliberabile».
2. **Portata del kill più stretta del claim.** I mock aggrediscono UN handler; E2 mostra 11,1 ns di gap nei due handler di controllo (7,3 vs 1,8 ns/op) mai toccati da mock. «Interno-handler esaurito su arith» vale solo come «predecode/cotto sul fuso ≤10–13 ns». Hejlsberg non è contraddetto (i 2 op SONO handler), ma r.20 non lo riporta sul kill.
3. **m3 non misura frames[top].** La patch cambia forma: guardia AND, `Option<(Zval,Zval,Zval)>` (48 B mossi), `read_slot(l)` anticipato (m3 r.26-47). D=−2,28 = hoist − ristrutturazione, non separabili: «non rilevato» (sintesi r.14) è «confuso».
4. **Residuo sovra-attribuito.** m4b prova che il Sweep costa ~3 ns e in E2 NON c'è (e2 14,72→14,68) ⇒ «statement 32,04» lo contiene; la sintesi scrive «Sweep 0,00» e carica quei 3 ns sul handler.
5. **Legenda xctrace: c3 circolare, c0 tautologico.** c3 = discarded perché «sale col branchmut», ma lo stesso .out (4) dichiara il branchmut PREDICIBILE e rinvia c3 a S-169 — poi (2) legge c3 per refutare mispredict. c0 «scende in ogni mutante» è identità di normalizzazione (Σ=1); mutante proprio di c0 (az.rev.2 S-167) non eseguito. Il collaudo S-167 «morde 11,9×» è INVALIDATO; regge solo memstall→c1.
6. **Chiusura 52,7% = 21% meccanismo + 31% aggregato**: e2 è sottrazione, non nomina.

## Azioni S-169

1. Emendare la sintesi: kill «a filo, non deliberabile»; Σ grezza 12,9 a registro; Sweep ≈3 nel residuo; m3 «confuso».
2. Mock sui due handler di controllo (CmpJmpSC, IncDecSlotJmp) con giudice E2.
3. Ri-misurare m4b e m3 con R esteso (criterio p.2) fino a rumore <0,3; soglia = rumore, separata dal pavimento 4.
4. xctrace: `arith-rbranchmut.php` PRIMA di ogni lettura di c3; mutante proprio di c0; intanto «mispredict refutato» → «non firmato».
5. m2 senza guardia (via flag di build) per togliere il CONSERVATIVO dal kill.
