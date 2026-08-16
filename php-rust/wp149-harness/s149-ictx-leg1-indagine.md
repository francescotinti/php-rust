# s149-ictx-leg1-indagine — perché leg1 phpr esce SEGNALATA (t1+t2) — grado INDIZIO (lettura verdetti+harness, nessuna misura nuova)

1. Firma nei due tentativi: phpr ictx% della mediana MOTORE decade monotono
   sulle prime gambe (t1: 144%→121%→94-102%; t2: 202%→138%→90-100%); oracle
   resta 97–105% ovunque. La firma è di FINESTRA, non di gamba difettosa.
2. In ASSOLUTO l'eccesso è piccolo e su ENTRAMBI i lati: t2 leg1 phpr 177/s
   vs mediana 88 (+89/s); oracle leg1-2 1111–1119 vs 1064 (+47–55/s). Un
   disturbo di ~+50–90 preemption/s a inizio finestra, in decadimento, è
   +100% sulla mediana phpr (88/s) e +4–5% su quella oracle (1064/s): il
   giudice in % del motore amplifica il lato a baseline BASSA.
3. Meccanismo candidato (dal harness): il warmup è SOLO `--group media`;
   leg1 è la PRIMA full-suite della finestra — first-touch dell'albero WP +
   wipe/restore uploads (guardia) + DROP/CREATE wptests ⇒ daemon di
   indicizzazione/FS in coda, in decadimento su leg1–leg2. Coerente con
   deriva peak t2 e col fatto che la firma NON dipende dal pin.
4. Proposta per il criterio t3 (da PRE-REGISTRARE, non retroattiva):
   (a) warmup ESTESO: una full phpr NON giudicata prima di leg1 (warm della
   finestra sul cammino giudicato, bilaterale per simmetria);
   (b) firma letta ANCHE in assoluto (eccesso ictx/s vs mediana motore) a
   corredo della % — la % resta il gate, l'assoluto disambigua;
   (c) sampler osservativo dei daemon (ps ogni 30 s, top-3 CPU non-motore)
   per NOMINARE la sorgente se leg1 esce di nuovo elevata.
5. Nessun rigetto retroattivo: t1/t2 restano come giudicate (leg1 esclusa
   dove SEGNALATA per il criterio vigente); la deriva N=6 si applica a t3.
