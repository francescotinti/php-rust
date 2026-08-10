# s127-submicro-letture.md — letture pre-registrate applicate (cifre da s127-submicro-verdetto.out)

Run valido = SECONDO (il primo è a verbale in `s127-submicro-verdetto.contaminato.out`:
**incidente apparato S-127 #1** — indicizzatore LSP avviato DENTRO la finestra di misura;
segni: spread objchurn phpr 1,90 s vs 0,03 del run pulito).

ns/iter netti (R=5, pin s125 002e6cc1):

| lato | objalloc | objchurn | objmap | objdropdef | objdatains | objallocni |
|---|---|---|---|---|---|---|
| phpr | 1220,0 | 1806,7 | 173,3 | 1483,3 | 1540,0 | 1150,0 |
| oracle | 126,7 | 176,7 | 10,0 | 136,7 | 163,3 | 93,3 |

Letture (criterio p.3):
- **Δins** (insert su `$e->data` + drop array non vuoto) = phpr **320,0** · oracle 36,6.
- **Δdrop** (drop differito via overwrite mappa vs immediato) = phpr **+90,0** · oracle **0,0**
  (il sentiero di drop alternativo per l'oracle è gratis; per phpr no).
- **CHIUSURA additività**: |objchurn − (objdropdef + Δins)| = **3,4 ns** su ENTRAMBI i lati
  ≤ soglia 4 ns ⇒ additività CHIUSA; il residuo 427 ns di S-126 (qui 413,4) = 320 Δins + 90 Δdrop.
- Il **67%** di L-OL1 regge con additività verificata: objalloc/objchurn = 1220/1806,7 = **67,5%**.
- **objallocni − objalloc = −70 ns phpr / −33,4 oracle**: l'interpolazione `"n$i"` DILUIVA il
  rapporto (9,6 → **12,3×** senza) — il ciclo-di-vita puro è più sbilanciato di quanto pubblicato.
  Cifra registrata, nessuna decisione (criterio p.3); il giudice A/B di L-OL1 resta objalloc.php
  come pre-registrato in s126-leva-nominata.md.
