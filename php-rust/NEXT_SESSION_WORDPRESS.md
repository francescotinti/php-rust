# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)
⏱ **FONDAMENTALI**: rif WP **full ON 1,867 / OFF 1,869** (S-110, banda [1,81;1,88])
· media 2,673/2,612 (voce aperta) · ultima leva di codice SPEDITA S-109 · S-111:
leva TENTATA con A/B pieno e REFUTATA (ritmo rispettato, 0 spedite dichiarato) ·
sessioni-senza-Δ-rapporti = 2 · incidenti «mai collaudato»: 1 (de67cb64, S-106).

## Scoreboard (pin 92909544, micro R=5 di S-109 — S-110/111 non li hanno rimisurati)

**arith 9,3 · prop 7,9 · calls 5,1 · str 5,3 · arr 3,9 · re 3,5**
🆕 **HELD-OUT baseline (prima lettura S-111 sul pin, R=5, wp111-harness/heldout/):
poly 6,7 · err 2,6 · wploop 5,6** — d'ora in poi GUARDIA STANDARD di ogni leva
(lettura SOLO a leva conclusa; soglia held-out da PRE-registrare prima della
prossima lettura — az.5 rev. S-111). 🔬 Tesi frontend (S-110) raffinata da
S-111, in DIREZIONE (quote per-motore, forma pre-filtro): delivery arith
0,325→0,295 coi 4 handler ADIACENTI, residuo ~0,30 ⇒ lo scatter dei caldi non
regge come motore; salto-indiretto/fetch-helper = ipotesi da MISURARE. Tetto
×1,48 su arith resta il tetto di OGNI leva solo-frontend.

## Stato gate

- **phpr pin 929095448e823cb5** @ HEAD (stash `phpr-s109`; release VERIFICATO al
  byte a fine S-111 dopo revert) — batteria 1742/0 (rc=0, su build leva S-111;
  churn relink dichiarato) · corpus **1415 per NOME ×2** (fresco S-110) · fixture
  6/6 (S-110) · run_loop 287.944 B · flag-ON · oracle 07b0df8d. Build fresca
  dello stesso sorgente = e0a3ea6a (divergenza NOTA S-110/111: il pin si
  ripristina SOLO dallo stash).
- **php-server 443ae42f ×2** (stash s109; release = residuo) · GitHub sync S-110.

## §S-112 — ordine provvisorio

1. **LEVA PERF** (ritmo: una per sessione, qualunque verdetto). Candidati con
   istruttoria PRIMA del criterio: **(c) Sweep/Zval** (ciclo di vita Zval = collo
   dichiarato S-101) o **(b) arr RMW-su-dim** (cifra S-108: +11,67 ns/inner-iter);
   (e) fast-path i64 resta in lista (stime esterne MAI nei criteri). Vincoli:
   criterio PRE con soglia max(4 ns/iter; rumore; banda-layout 0,67) + guardie
   esplicite sui NON-bersagli (hanno deciso S-111) + held-out a leva conclusa +
   disasm bl-count prima/dopo + commit+push a ogni passo + admission bipartita.
2. **Causa fine frontend** (se blocca o se l'utente la ordina): il residuo
   delivery ~0,30 punta a redirect/BTB del salto indiretto o fetch degli helper
   outlined — misurarla vuole kpc/sudo o template GUI (AZIONE UTENTE nominabile).
   Az.2 S-110 (rumore tra-sere: 3 serate stesso pin, spread pubblicato) resta
   DOVUTA prima di ogni nuova banda WP.
3. **Backlog PRODOTTO (Codex `20260807-codex.md`, sequenza)**: fuzz end-to-end
   P0 → stub memory_get_usage correct-or-absent → soak fault-oriented → CI
   macOS+fmt → **Laravel arbitro anti-overfitting**. Rotta perf-first: si aprono
   SOLO su ordine esplicito.
4. Chiusura lean: rotazione + revisore singolo (lente: PROCESSO).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

media voce aperta (2,673/2,612: sesta lettura con la prossima coppia) · rumore
tra-sere WP (az.2 S-110) · **A/B pre-filtro VUOTO** (az.2 rev. S-111: ripartire
tassa-filtro vs flip-inliner) · causa fine frontend NON ripartita (kpc/sudo —
azione utente) · **§3.16/§3.17 righe dei warning** (famiglia §3.13) ·
retro-A/B str stash s107b/s108/s109 · denti rinviati (OBS-8; fx20; direct-bind;
drop-order; hit/miss) · $z++/$z-- undef non warna · §3.13 · §3.12-i · §3.14 ·
get_gc · drift TODO.md «0%» su aree fatte (prossimo gh-status-sync) · bl-count
metodo nuovo non confrontabile col «29» storico.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze tra
A/B distinti come cifra · componenti prezzate nei criteri · magnitudine
ripartita senza A/B proprio · finestre estese senza criterio+dente · simple_call
allargato senza dente+fx21 · fixture su memory_get_usage (stub) · «icache»
NON-premessa · **pre-filtro/hot-cluster (REFUTATA S-111; tassa non ripartita
dal flip inliner) senza forma che non tassi i freddi** · denominatori a
memoria · output di run nel repo · rc di gate da
pipe (4 morsi) · tee/log prima del mkdir · finestre fuse oltre un helper
sospendibile · admission sul dump intero · xctrace senza guardie disco · path
del volume esterno nel figlio tracciato (TCC).

---
**Riscritto**: 2026-08-08 (chiusura S-111). Storia: `sessions/` ·
`gaps/GAP_TREND.md` · revisioni in `wp1*-harness/`.

Pre-flight S-112: pin phpr **92909544** (HEAD fa fede; release GIÀ al pin) ·
server 443ae42f (stash) · MySQL wp8 con elenco DB · debug/ da rimuovere ·
uploads sotto guardia · nessuna run in volo · disco Data ~16G (stretto: xctrace
SOLO con guardie s110/s111) · held-out baseline 6,7·2,6·5,6 (wp111-harness).
