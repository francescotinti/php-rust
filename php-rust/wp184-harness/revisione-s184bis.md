# Revisione adversariale S-184 bis — lente PROCESSO
Generato con assistenza AI (Claude Fable 5)

**Verdetto: REGGE CON RILIEVI** — WP regge, i due rc=8 sono meccanici; la mossa oracle-only non risponde al quesito che conta.

## Rilievi

1. **Attribuzione cieca.** Tra t24 (s181) e t25/t26 (s182) oracle e phpr si muovono in direzioni OPPOSTE: oracle 4,89 → 4,80-4,81, phpr netto 34,28-34,30 → 34,46-34,60 (+0,2-0,3 s, limite del rumore ±0,293); rapporto 7,01 → 7,18-7,21, sopra 7,05 in due finestre. Gambe oracle-only non distinguono «oracle più veloce» da «s182 più lento su ORM» (criterio p.1). Manca il controllo phpr s181.

2. **Sentinella rincorsa.** Seconda ri-fondazione in nove sessioni: S-176 la spostò da [4,83;4,94] a [4,84;5,04] perché l'oracle rallentava; ora torna a 4,80. Una banda ±2 % sulla mediana di UNA sessione è più stretta dell'escursione tra sessioni (4,86 → 4,94 → 4,80). Con quattro gambe già a 4,80-4,84 l'esito «<4,84 ⇒ ri-fondare» è scontato. REGOLE 3: emenda dichiarata una volta su base storica.

3. **Rerun senza gate di finestra.** `s184-lancio-orm-leg1.sh` verifica solo lock, CI, phpr vivo, pin: niente anti-flare, loadavg, E2, processo singolo. Cinque ore dopo, utente al computer, 207 file rimossi poco prima: rodaggio oracle 178,8/s. La «scaletta a due estremi» presuppone gambe della STESSA finestra. NEXT p.0 lo chiama «verdetto valido»: è un reperto.

4. **Scoreboard contro il criterio.** p.4: in banda = COMPATIBILE, GIÙ solo <1,738; 1,747 dista 0,023 da t24, meno della banda ON 0,027. La freccia «↓» è il veto «festeggiare una cifra sopra attesa». Il riferimento regge: gambe 01:13-04:04, blocco alle 04:26.

5. **Gate ictx solo relativo.** t26 dbal oracle1 6427/s (t24: 215/s) = «contesa ok»: tutte le gambe contese.

6. **Watchdog fuori repo, orfano ignoto.** Emenda senza versione né manifest. Ucciso `time`, non il figlio phpr: fra 04:41 e 09:23 la sua sorte non è provata.

## Azioni

A. Catena S-185: ORM interleaved oracle / phpr s181 / phpr s182, R≥3, criterio pre-registrato: s181 in [7,01;7,05] e s182 >7,05 ⇒ regola 4 sul pin; entrambi >7,05 ⇒ drift di macchina, emenda E5.
B. Banda sentinella fondata su TUTTE le gambe pulite S-162..S-184, dichiarata una volta; lato veloce = segnalazione, non blocco.
C. Tetti ictx ASSOLUTI per motore e workload; campione CPU per gamba nel `.out`.
D. NEXT p.0: t25bis declassato a reperto; scoreboard WP «=».
E. Copia con hash di `run-with-watchdog.sh` nel harness; i rerun ereditano i gate per copia dichiarata del lanciatore pair.
