# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)

⏱ **FONDAMENTALI**: rif WP **full ON 1,842 / OFF 1,911** (S-109; S-110 sul pin
nuovo: **1,867/1,869 IN banda [1,81;1,88]**, lotto-3 ≤~2%) · media 2,673/2,612
(off SOTTO il rif 2,64, voce aperta vicina a rientrare) · ultima leva di codice
S-109 · S-110: 0 leve di codice (dichiarato), leva misurativa (d) FIRMATA ·
sessioni-senza-Δ-rapporti = 1 · incidenti «mai collaudato»: 1 (de67cb64, S-106).

## Scoreboard (pin 92909544, micro R=5 di S-109 — S-110 non li ha rimisurati)

**arith 9,3 · prop 7,9 · calls 5,1 · str 5,3 · arr 3,9 · re 3,5**
🔬 **TESI FRONTEND FIRMATA (S-110, criterio v2 9ff53cf; soglia ≥2× SUPERATA
con margine, controllo arr pari)**: delivery phpr **arith 32,5% dei cicli vs
oracle 3,3% · prop 10,2% vs 1,7%**. Firma di FAMIGLIA (IC/ITLB/redirect non
ripartita). ⚠️ TETTO revisore: azzerare TUTTA la delivery vale ≤×1,48 su arith
(9,3→~6,3). Verbale+emendamento: `wp110-harness/s110-l1i-verdetto.out`.

## Stato gate

- **phpr pin 929095448e823cb5** @ HEAD (stash `phpr-s109`, ripristinato al byte
  post-churn docs-build) — batteria 1742/0 · corpus **1415 per NOME ×2** (fresco
  S-110: 2652/1415/1238) · fixture 6/6 · ORM 16 nomi · hk 0E/0F · run_loop
  287.944 B · flag-ON · oracle 07b0df8d. Pendente (rev. S-109 az.3): al prossimo
  pin, batteria sul byte O dichiarazione permanente.
- **php-server 443ae42f GRADATO PIENO ×2** (pre-lotto-3; re-pin nominabile) ·
  GitHub SINCRONIZZATO (23c35b0): corpus 2652/1415, blockquote perf riscritto.

## §S-111 — ordine provvisorio

1. **LEVA THREADED-DISPATCH (A/B proprio)** — bersaglio FIRMATO (fame frontend
   arith/prop). Cautele Codex (revisione `20260807-codex.md`, recepite): la
   leva è un **DISCRIMINATORE tra famiglie di dispatch**, non la soluzione
   finale (tetto ×1,48); **PRIMA di progettarla congelare 2-3 giudici HELD-OUT**
   fuori dal ciclo di progettazione (tipi alternati/megamorfici, error-path,
   hot-loop estratto da WP) da leggere SOLO a leva conclusa — anti-overfitting
   ai sei micro. Poi istruttoria forma minima (tail-call handler? clustering
   caldi?) su run_loop; criterio PRE con TETTO ×1,48 su arith (az.4) + rumore
   tra-sere da caratterizzare prima di nuove bande (az.2); guardia:
   contro-lettura delivery post-leva con `s110-l1i-run.sh`. Vincoli: commit+push
   a OGNI passo; admission bipartita; diff prelude se emette.
2. Coppia WP: NON dovuta in apertura (S-110 in banda); torna dovuta se la leva
   1 spedisce. Se la leva si blocca: (c) Sweep/Zval o (b) arr RMW-su-dim previa
   istruttoria; (e) fast-path i64 resta in lista (stime esterne MAI nei criteri).
3. **Backlog PRODOTTO (revisione Codex `20260807-codex.md`, sequenza)**: fuzz
   end-to-end P0 (no-crash su input arbitrario) → stub memory_get_usage da
   implementare-o-rimuovere (correct-or-absent) → soak server fault-oriented →
   CI macOS+fmt → **Laravel come arbitro anti-overfitting**. La rotta resta
   perf-first (decisione utente): questi filoni si aprono su ordine esplicito.
4. Chiusura lean: rotazione + revisore singolo (lente: SEMANTICA).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

media voce aperta (2,673/2,612: sesta lettura con la prossima coppia) · **§3.16
warning undef-var prop-assign** (repro wp109-harness/w9-fixtures/) · causa
frontend NON ripartita (serve kpc/sudo o template GUI — azione utente
nominabile) · retro-A/B str stash s107b/s108/s109 · denti rinviati (OBS-8;
fx20; direct-bind; drop-order; hit/miss) · $z++/$z-- undef non warna · §3.13 ·
§3.12-i · §3.14 · get_gc · drift TODO.md «0%» su aree fatte (Codex; prossimo
gh-status-sync) · bl-count nuovo metodo non confrontabile col «29» storico.

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
**Riscritto**: 2026-08-08 (chiusura S-110 + recepimento revisione Codex).
Storia: `sessions/` · `gaps/GAP_TREND.md` · revisioni in `wp1*-harness/`.

Pre-flight S-111: pin phpr **92909544** (fa fede HEAD; release ripristinato
dallo stash) · server 443ae42f (stash php-server-s109) · MySQL wp8 con elenco
DB · debug/ da rimuovere · uploads sotto guardia · nessuna run in volo ·
disco Data ~15-17G (soglia 15G: OK ma stretta; xctrace SOLO con le guardie di
`s110-l1i-run.sh`) · xcrun xctrace version OK (licenza accettata).
