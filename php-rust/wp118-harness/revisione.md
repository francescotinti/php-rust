# Revisione S-118 (revisore singolo, lente: PROCESSO)

## Verifiche fatte
- Ordine temporale: criterio R3 9cf1fcd (21:00:26) PRECEDE tutti i raw (gate1 21:05-21:08, grado off/on 21:08→21:46, pair109 off 22:10→22:36 / on 22:36→23:02, census 23:02→23:20). Criterio treno-1 9013b2b (23:06:42) PRECEDE build-cand (23:23), admission (23:23-24), smoke (23:27), full (23:29), held-out (23:33). Nessuna misura prima del suo criterio.
- Ordine REGOLE §6 nel promo-out (mtime): build 23:37 → hash-check → batteria 23:42 → re-build/re-hash 23:42 → pin/stash 23:42:29 → corpus →23:54 → fixture 23:54 → micro 23:55 → held-out 23:57. Tutti gli rc da FILE scritti dal `$?` del comando, mai da pipe.
- A/B: `run_pair` esegue A poi B per coppia, ×5 → ABAB interleaved (solo i floor sono mediana-di-3 a burst, macchina s117 invariata, dichiarata). Durate raw coerenti con le cifre (full ≈125 s, micro R=5 ≈65 s): run reali.
- pair109 stessa sera: epoch 1786219820 (off) / 1786221398 (on), Δ26 min; release==pin 1656580e ×2; failnames diff = solo wp_is_stream.
- Byte-identità: candidato 15dfb6b3 riprodotto dalla ricetta DUE volte (post-commit e post-batteria) con STOP su mismatch; stash via cp+re-hash; pin solo da pin-phpr.sh. Nessun salto di fiducia.
- Deroga «admission sul dump intero» (trappola S-117): dichiarata NEL criterio PRIMA della run, con motivazione (leva runtime-only ⇒ emissione invariata); non nominata però come deroga alla lista trappole.

## Il punto che ridimensiona
La catena documentale del treno-1 è fuori dal repo: `s118-treno1-verdetto.out` (unica fonte di prop +5,33, guardie, held-out, PROMOZIONE COMPLETA) risulta MAI committato (`??` in git status) e `pair109-ratios-{off,on}.out` sono modificati ma non committati — mentre session file, REPORT_GAP e NEXT_SESSION che ne CITANO le cifre sono committati e pushati. «Commit+push a ogni passo» (§10) e «nessuna cifra fuori dai .out» valgono solo sul disco: il repo pubblica cifre di cui non possiede la fonte. Aggravante minore: REPORT_GAP dichiara la media «ON minimo storico della voce» e il peak «minimo famiglia» — ranking cross-pipeline contro la pre-registrazione «nuovo riferimento senza soglia, tra-sere solo direzione» (§4); la riga full invece è pulita.

## Verdetti
- Claim 1 «R3 saldato 4/4»: **REGGE** (criterio prima, 4 gate stessa sera, rc/void da file, raw presenti); il contorno «minimo storico» è FUORI criterio, da emendare.
- Claim 2 «treno-1 promosso con §6 pieno»: **REGGE nel merito** (ordine §6, ABAB, byte-identità ×2, gate canonici) ma **RIDIMENSIONATO in tracciabilità**: verdetto non committato = promozione non auditabile dal solo repo.

## Azioni (S-119)
1. Committare subito s118-treno1-verdetto.out + pair109-ratios-{off,on}.out; ripulire gli untracked residui in wp109-harness.
2. Emendare REPORT_GAP_118: «minimo storico/famiglia» → direzione-solo, finché la serie sotto A′ non ha N≥2.
3. Saldare il debito dichiarato: WP full/media + repin/grado server sul pin s118 (il riferimento nuovo è sul pin vecchio).
4. Ogni deroga a una trappola della lista va NOMINATA come tale nel criterio, con la trappola citata.
5. Held-out: banda A′ N=2 (0,01 s su poly) troppo sottile come guardia — portarla a N=3 prima del prossimo giudizio.
