# Revisione S-115 — lente: PROCESSO (revisore singolo, REGOLE §7)

**Verifiche fatte.** Git log 1a29438→HEAD coi timestamp; diff di 36b286e (4-bis committato 14:34, PRIMA di patch 14:36, admission 14:37, misura 14:38-44); i due criteri; i tre script; i due verdetti; raw ab-out/ e heldout-out/ con mtime (ordine criterio→apparato→run→verdetto rispettato ovunque); s114-criterio-la.md p.5; run-heldout.sh originale. Esclusioni prop nominate nei raw; entrambe le coppie lente su ENTRAMBI i lati (Δ +44/+62 esclusi ⇒ mediano DEPRESSO: conservativa). Non-monotonia famiglie: loop e verdetto ricalcolano sulla serie intera con la stessa regola — coerente.

**Il punto che ridimensiona.** Il termine decisivo del gate held-out — «spread corrente» (S-114 p.5, ereditato) — non è definito nel criterio: lo script lo risolve in spread del SOLO candidato (0,02 → pavimento 0,12 → limite 9,71). La lettura alternativa, ugualmente compatibile col testo (spread del pin, o max dei due), con spread pin da 0,15 DOCUMENTATI in-sessione (nulla-2, stessa macchina) avrebbe dato limite fino a 9,89: poly 9,86 PASSA e L-A si promuove. Il verdetto NON PROMOSSA — e l'attribuzione costruita sopra — poggia su un'interpretazione fissata nello script (pre-run: 14:29 vs 14:32, ma non nel criterio) di un termine ambiguo potenzialmente outcome-flipping. Lo spread pin della run L-A non fu nemmeno pubblicato: irricostruibile.

**Verdetti.**
1. REGGE con perimetro: +26,33 è la magnitudine NELLA famiglia veloce (caveat p.9); la deviazione min-vs-mediana fu disegnata sui dati S-114 ma dichiarata pre-run e stavolta ha agito CONTRO la leva.
2. RIDIMENSIONATO: la nulla-2 refuta la DIAGNOSTICITÀ del gate, non attribuisce — banda misurata 0,20 < deficit 0,26, N=1, patch diversa (+1.676 B in Op::Clone vs +3.176 B); «non è più solo un'ipotesi» è sovradichiarato.
3. RIDIMENSIONATO: ordine commit impeccabile, ma tre vizi: termine decisivo del gate non pre-registrato; rc admission letto dall'echo interno via pipe (viola la regola scritta nel criterio stesso, sesto morso); riga «revert verificato» scritta PRIMA della verifica (verbale-profezia, mitigata dal commit posticipato). Aggravante: calls −7,00 = soglia esatta, «tiene» per un epsilon di float su un tie non pre-registrato.

**Azioni.**
1. Criterio S-116: definire «spread corrente» (proposta: max(pin, candidato)) e pubblicare ENTRAMBI gli spread nei raw held-out.
2. Declassare nel verbale nulla-2 p.3 «attribuito» in «gate refutato come diagnostico»; banda held-out a N≥2 con zavorra paragonabile a L-A prima di riusare la parola.
3. Rc di gate scritti dagli script in un FILE (admission-out/rc), mai letti da echo/pipe; vale anche per BUILD_RC.
4. Le righe di esito nei verbali le appende lo script di verifica a esito acquisito, mai il conduttore in anticipo.
5. Pre-registrare il trattamento dei tie esatti sulle soglie.
