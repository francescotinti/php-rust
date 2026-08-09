# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full = 1,810–1,889** (S-120, su pin VECCHIO s120 —
**da rimisurare sul pin s124**: leva promossa ⇒ trigger p.4) · ultima leva
SPEDITA **S-124 (PhpStr single-alloc)** · **sessioni-senza-Δ = 0** · incidenti:
1 (S-106) + 2 processo (S-115; **S-124: arbitro admission committato DOPO i
suoi verdetti** — rev. PROCESSO).

## Scoreboard (pin **s124 c5ba2573**a23adf69 @ fb140d1; micro R=5 sul pin vs oracle)

**arith 5,5 · prop 5,6 · calls 4,7 · str 4,2 · arr 3,2 · re 2,5**. S-124 =
PhpStr single-alloc PROMOSSA: ZStr = blocco unico {rc,hash,len,cap}+bytes
(refcount custom, PartialEq ptr-fast-path, try_append realloc, builder ConcatN,
drop_slow OUTLINED — B1 inlined sfondava calls −3,94, flip a +0,81 = canale
icache indiziato, attribuzione NON firmata da A/B proprio, rev. #2). A/B str2:
**str +39,42 · arr +29,38 · re +50,48 ns/iter (5/5), guardie 6/6**. Admission
census: **5 predette esatte + 1 (re) costruita dopo ridiagnosi Vec-fed→slice-fed**
(dicitura da rev. #5); Δalloc/iter vs oracle ora **str +1,00 · arr +0,02
(≈PARITÀ) · re +2,00**, resto 0. Gate: batteria 1746/0/2 (inventario + 4 zstr
dichiarati) · corpus congelato+golden+off↔on 0 · fixture verdi · ORM 3484
3E/13F per NOME == baseline · http-kernel 1665 0E/0F · server **s124 f9526be3**
GRADO MINIMO (pieno = debito).

## §S-125 — ordine proposto

1. **Full/media WP A/B sul pin s124** (trigger p.4 scattato: leva promossa):
   coppia bimodale pair109 intercalata vs oracle, riferimento S-120
   1,810–1,889 · ~2,51–2,53 · peak 1862–1983 MiB; uploads sotto guardia;
   run DETACHED sequenziali col marker .done.
2. **Controllo ±zval-census STESSO head** (az. rev. S-123 #1, rinviato da
   S-124) + **rivalidare le bande layout sul regime post-patch** (az. rev.
   S-124 #4: le SOGLIA_LAYOUT v2 vengono da binari s120/S-123; i candidati
   nuovi sono post-patch): banda-layout v2 rieseguita con K copie del pin s124
   PRIMA della prossima leva micro.
3. **prop oltre i cloni** (istruttoria s122): SOLO dopo il p.2 (attribuzioni
   per-sito o etichettate INFERENZA).
4. **PGO stadio-2** (criterio §6 s117: workload WP con teardown, mai le micro).
PROCESSO (rev. S-124, vincolanti): arbitri d'admission (tolleranze incluse)
si committano NEL commit del criterio PRIMA del primo run (verdetto da script
non committato = incidente contato) · patch oltre il perimetro dichiarato ⇒
ri-commit del criterio emendato PRIMA di rieseguire l'arbitro · registro di
promozione nel verdetto del run PROMOSSO, non del bocciato.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

Full/media su pin s124 (p.1) · bande layout post-patch (p.2) · str residuo
+1,00 alloc/iter vs oracle (ultimo malloc di str.php da classificare) · re
residuo +2,00 (dopo single-alloc: 7,00 vs 5,00) · grado pieno server s124 ·
cura §3.18 (gate preg ROSSO + golden stesso commit) · fame frontend (kpc/sudo)
· §3.16/§3.17 warning · retro-A/B str s107b/s108/s109 · denti rinviati (OBS-8;
fx20; direct-bind; drop-order; hit/miss; checkout-staging) · $z++/$z-- undef
non warna · §3.13 · §3.12-i · §3.14 · get_gc · drift TODO.md · latin1-cliff.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

BOLT su Mach-O · NaN-boxing (veto Hoare) · threaded-dispatch (veto Hejlsberg)
· PGO addestrato sui giudici · verdetti su build emendata senza ri-banda ·
pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze
tra A/B distinti come cifra · componenti prezzate · magnitudine ripartita
senza A/B proprio · fixture su memory_get_usage · «icache» NON-premessa ·
pre-filtro che tassa i freddi · guardie non-bersaglio BILATERALI ·
denominatori a memoria · output di run nel repo · rc di gate da pipe (6
morsi) · tee/log pre-mkdir · admission sul dump intero (deroga forma S-118
citata nel criterio) · xctrace senza guardie disco · run pesanti come task ·
edit coi build in volo · promozione sotto banda · gate a soglia fissa senza
banda · bande pre-pipeline su binari post-pipeline · corpus-gate solo-nomi ·
strumentazione nei sorgenti del pin · leve micro senza banda v2 · zavorra
run-to-run come arbitro del layout · alloc-removal senza modello del costo
SOSTITUTIVO · probe senza riferimento vivo · classifiche da census che spegne
la fusione · ordine FISSO di misura: SEMPRE permutato o alternato · delta tra
census di epoche diverse senza datare i raw · **verdetti da script d'arbitrio
non ancora committati** · **«N/N esatte» quando una predizione è costruita
post-ridiagnosi** · SSO inline (WP-38, riconfermato: single-alloc ≠ SSO).

---
**Riscritto**: 2026-08-10 (chiusura S-124). Storia: `sessions/` ·
`gaps/GAP_TREND.md` · revisione in `wp124-harness/revisione.md`.

Pre-flight S-125: pin phpr **s124 c5ba2573**a23adf69 @ fb140d1 (ricetta A′; la
batteria rilinka ⇒ build ricetta e pretendere c5ba2573 al byte) · server
**s124 f9526be3** GRADO MINIMO (pieno = debito) · MySQL wp8 con l'elenco (se
giù: mysqld_safe su datadir esterno via daemonizer) · uploads sotto guardia ·
conservati: phpr-s118/s119/s119-treno2/s120-re1/s121-st1/s122-lay1..3/
s122-re2/s123-p0b/**s124-str1(reperto B1)/s124-str2(==pin)** ·
php-server-s119/s120/**s124** · census-target = build S-124 fusa (head con
patch) · Serena OK (hang S-123 rientrato) · disco Data risanato (26G) ·
nessuna run detached in volo.
