# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)

⏱ **FONDAMENTALI**: riferimento WP = **full ON 1,842× / OFF 1,911×** (S-109;
lettura S-110 sul pin nuovo: **1,867/1,869 DENTRO banda [1,81;1,88]** — debito
lotto-3 ASSOLTO, stasera ON~OFF) · media 2,673/2,612 (quinta lettura: entrambe
in discesa, off SOTTO il rif 2,64 — voce aperta vicina a rientrare) · ultima
leva di codice = S-109 (lotto-3 str) · **S-110: 0 leve di codice (dichiarato),
leva MISURATIVA (d) FIRMATA** · sessioni-senza-Δ-rapporti = 1 (micro fermi, pin
invariato) · incidenti «mai collaudato»: 1 (de67cb64, S-106).

## Scoreboard (pin 92909544, micro R=5 di S-109 — S-110 non li ha rimisurati)

**arith 9,3 · prop 7,9 · calls 5,1 · str 5,3 · arr 3,9 · re 3,5**
🔬 **TESI FRONTEND FIRMATA (S-110, criterio v2 9ff53cf)**: delivery-share
phpr/oracle **arith 9,75× (32,5% vs 3,3% dei cicli) · prop 5,96× · controllo
arr 1,04×** (specificità ✓); discarded 30×/169× su quote piccole. Firma di
FAMIGLIA frontend — causa IC/ITLB/redirect NON ripartita (limite strumento).
Verbale: `wp110-harness/s110-l1i-verdetto.out`, raw in `l1i-out/coll/`.

## Stato gate

- **phpr pin 929095448e823cb5** @ HEAD (stash `phpr-s109`, ripristinato al byte
  in chiusura S-110 dopo churn docs-build) — batteria 1742/0 · corpus **1415
  per NOME ×2** (fresco S-110: 2652/1415/1238) · fixture 6/6 · ORM 16 nomi =
  baseline · hk 0E/0F · run_loop 287.944 B · flag-ON · oracle 07b0df8d. Pendente
  (rev. S-109 az.3): al prossimo pin, batteria sul byte O dichiarazione permanente.
- **php-server 443ae42f GRADATO PIENO ×2** (pre-lotto-3; re-pin nominabile).
- GitHub SINCRONIZZATO (23c35b0): corpus 2652/1415, blockquote perf riscritto.

## §S-111 — ordine provvisorio

1. **LEVA THREADED-DISPATCH (esperimento con A/B proprio)** — il bersaglio è
   FIRMATO (fame frontend su arith/prop). Istruttoria PRIMA: forma minima
   (tail-call sui handler? dispatch table computed-goto-like? clustering dei
   handler caldi?) su run_loop; criterio PRE-registrato (giudici arith+prop,
   soglia, R, disasm bl-count prima/dopo); guardia: contro-lettura delivery
   post-leva con lo stesso apparato S-110 (`s110-l1i-run.sh` riusabile).
   Vincoli revisore ATTIVI: commit+push a OGNI passo; admission bipartita;
   se la leva emette bytecode: diff del prelude enumerato.
2. Coppia WP: NON dovuta in apertura (S-110 in banda); torna dovuta se la leva
   1 spedisce (collaudo aggregato sul pin nuovo).
3. Se la leva 1 si blocca: (c) Sweep/ciclo-vita Zval o (b) arr RMW-su-dim
   [FetchDim;BinarySTDst] previa istruttoria; (e) fast-path i64 op fuse resta
   in lista (parere esterno vagliato, stime esterne MAI nei criteri).
4. Chiusura lean: rotazione + revisore singolo (lente: SEMANTICA).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

media voce aperta (2,673/2,612: sesta lettura con la prossima coppia) ·
**§3.16 riga errata warning undef-var ricevitore prop-assign** (bilaterale,
repro in wp109-harness/w9-fixtures/; w9a caso B rientra nel gate quando curata)
· causa frontend NON ripartita (IC/ITLB/redirect: serve kpc/sudo o template
GUI — azione utente nominabile) · retro-A/B str stash s107b/s108/s109 · denti
rinviati (OBS-8; fx20; direct-bind; drop-order; hit/miss) · fedeltà: $z++/$z--
undefined non warna · §3.13 · §3.12-i · §3.14 · get_gc · bl-count run_loop
metodo nuovo (otool 5849) non confrontabile col «29» storico.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze tra
A/B distinti come cifra · componenti prezzate nei criteri · magnitudine
ripartita senza A/B proprio · estensioni di finestre senza criterio+dente ·
allargare simple_call senza dente+fx21 · fixture su memory_get_usage (stub) ·
**«icache-bound» resta NON-premessa: la firma S-110 è di FAMIGLIA frontend, la
causa fine va misurata prima di prezzarla** · denominatori a memoria · output
di run nel repo · rc di gate da pipe (4 morsi) · tee/log prima del mkdir ·
finestre fuse OLTRE un helper sospendibile · admission d'emissione sul dump
INTERO ({main} il perimetro; diff prelude enumerato quando emette) · xctrace
senza guardie disco (TMPDIR interno = 15G di ktrace, rc=134, S-110) · path del
volume esterno nel figlio tracciato (TCC headless blocca open()).

---
**Riscritto**: 2026-08-08 (chiusura S-110). Apertura/chiusura = skill v2.
Storia: `sessions/` · `gaps/GAP_TREND.md` · revisioni in `wp1*-harness/`.

Pre-flight S-111: pin phpr **92909544** (fa fede HEAD; release ripristinato
dallo stash) · server 443ae42f (stash php-server-s109) · MySQL wp8 con elenco
DB · debug/ da rimuovere · uploads sotto guardia · nessuna run in volo ·
disco Data ~15-17G (soglia 15G: OK ma stretta; xctrace SOLO con le guardie di
`s110-l1i-run.sh`) · xcrun xctrace version OK (licenza accettata).
