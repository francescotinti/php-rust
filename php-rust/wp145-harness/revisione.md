# Revisione S-145 — lente PROCESSO

## Reperto principale
**Le guardie di FR1 esistono come stampa, non come giudice.** Lo script A/B (s145-ab-fr1.sh, r.91–97) stampa i grezzi delle quattro guardie e scrive rc=0 SENZA calcolare banda né verdetto. La m-dimrmw è uscita +0,01s in 3/3 repliche: sopra la banda pre-registrata (drop-1 della serie A, spread zero) e ~+3,3 ns/iter, sopra anche la banda-layout 0,67. L'esito è stato dichiarato «tick di quantizzazione», non rimisurato. In più il criterio p.5 nominava una guardia inesistente (m-dimwrite): pre-registrazione mai collaudata. «Guardie verdi» nel claim B è un giudizio a occhio, su strumento cieco sotto 1 tick, davanti a un segnale coerente 3/3. Non tocca il D=+16,7 sul bersaglio; indebolisce «SPEDITA».

## Reperti secondari
1. **Coppia WP assente al pin s145** — regola utente 2026-08-12: «a OGNI pin nuovo»; il rinvio a S-146 ripete il diradamento già RESPINTO. Debito che declassa il pin finché non saldato.
2. **Emenda gate pair 1%→5% DOPO t1/t2/t3, giudizio rieseguito sui raw t2**: REGOLE §3 chiede la riesecuzione del criterio emendato (run nuova), non il riparse; igiene mutata in corsa a t3 e record scelto post-hoc fra tre. Materialità nulla su KS-B4 (chiavi partizione ≤0,12%; 69,5% lontano da 60%), ma i prezzi pair sono record sotto un gate mai ricollaudato.
3. **Braccio A nato male ×2 + rc-da-pipe**: incidente da CONTARE (REGOLE §2/§5: lettera rc violata), non near-miss — anche se i gate hanno morso prima del giudizio.

## Vagliate e respinte
- Sonda bi-binaria (f.6): regge — «monobinario sui contrasti di tempo» fissato nel modello PRIMA dei dati, conforme alla forma S-143.
- Emende giudice inventario promo (f.5): committate prima del run che le usa, t1/t2 agli atti — emenda dichiarata conforme §5.
- Conferma post-pin per identità di byte (f.8): l'hash uguale trasferisce la misura per costruzione; micro R=5 comunque eseguite al pin.

## Azioni S-146
1. PRIMO atto: coppia WP al pin s145.
2. Rimisurare m-dimrmw A/B pre-leva vs pin a N ≥10× (o timer a ns); regressione confermata ⇒ leva in istruttoria, il guadagno dimread resta (keep-partial-wins).
3. Dare un giudice alle guardie DENTRO lo script A/B: banda calcolata, verdetto nel `.out`, rc dal giudizio.
4. Collaudare il criterio prima della firma: ogni giudice/guardia nominata deve esistere per nome, banda ≥ risoluzione dello strumento.
5. Contare l'incidente rc-da-pipe nel registro; run fresca prezzi sotto gate 5% oppure declassare i pair a indizio non-record.
