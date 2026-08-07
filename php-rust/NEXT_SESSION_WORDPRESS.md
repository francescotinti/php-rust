# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)

⏱ **FONDAMENTALI**: riferimento WP = **full 1,894×** (WP-102, citabile) ·
media 2,64× · **coppia WP dovuta entro S-108** · ultima leva = S-106 (H-A1) ·
sessioni-senza-Δ-rapporti = 0 · incidenti «mai collaudato» finora: 1
(de67cb64, S-106 — contatore nuovo, REGOLE §2).

## Scoreboard (pin eb555106, micro R=5, S-106)

**arith 11,6 · prop 10,6 · calls 6,3 · str 6,6 · arr 4,2 · re 3,5**
Il conto del target: oracle arith ≈8,6 ns/iter ⇒ 3× ≈ 26; phpr ≈ 99.
Mancano ~73 ns/iter sulla sola arith: servono colpi strutturali (batch,
dispatch), non solo peephole singole.

## Stato gate

- **phpr pin eb555106a3c7b718** @ HEAD S-106 (stash `phpr-s106`; contiene
  leva H-A1 BinarySTDst) — batteria 1740/0 · corpus **1417 per NOME ×2**
  (=wp82) · fixture 13+5+19a/b+fx20+fx21-gate · run_loop 257.828 B ·
  default flag-ON · oracle 07b0df8d (8.5.7).
- **php-server dde2a64d GRADATO PIENO** (S-106) ma runtime PRE-H-A1: per
  CIFRE server serve regrade a HEAD (scripts/pin-server.sh + launcher
  `wp106-harness/s106-grado-server.sh`); per la coppia CLI basta il pin phpr.
- Concili WP-107/WP-108 ARCHIVIATI in wp107/wp108-harness (vincolanti come
  storia; le regole vive sono SOLO in REGOLE.md).

## §S-107 — BATCH SUPERISTRUZIONI DAL CENSUS (decisione utente 2026-08-07)

1. **Census op/bigrammi sui SEI giudici** (`PHPR_OP_CENSUS=1`, build census
   in `phpr-census-target`, hash del binario census a verbale): classifica
   dei bigrammi/trigrammi caldi PER categoria — lo strumento esiste da
   WP-33, finora mai usato per derivare le finestre.
2. **Top-N candidati di fusione dai dati** (N deciso dalla classifica, con
   stima statica op/iter risparmiate per giudice). Esclusioni: niente
   fusioni che toccano semantica GC/diagnostica (Sweep resta fuori); ogni
   forma nuova riusa gli helper esistenti (zero biforcazione, modello H-A1).
3. **Implementazione A LOTTO nel pass reg_lower** con UN criterio
   pre-registrato (≤10 righe, REGOLE §3: attesa di segno per giudice,
   soglia, R) → admission (diff disasm + taglia run_loop) → smoke →
   **A/B R=5 ABAB per ogni categoria toccata**.
4. **Protocollo PIN intero** in chiusura (REGOLE §5-6) + emendamento
   DICHIARATO delle lettere-gate che mordono (funnel/census op_index:
   già successo con H-A1, succederà di nuovo).
5. Se la finestra regge: **fedeltà §3.15** (cura Zend-esatta ≥ vslot,
   attesa 1417→1415 citata dal fix, golden fx21 aggiornato NELLO STESSO
   commit, gate ORM/hk).
6. Chiusura lean: session file ≤40 righe + **revisore singolo** (lente:
   MISURA) + rotazione.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

coppia WP bimodale (dovuta entro S-108; chain v2 + hash oracle
nell'identity; server nel chain = identità d'ambiente PRE-H-A1) ·
retro-A/B prop coi due stash (riparte il −0,9) · doc «fold rules» di
reg_lower da riallineare alla finestra ST · voci coppia S-105 fuori banda
(full-off 1,947; media 2,697/2,734: rerun, mai bisect) · riesame
threaded-dispatch CON misura (candidato S-108+) · IncDecSlot+Pop (se non
assorbita dal batch) · contatori L1I · lettore per GA_ARGPLACE_DECAY ·
denti rinviati (OBS-8 terza mutazione; mutante fx20; dente direct-bind;
dente drop-order) · fedeltà §3.13/§3.12-i/§3.14/§3.11/get_gc ·
banda-layout terzo punto.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

pin/stash senza collaudo-nell'atto · contenitori sul call path ·
differenze tra A/B distinti come cifra · componenti prezzate nei criteri ·
magnitudine ripartita senza A/B proprio · estensioni BinarySTDst senza
criterio+dente · allargare simple_call senza dente+fx21 · fixture su
memory_get_usage (stub) · «icache-bound» come premessa firmata ·
denominatori a memoria · output di run nel repo.

---
**Riscritto**: 2026-08-07 (adozione processo lean). Apertura/chiusura =
skill `apri-sessione`/`chiudi-sessione` v2. Storia: `sessions/` ·
`gaps/GAP_TREND.md` · concili in `wp107/wp108-harness/`.

Pre-flight S-107: pin phpr **eb555106** (fa fede HEAD, la build churna) ·
MySQL wp8 con elenco DB · debug/ da rimuovere · uploads sotto guardia ·
nessuna run in volo.
