# Revisione S-142 — revisore singolo, lente PROCESSO

**Reperto principale — la «doppia refutazione» del peak contraddice il suo stesso verdetto.**
Il verdetto meccanico (`s142-pair-verdetto-t1.out` r.35) dice: «esito MISTO ->
nessuna firma, si dichiara» — il terzo esito pre-registrato del criterio p.5.
Il session file e NEXT_SESSION lo promuovono a «ENTRAMBE le ipotesi S-140
REFUTATE»: lettura MAI pre-registrata, costruita a posteriori. Peggio: il
disegno non ha controllo per l'inserzione stessa — leg5–6 basse (1744/1753)
arrivano DOPO il rewarmup, compatibili con un effetto RITARDATO dell'inserzione
(mai contemplato); la replica senza inserzione era prevista solo per CPU fuori
banda, non per il peak. Declassamento: «peak bimodale con due ipotesi refutate»
vale «esito misto, nessuna firma; bimodale possibilmente artefatto
dell'inserzione». La coppia in sé (FERMA, COMPATIBILE) non è toccata.

**Reperti secondari.**
- Il promo `.out` non stampa alcuna sentinella pgrep prima del gate micro (il
  census sì, r.8–11): con un rust-analyzer che in catena ha già respawnato una
  volta, i sei rapporti micro dello scoreboard sono senza prova d'igiene
  registrata («misure con LSP in volo» è veto). Il near-miss andava contato o
  la regola di conteggio scritta: oggi il confine è auto-giudicato.
- Emenda census: gate sostituito IN SESSIONE dall'attore che l'aveva scritto,
  dopo due STOP. Dichiarata, più forte in natura (stash immutabile +
  cfg-inspection + ripristino verificato), quota mai gate (p.6) — non invalida;
  ma l'auto-emenda senza contro-firma è un precedente da chiudere.
- «Esito CI PRIMA» (handoff verso S-142) violato e auto-sospeso: candidato mai
  processato dalla CI prima della promozione. CI non è gate di record — ma
  un'istruzione d'ordine caduta senza verbale è debito di processo.

**Vagliate e respinte (con la prova).**
- Gemello A′: veto coperto da conferma pre-registrata PRIMA dei run (criterio
  p.3–4), D=+5,0 > soglia 4,0 stretta, segni 5/5 (verdetto r.8–11). Nota
  dichiarata: lato basso non-gate ⇒ la conferma è di DIREZIONE.
- PIPESTATUS/zsh: run esplorativa non-record; il rc di record viene da file
  (`promo-out/batteria.rc`, promo r.2).
- Cap rotazione: verificati da me con wc -l = 80/40.

**Azioni S-143** (ordinate):
1. Rettifica a verbale: «peak = esito MISTO, nessuna firma» in WP_SESSION_142 e
   NEXT_SESSION; nessuna ipotesi nuova parte da «refutate».
2. Replica peak-only SENZA inserzione prima di ogni sonda sul bimodale.
3. Promo-script: sentinelle pgrep stampate nel `.out` prima del gate micro.
4. Decidere con l'utente se il near-miss vale incidente 15; regola di conteggio
   scritta (una riga, sostituendone una).
5. CI: smaltire la coda o togliere dall'handoff le istruzioni d'ordine non
   vincolanti — mai istruzioni che si auto-sospendono.
