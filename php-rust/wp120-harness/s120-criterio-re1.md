# s120-criterio-re1.md — leva L-RE1 (classifica C-lite: re +12 alloc/iter), PRE-registrato

1. Leva: hot path `ho_preg_match` — (a) borrow di pattern/subject (via i `to_vec`, host.rs:3974-3976); (b) fast-path senza nomi in `captures_array` (`Vec::new()` non alloca); (c) materializzazione OWNED del testo gruppi (move in `PhpStr`, variante owned di captures_array/capture_value); (d) riuso `CaptureLocations` nell'arm `Engine::Regex` (wrapper Deref + `captures_read_at`). Runtime-only, php-types INTOCCATO (lezione S-119); composta su HEAD, target canonico via build-script, MAI il pin.
2. Meccanismo (firma, PRIMA del tempo): census C-lite (patch wp119-harness + re1.patch, target separato) su re: alloc/iter 17,00 → atteso ≤10,00; nessuna categoria non-bersaglio con alloc/iter in AUMENTO; R=2, conteggi attesi identici.
3. Giudice del tempo: forma s119-ab (floor med3 per-binario, famiglia 1,3×min, coppie extra ≤6), A = pin s119 (stash phpr-s119), B = candidato (stash phpr-s120-re1); N dal sorgente; R=5 ABAB.
4. Bersaglio re — segno atteso D_med(re) > 0 (B più veloce); soglia promozione: D_med(re) ≥ max(banda micro N=2 re = 10,00; 2×spread_A; 2×quanto) — la banda-v2 re 0,00 è SOSTITUITA dalla banda micro (apertura NEXT_SESSION trattata qui).
5. Guardie non-bersaglio a SOLO-REGRESSIONE (banda-v2): arith 0,80 · prop 3,33 · calls 0,50 · str 7,50 · arr 6,67; soglia = −max(2×spread; banda; 2×quanto), tie ⇒ tiene; morso calls dentro 0,50 si REGISTRA.
6. Admission: 6/6 parità output + dump INTERO byte-id — DEROGA NOMINATA forma S-118 (leva runtime-only a emissione INVARIATA), citata qui.
7. Smoke R=2 di guardia: early-stop SOLO su regressione concorde ≤ −1,00; il resto si registra, la banda giudica nel full (lezione S-119: lo smoke post-build può mentire).
8. Giudice del rischio (promozione): batteria `cargo test --release` rc da FILE · corpus-gate canonico ×2 modi (fail-set 1415 per NOME) · fixture bilaterali · pin SOLO via `scripts/pin-phpr.sh` (ricetta A′), churn dichiarato.
9. Bisezione pre-registrata: meccanismo mancato o guardia sfondata ⇒ stacca (d), poi (c); max 2 giri.
10. Cifre solo dai `.out` (`s120-re1-verdetto.out`); strumentazione MAI nei sorgenti del pin.
