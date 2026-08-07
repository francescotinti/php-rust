# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)

⏱ **FONDAMENTALI**: riferimento WP = **full ON 1,842× / OFF 1,911×** (S-109,
sul pin s108 PRE-lotto-3; minimo storico, terza sera in discesa) · media
2,707/2,634 (voce APERTA, off rientrata al rif 2,64) · **coppia WP DOVUTA in
S-110 sul pin NUOVO 92909544 (debito: lotto-3; run_loop −976 B, attesa nulla
o favorevole)** · ultima leva = S-109 (lotto-3 str: +37,5 ns/iter 5/5) ·
sessioni-senza-Δ-rapporti = 0 · incidenti «mai collaudato»: 1 (de67cb64, S-106).

## Scoreboard (pin 92909544, micro R=5, S-109)

**arith 9,3 · prop 7,9 · calls 5,1 · str 5,3 · arr 3,9 · re 3,5**
str assolto dal lotto-3 (6,2→5,3; corpo 8 op/iter). Colli restanti per NOME:
arith/prop = dispatch (threaded, GATED L1I — xctrace vuole Xcode, decisione
utente) · ciclo di vita Zval (Sweep ovunque) · funnel interni · calls =
cross-frame (inlining/threaded).

## Stato gate

- **phpr pin 929095448e823cb5** @ HEAD S-109 (stash `phpr-s109`; = il binario
  dell'A/B, conservato anche come phpr-s109-ab-candidate) — batteria 1742/0
  rc=0 · corpus **1415 per NOME ×2** · fixture 6/6 (con W9 nuove) · ORM 16
  nomi = baseline ESATTA · hk 0E/0F · run_loop 287.944 B · default flag-ON ·
  oracle 07b0df8d. Stash 3b3d25e2 (s108) resta per retro-A/B.
- **php-server 443ae42f GRADATO PIENO ×2 (S-109, post-lotti 1-2)**: cifre
  server ATTRIBUIBILI. Il binario server NON contiene il lotto-3 (HEAD
  5e2713d); re-pin post-lotto-3 nominabile, non dovuto per le cifre gradate.
- Census: terzo giro S-109 in wp109-harness/census-out (fuori repo).

## §S-110 — ordine provvisorio

1. **COPPIA WP FULL+MEDIA BIMODALE in APERTURA sul pin 92909544** (debito
   lotto-3; criterio PRIMA: banda su riferimento 1,842/1,911; fuori banda
   sopra il cap della banda pre-registrata BLOCCA la leva).
2. **Leva S-110 — scelta per NOME** (criterio PRIMA): (a) str [StringifySlot;*]
   solo con istruttoria sospendibilità __toString (vincolo S-108) · (b) arr
   RMW-su-dim [FetchDim;BinarySTDst] previa istruttoria FetchDim · (c) Sweep/
   ciclo-vita Zval (in TUTTI i giudici) · (d) contatori L1I: Xcode INSTALLATO
   ma xctrace E GIT rifiutano senza `sudo xcodebuild -license` (azione utente;
   ripiego git: `DEVELOPER_DIR=/Library/Developer/CommandLineTools`) ·
   (e) funnel interno arith: fast-path i64 nelle op fuse (parere esterno
   VAGLIATO in wp109-harness/parere-esterno-gemini-20260807.md: solo con
   criterio+A/B, stime esterne MAI nei criteri).
3. Azioni revisore S-109 (wp109-harness/revisione.md, PROCESSO): admission
   BIPARTITA per commit · diff prelude ON enumerato (conteggi F1/F2 attesi) ·
   batteria-sul-byte o dichiarazione permanente · **vincolo: commit+push A
   OGNI PASSO anche nella finestra della leva (monocommit 35d9ff1 contato)**.
4. Chiusura lean: rotazione + revisore singolo (lente: MISURA).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

media voce aperta (2,707 on / 2,634 off: quinta lettura con la coppia S-110) ·
**§3.16 riga errata warning undef-var ricevitore prop-assign** (bilaterale,
repro parcheggiata in wp109-harness/w9-fixtures/) · fixture w9a caso B rientra
nel gate quando §3.16 è curata · contatori L1I (prerequisito: Xcode) ·
retro-A/B str coi tre stash s107b/s108/s109 · denti rinviati (OBS-8 terza
mutazione; mutante fx20; dente direct-bind; dente drop-order; contatore
hit/miss) · fedeltà: $z++/$z-- undefined non warna · §3.13 · §3.12-i · §3.14 ·
get_gc · bl-count run_loop: metodo NUOVO (otool sul simbolo, 5849) non
confrontabile col «29» storico — se serve, si ricostruisce il vecchio per NOME.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze
tra A/B distinti come cifra · componenti prezzate nei criteri · magnitudine
ripartita senza A/B proprio · estensioni di finestre senza criterio+dente ·
allargare simple_call senza dente+fx21 · fixture su memory_get_usage (stub) ·
«icache-bound» come premessa firmata · denominatori a memoria · output di
run nel repo · rc di gate da pipe (MORSO per la 4ª volta in S-109 al lancio
batteria, fermato prima del verdetto — vale anche per i comandi detached) ·
tee/log prima del mkdir · finestre fuse OLTRE un helper sospendibile
(vincolo S-108) · **admission d'emissione sul dump INTERO (il perimetro è il
{main}: il prelude fonde in ogni giudice — emendamento S-109)**.

---
**Riscritto**: 2026-08-07 (chiusura S-109). Apertura/chiusura = skill v2.
Storia: `sessions/` · `gaps/GAP_TREND.md` · revisioni in `wp10*-harness/`.

Pre-flight S-110: pin phpr **92909544** (fa fede HEAD, la build churna) ·
server 443ae42f (stash php-server-s109) · MySQL wp8 con elenco DB · debug/
da rimuovere · uploads sotto guardia · nessuna run in volo · ⚠️ disco Data
a 12G post-Xcode (soglia 15G: liberare o dichiarare).
