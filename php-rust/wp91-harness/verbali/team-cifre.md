# TEAM-CIFRE — Concilio WP-91 (relatore: sedie 3, 4, 9)

Verbali fonte (vincolanti): verbale-3-klabnik.md, verbale-4-hejlsberg.md, verbale-9-gregg.md.

## CONVERGENZE (emendamenti unificati)

1. **Provenienza committata per OGNI anello della catena** — A-SK60+A-SK63+A-SK64 (Klabnik: operandi [derivata] risolti per provenienza `file:riga` a HEAD; allowlist/perimetro da MANIFEST committato, non regex di nome) ↔ A-AH54 (Hejlsberg: triangolo attempts↔stamp↔OUT con sha256==DSHA) ↔ A-BG53 (Gregg: giudice committato PRIMA del run). Stesso principio: nulla è verdict-grade se non risolve a blob committato.
2. **judge_sha g2 dangling** — A-AH56+KS-AH-91-2 (Hejlsberg) ≡ BUCO 1+KG-91-1 (Gregg): g2 `24cd290ae0a9fc2b` non risolve ad alcun blob; entrambi esigono generazione dichiarata "judge-unrecoverable"/VOID-of-record e non citabile. Verificato indipendentemente da entrambe le sedie.
3. **Dente generazioni / supersede leggibile in-banda** — A-SK66+KS-SK-91-4 (Klabnik: doc deve citare la G massima; .out superseded fuori dal corpus) ↔ A-BG53 (`reason=`, `supersede_of=` nel ledger) ↔ A-AH57 (Hejlsberg: consumazione --same-rev ledgerata). Unificare: il ledger porta supersede e consumazione; il gate morde sulla citazione.
4. **Per NOME, mai per conteggio** — KG-91-3+A-BG56 (Gregg: doc dice "un marginale", g3 ne marca DUE) rima con A-SK62 (Klabnik: giudicare OGNI token ≥3 cifre su riga [derivata]): il gate deve enumerare, non contare.
5. **Recorder/estrattori fail-closed simmetrici** — A-AH55 (cargo=unknown senza dente) ↔ A-BG54 (estrattore win=0 posizionale, ckpt non nominato): stessa classe, canale non autenticato dal nome.

## CONFLITTI

Nessun conflitto frontale: le tre sedie si rafforzano. Una **tensione di collocazione** su A-SK60: Klabnik preferisce che la cifra derivata la emetta l'EMITTER in un .out ("meglio ancora"), mentre la variante minima (gate che risolve e stampa la provenienza) basterebbe al dente di Hejlsberg. Da sciogliere in sede di attuazione; non tocca i kill-switch. Nota: Gregg CONFERMA la riqualificazione ==2 (due checkpoint by design) — nessuno la contesta.

## PRIORITÀ PROPOSTE per l'ordine S-90.0

1. **A-SK60+A-SK62 (+A-SK61 igiene corpus)** — primo assoluto: il forge A-SK56 è passato DAL VIVO (bound fabbricato PASS con tag [derivata]; 23,5% del corpus è cifre di indirizzi vmmap, amplificazione ~2×10⁴). Finché non atterrano, KS-SK-91-1 rende non-verdict-grade ogni PASS su doc con [derivata]: blocca tutto il resto.
2. **A-SK65** — cache avvelenabile E esfiltratrice (usata per costruire il forge): argv+nonce, env ignorato. KS-SK-91-3 già attivo.
3. **Catena judge_sha: A-BG53+A-AH56** — giudice committato pre-run, g2 annotata dangling nel doc; sana KG-91-1/KS-AH-91-2 prima della prossima campagna.
4. **A-AH54** — prefix-check esteso a battery-attempts.ledger + sha256==DSHA: chiude la riscrittura di storia in-window (KS-AH-91-1).
5. **Sanatorie e manifest**: A-BG56 (doc: DUE marginali per NOME), A-SK63/A-SK64 (manifest prima di WP-100, KS-SK-91-2), A-AH57, A-BG54/A-BG55, A-AH55, A-SK66.

Razionale d'ordine: prima i denti che oggi NON mordono su forge dimostrati (1-2), poi l'integrità di catena retroattiva (3-4), infine sanatorie documentali e scadenze (5).
