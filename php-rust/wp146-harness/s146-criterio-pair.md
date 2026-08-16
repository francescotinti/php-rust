# S-146 — criterio COPPIA WP @ pin s145 + REPLICA PEAK-ONLY (pre-registrato PRIMA del run)

Obbligo: pin NUOVO s145 (phpr a89faf32c62142f9 + server 4a9adc51a62b21ba) ⇒
coppia dovuta (regola utente 2026-08-12; revisione S-145 reperto #1: il debito
declassa il pin finché non saldato — questo run lo SALDA come PRIMO atto).
Harness: `s146-pair.sh` = COPIA DICHIARATA di `wp142-harness/s142-pair.sh`
(manifest `s146-pair-copia.diff`, collaudo copia-gate) coi SOLI adattamenti:
pin s145; nomi s146/pair146/wp146-harness; **RIMOZIONE dell'inserzione
rewarmup tra leg3 e leg4** (az.rev. S-142 #2: la replica peak-only si fa
SENZA inserzione — questo run la salda) e del suo tag nel giudice; rif e
banda aggiornati al canone post-S-142.

1. Config: 6 gambe TUTTE ON; warm-up media-only bilaterale MAI giudicato;
   emende S-136 (assestamento a STREAK + retry gate ×3) e gate ictx/s a
   mediana PER MOTORE EREDITATE INVARIATE (pair109 INVARIATA).
2. Attesa: **FERMO dentro banda** — L-FR1 (dim-read fuso) non morde WP:
   pattern dim-read-const raro nella suite (census S-145).
3. **Confronto formale**: coppie proprie on-only vs rif S-142
   **[1,765–1,788]** su **banda_ON canonica 0,036** (max-min UNIONE
   S-139+S-140+S-142, PERF_MAP). COMPATIBILE ⇒ banda resta canonica;
   FUORI BANDA ⇒ nessun claim dal singolo tentativo: replica prima di
   ogni delta (canone S-140 p.3).
4. Banda_ON aggiornata: max-min delle coppie ON pulite (N≥5) a verbale;
   canonica post-S-146 = max-min sull'UNIONE S-139+S-140+S-142+S-146.
5. **REPLICA PEAK-ONLY (osservativa, NESSUN gate — salda az.rev. S-142 #2)**:
   peak per gamba a verbale, SENZA inserzione tra le gambe. Lettura
   PRE-REGISTRATA (soglie s142 invariate ≤1825 / ≥1830 MiB): leg1 bassa e
   leg2–6 alte ⇒ pattern S-140 RIPRODOTTO senza inserzione (coerente con
   STATO-post-media; nessuna firma causale da run osservativo); 6/6 alte ⇒
   livello ALTO unico (il doppio livello non si riproduce); 6/6 basse ⇒
   livello BASSO unico; altro ⇒ esito MISTO, si dichiara.
6. **Collaudo del criterio PRIMA della firma (az.rev. S-145 #4)**: giudice =
   blocco python DENTRO `s146-pair.sh` (esiste per nome; derivazione
   meccanica dai `.time`); guardie nominate ED ESISTENTI: gate pin in testa
   allo script · quiescenza per gamba (`wp129-harness/s129-quiescenza.sh`,
   rc su file proprio) · gate ictx/s >1,5× mediana PER MOTORE (dentro il
   giudice) · gate parità per NOME (media diff vuoto; full diff == solo
   `wp_is_stream #2`). Banda 0,036 ≥ risoluzione (tick 0,01 s su ~35 s CPU
   ⇒ ~0,001 sul rapporto). Nessuna guardia nominata fuori dallo script.
7. Finestra: lock misura `/private/tmp/phpr-measure.lock` PRESENTE (runner
   CI in quiet_wait); run CI in volo ESAURITA prima del lancio; pgrep
   rust-analyzer prima del lancio (non-gate: il gate CPU della quiescenza
   arbitra); run DETACHED, sequenziale, unica finestra pesante.
