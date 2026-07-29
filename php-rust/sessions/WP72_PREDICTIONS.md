# predictions72 — pre-registro della LEVA S-72.4 (mass-teardown) — lock PRIMA di ogni letto post-fix

Binario leva: release post-`d5fd1c9` (repo php-rust). Baseline pre-fix:
tripla WP-71 = 20,000 obj/req ESATTO · 2,1109 KiB/req (memgc71,
spread 0,000/0,0069); amp71 K=10 = +80,000 obj/req; uc-steady =
miss 512 (solo R1) · hit_cross 512/512; ladder release 1,078
(~2,0 KiB/req phys). Metro di ogni riga dichiarato nella riga.

## Bande LOCKATE (G-72.2 + composizione concilio)

| Riga | Metro | Banda PASS | Note |
|---|---|---|---|
| T-72.a tripla post-fix used_n | census memgc72, 3 leg N=1000, slope obj/req | **[−0,009, +0,5]** | (0,5, 1,5] = PARZIALE con RI-ATTRIBUZIONE obbligatoria (mai "rumore"); >1,5 = FAIL (KL72-1/KG72-1) |
| T-72.b tripla post-fix used_b | idem, KiB/req | **< 0,15** | |
| T-72.c per-bin | stessi 5 canali della firma WP-71 (96\|112 straddle ecc.) | **→ 0 (±0,009)** | **160\|192 DECIDE C4**: >0,2 obj/req = canale separato, letto dedicato (KG72-2) |
| T-72.d contatore leva | tag=teardown per-req su wpdev: broken/req | banda FISSATA DA B-72.1 (istogramma pre-letto, addendum additivo) | KL72-2: fuori banda = la leva raccoglie altro/non tutto |
| T-72.e busy | tag=teardown busy | **== 0 SEMPRE** | K-M72.2: >0 = STOP leva |
| T-72.f alive_after | tag=teardown alive_after/req | banda da B-72.1 (VM-field-held attesi) | KH72-1: fuori banda alta = radicamento incompleto |
| A-72 controllo negativo | amp71 K=10 POST-fix, delta obj/req vs base post-fix | **[−0,5, +5]** | G-72.3: i cicli sintetici ORA raccolti; fuori banda = meccanismo non chiuso |
| U-72 uc-steady | uc1.log della tripla: miss / hit_cross | **miss==512 (solo R1) ∧ hit_cross==512/512** | E-72.H2/KS72-H2: miss oltre R1 = leva FERMA (regressione interning) |
| C-72 cap CPU | coppia stessa-sera: media-group user CPU mediana + full master-CPU | **≤ +0,5 ms/req mediana media E ≤ +1% full** | B-72.2/KB72-1/KG72-3/KL72-3: sopra cap = NON si spedisce; fast-path store-vuoto = zero lavoro. B-72.1 può RIDIMENSIONARE solo con addendum additivo derivato dal letto istogramma, mai post-hoc sul letto CPU |
| P-71.3 (forma L-72.1) | ladder release three-boot 1k/5k/9k, /usr/bin/time -l phys | **d12 e d23 ciascuno < 2 MiB su ΔN=4000** | rumore del metro DICHIARATO: two-boot lossy sotto-legge ~30% (5,8 vs 8,25) e σ boot-to-boot ~1,5–2,5 MiB ⇒ banda NON-CONCLUSIVA = d ∈ [2, 4] MiB (né PASS né FAIL: si cita il census tripla come metro primario, L-72.1); d > 4 MiB = FAIL |

## Regole di lock

- Catena ADDITIVA (WP-71): ogni addendum si appende, hash per
  versione via git (wp72-harness repo); NESSUNA riga lockata si
  riscrive. Max 2 re-lock derivati per esperimento (KK72-1).
- Ogni leg da RUNNER committato in questo repo, hash pre-letto = il
  commit del runner PRECEDE il mtime del letto (P-72.1/K-72.2;
  KS-P72.1: leg inline = NULLO).
- Contratto canonico wpdev ESTESO: assert mu-plugins/ vuota in
  apertura E chiusura di ogni leg (P-72.2/KS-P72.2); kill server per
  PID (P-72.5).
- Etichetta G-72.1 su ogni citazione del libro mastro WP-71:
  "attribuzione via escalation KL71-2, causa FUORI tassonomia
  lockata"; C4 = ASSORBITA-NON-MISURATA (mai "0"), decisa da T-72.c.
- Verdict SOLO da script committato; bddelta (se riusato) = identità
  contabile + coda negativa (G-72.4), conteggi per-pattern MAI citati
  come misura (KG72-4); blockdump solo con buffer fisso (L-72.5) —
  in WP-72 NON si riusa il blockdump se non necessario.
- KS-S72.3: ogni letto su build dove gate integrale regredisce = NULLO.
