# Revisione S-137 — lente MISURA (revisore singolo)

## Reperto principale
La sonda FD1 usa lo stesso strumento in due regimi di fiducia opposti: l'identità viene refutata per "artefatto-inlining" (arm probe 56,7 vs ~34,9 implicato dall'A/B: +21,8, +62%), ma l'"indizio dominante" plumbing set-entry 17,6→4,6 = **+13,0** è tenuto a verbale come cifra. Lo stesso verdetto contiene la prova che i delta per-segmento tra le due sonde non sono affidabili: **pop+keys, segmento NON toccato dalla leva, passa 4,5→2,5 (−44%, +2,0 "restituito")** — deriva di calibrazione tra sonde ≥2 ns/segmento su un canale che doveva muoversi di zero, e dispatch+push sale 7,0→9,6 senza ragione nominata. Inoltre l'"artefatto-inlining" è verbalizzato in WP_SESSION_137 come FATTO ("= artefatto del probe (timer nel call-site rompono l'inlining)") senza alcun disasm, contro la regola S-104 (leva su run_loop ⇒ bl-count prima/dopo). Il NON CHIUSA è corretto; è l'attribuzione + l'indizio-cifra che orienteranno S-138 su base non misurata.

## Reperti secondari (numerati)
1. Banda coppia 0,041 = variabilità OFF di S-134, riusata per la terza volta su pin diversi e applicata a un confronto ON-ONLY: mismatch di configurazione della banda. Lo spread on proprio (1,781−1,767=0,014, 7× lo 0,002 di S-136) indica un regime di variabilità cambiato: con banda 0,041 il test ha potere solo su moti >2,3%.
2. Chiusura interna sonda 83% (<90%): 22,1 ns di "fast_altro" non assegnati = 39% dell'arm; il criterio p.3 impone "modello INCOMPLETO dichiarato" — il verdetto stampa solo NON CHIUSA, la dicitura manca.
3. Inerzia patch s137tp dichiarata ma non collaudata (nessun run con/senza); costo del gate: un run.
4. "1 sweep per statement" impreciso: con due statement/iterazione, sweeps main 3.000.001 ≈ 1/iterazione — è 1 per statement CHE NOTA un oggetto (m1 conferma: 2). L'attribuzione regge, la verbalizzazione no. Demoted 3.000.002 > inserted di 1, non spiegato.

## Vagliate e respinte
- Leg1-on ELEVATA al bordo alto: esclusa, on-only = 1,767 secco, ancora dentro [1,736–1,820] — compatibile in entrambe le letture.
- Canone media 2,445–2,529: calcolato ESCLUDENDO leg1-off 2,559 (SEGNALATA) — corretto.
- Costanti s136 nel contrasto: dichiarate "tra sonde omogenee", il limite è già a verbale (assorbito nel reperto principale).

## Azioni S-138 (3-5, concrete)
1. Disasm field_assign_fast su pin s136 vs probe 8dc582d9 (inline sì/no, bl-count): promuovere o refutare l'"artefatto-inlining" PRIMA di ogni criterio-leva.
2. Declassare +13,0 a IPOTESI nel criterio S-138; ogni delta tra sonde porta banda di non-omogeneità ≥2,0 ns/segmento (fondata su pop+keys).
3. Probe arm-only (timer fuori dal call-site, nessun sotto-segmento) sul sorgente s136: se arm ≈34,9±13,3 l'identità chiude senza ripartizione.
4. Rifondare la banda coppia: variabilità ON-config sul pin corrente (N≥5), pre-registrata per configurazione; pensionare lo 0,041 S-134.
5. Gate inerzia s137tp: run census con/senza patch, conteggi attesi identici.

**[Applicata in S-137]**: verbalizzazione corretta nei file di rotazione — "artefatto-inlining" declassato a IPOTESI (disasm dovuto S-138) in WP_SESSION_137 e NEXT_SESSION.
