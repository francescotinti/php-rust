# REPORT_GAP_105 — coppia WP bimodale S-105 (SOLO questa sessione)

**Quando**: chain detached 23:03→00:01 (2026-08-06/07), lettura 00:1x
col protocollo pre-registrato (banda KS-GR-107-3, precondizioni
A-PE-107-3). **Binari**: phpr **d4d0fa5217515dd9** (pin S-105, leva args
inclusa) · php-server de67cb64 (presente, NON esercitato dal collaudo
CLI) · oracle 8.5.7. Raw: `wp105-harness/pair-out-{off,on}/`, ratios
`wp105-harness/pair105-ratios-{off,on}.out`, lettura
`wp105-harness/pair105-lettura.out`.

| metrica | off | on | banda pre-registrata | esito |
|---|---|---|---|---|
| full CPU ×oracle | 1,947 | **1,894** | [1,84;1,89] | on sul bordo ≡ WP-102 (conferma); off FUORI +0,056 (voce aperta) |
| media CPU ×oracle | 2,697 | 2,734 | [2,57;2,64] | FUORI sfavorevole +2/+4% (voce aperta; media = metrica ad alta varianza tra-sere) |
| full peak phpr MiB | 1897,5 | 1867,3 | — (storico 1863-1998) | in famiglia |
| parità per NOME | media 0 fail ×2 · full = SOLO wp_is_stream pre-esistente | | | ✓ nei 2 modi |

**Lettura Gregg (f̂ pre-registrata)**: f̂(on) ≈ −0,015 ⇒ l'effetto della
leva args (+23 ns/iter su calls, −14% sulla categoria) è
**indistinguibile sul full CPU WordPress** — atteso e coerente con la
spina dorsale: l'aggregato WP è diluito (I/O/builtin/DB), il giudice
della perf del nucleo resta la micro-categoria.

**Debito KS-PE-106-2: SALDATO** (trigger leva promossa → coppia stessa
notte). Voci APERTE per NOME (non verdetti): full-off sopra banda
(+0,056); media fuori banda nei 2 modi (rerun prima di ogni
attribuzione). Il trend va in `GAP_TREND.md` (riga WP-105 aggiornata).
