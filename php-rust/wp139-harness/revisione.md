# Revisione S-139 — revisore singolo, lente PROCESSO

## Reperto principale (indebolisce il claim 1)
La banda_ON=0,033 è pre-registrata (p.4, commit cac331c 21:51, prima del run), ma le 5 gambe sono dello STESSO tentativo t1, stessa sera, stessa finestra: è variabilità run-to-run INTRA-finestra. Il criterio non ne dichiara la portata, e «da S-140 sostituisce 0,041» la promuove a comparatore TRA finestre/pin — categoria più larga giudicata con banda più stretta: rischio strutturale di falsi delta. Da QUALIFICARE, non ritirare: il rif S-137 [1,767–1,781] cade dentro [1,752–1,785], quindi il primo uso trasversale regge.

## Reperti secondari
1. **Push mid-finestra**: commit CI 6f2280c (22:24) e 045528e (22:35) spediti mentre le gambe della coppia giravano; la CI locale fa build+batteria per-push in background. «Niente build mentre le gambe girano» è rispettato solo se il runner CI onora phpr-measure.lock — non dichiarato in alcun file letto. Le firme ictx ≤110% delle gambe 1–5 dicono che non ha morso: fortuna, non processo.
2. **Emenda byte-id (pista a)**: legittima ex REGOLE §5 — dichiarata nel verdetto, filtro a coppia esatta, divergenza già a catalogo per NOME da S-138 — ma la falsificazione del p.2 era PREVEDIBILE: cmp rc=0 preteso su un file contenente vettori con divergenza vs oracle già catalogata. Contraddizione interna al criterio risolta solo dopo il diff.
3. **Gate ictx rimisura senza conseguenza pre-registrata (pista c)**: il criterio p.3 nomina la soglia ma non cosa fare di una gamba SEGNALATA; l'adiudicazione «coerenza stesso-lato» è avvenuta fuori verbale. Il REPERTO ORM sopravvive (escluso phpr1 resta 8,71 ≥ 8,56); il «dbal ↓» invece poggia su DUE gambe oracle entrambe segnalate: da declassare a indicativo. La quiete post-coppia come solo chain+sleep è come registrata: debole ma non violata.

## Vagliate e respinte (con la prova)
- **(d)** Soglie hit≥6/fill≥1 pre-registrate PRIMA dei contatori: criterio 1e31ac6 21:57:33, feature ic-stats f7581e7 00:53:58 (timestamps git). Il «21==7×3» è ricostruzione post-hoc, ma il criterio stesso rimandava i conteggi esatti a verbale: lecito, purché non spacciato per predizione.
- **(e)** Lock tenuto sui build post-finestra e rimosso a fine sessione: esattamente ciò che p.3 registra. Coerente.
- **Ordine criteri→run**: tre criteri 21:51–21:57, verdetti 00:32–00:53. Coerente.

## Azioni S-140
1. Etichettare banda_ON=0,033 «intra-finestra» in PERF_MAP; confermarla su una seconda finestra al prossimo pin prima che dichiari un delta da sola.
2. Regola scritta: niente push durante la finestra di misura, oppure runner CI che verifica phpr-measure.lock, dichiarato nel criterio.
3. Pre-registrare nel criterio rimisura la conseguenza di una gamba segnalata (esclusione/annotazione, N minimo).
4. Dry-run dei gate byte-id contro il catalogo divergenze PRIMA di committare il criterio.
5. Declassare «dbal ↓» a indicativo nel session file e in PERF_MAP.
