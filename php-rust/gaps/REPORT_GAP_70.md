# REPORT_GAP_70 — misure della SOLA sessione WP-70 (2026-07-29)

Gap classico (media CPU / footprint media / full CPU) NON rimisurato
(sessione fedeltà + caccia al residuo). Riferimenti invariati: media
CPU 2,58× · footprint media ~3,0-3,1 (banda 2,9-3,2) · full CPU
2,06-2,11× · peak ~1,98-2,03 GB · server hit_cross 512/512.

## Misure della sessione (caccia KL69-1, predictions70 lock cbf251fe)

- **P70-0-bis (RELEASE strumento-free, two-boot vmmap)**: footprint
  185,0 MiB @N=1000 → 190,8 MiB @N=5000 ⇒ Δ=5,8 MiB su ΔN=4000
  (≈1,45 KiB/req lato phys) ∈ banda [5,15] ⇒ **ESISTE-SU-RELEASE**
  (ipotesi artefatto-census refutata).
- **P70-T (tripla census memgc70, 3 boot indipendenti, HITS-off +
  DISABLE_WP_CRON, finestre [R100,R1000] con assert macchina)**:
  used_b slope LS 2,1062 / 2,1121 / 2,1302 ⇒ **mediana 2,1121
  KiB/req** (endpoint 2,0939/2,1198/2,1366); used_n slope 20,000 /
  20,000 / 20,003 ⇒ **mediana 20,000 obj/req**; firma per-bin (n/req):
  112→8,3-9,0 · 128→6,000 · 64→2,98-3,00 · 160→0,88-0,92 · 32→1,000
  (tutte ±15% dalle attese lockate). **Self-spread: 0,024 KiB/req ·
  0,003 obj/req = MDE fondato per le cacce future.**
- **P70-D (defermini)**: calls 16,00/req esatto; net_b (flow-form,
  clamp per-call) 1526 KiB/req = churn ~95 KiB/defer — non misura il
  ritenuto; attribuzione APERTA.
- Corpus: 1422→1421 (−1 fix reale). Cargo 1650/0. GATE70 PASS 1° run.
