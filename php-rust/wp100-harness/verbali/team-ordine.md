# Team «ordine» — Concilio WP-100 (relatore)

**Fonti vincolanti**: verbale-6-pedersen.md, verbale-9-gregg.md. Questa nota riconcilia solo l'ORDINE di S-99.

## Convergenze

1. **Nessuno dei due vuole rinviare il collaudo WordPress**: Gregg lo esige come primo deliverable (KS-GR-100-1: nessun rollout né claim CPU prima del punto 1 chiuso per NOME); Pedersen non chiede di spostarlo a un'altra sessione, chiede che non sia VOID per costruzione.
2. **Il rollout (punto 3) è l'ultimo atto in entrambe le letture**: per Pedersen dietro sigillo eager + dente anti-putenv (KS-PE-100-2); per Gregg dietro criteri derivati da misura flag-on, non dal D flag-off (KS-GR-100-2). Le due catene di precondizioni sono compatibili e si sommano.
3. **Le baseline morte vanno rianimate al collaudo, non dopo** (A-GR-100-4 combacia con l'igiene dei pin di A-PE-100-3: tutto ciò che è citato deve essere fresco o etichettato).

## Il conflitto

- **Pedersen (R1)**: il punto 1 USA php-server; il pin 365f4d40 non è collaudato ⇒ per KS-PE-99-1 il collaudo eseguito prima della parità server è VOID dal suo stesso kill-switch. Parità server PRIMA.
- **Gregg (e)**: «alla prossima occasione utile» ha già fatto slittare il collaudo quattro volte; il punto 3 è strutturalmente attraente; la gamba oracle dei rapporti è stantia e va rimisurata al collaudo.

**Risoluzione**: il conflitto è apparente. Pedersen chiede il sigillo eager PRIMA del punto 3 («non al 4»), NON prima del punto 1; e la parità server non è un rinvio del collaudo WP ma la sua precondizione tecnica — si esegue DENTRO il punto 1, stessa sessione. Nessun KS dei due verbali collide con questa lettura.

## Sequenza S-99 proposta

1. **Punto 1a (precondizione dentro il punto 1)**: parità server restapi+option per NOME sotto env -i sul pin 365f4d40; esito nel registro pin come `collaudato: sì` (A-PE-100-1/3; soddisfa KS-PE-99-1 e KS-PE-100-1/3). La ricetta si scripta: servirà di nuovo al passo 4.
2. **Punto 1b**: collaudo WP full+media stessa sessione, immediatamente dopo. Dente anti-slittamento: se 1a fallisce, la sessione ripara la parità — non passa ad altro (KS-GR-100-1 resta armato).
3. **Ri-baseline** delle sei categorie flag-off su ENTRAMBI i motori stessa-finestra (A-GR-100-4); etichetta «nominale, gamba oracle stantia» retroattiva sui rapporti già pubblicati (A-GR-100-1).
4. **Sigillo eager** di `reg_lower::enabled()` nei due main + test anti-putenv + braccio flag-OFF del funnel (A-PE-100-2/4). Il rebuild produce pin nuovi ⇒ ri-parità server per NOME con lo script del passo 1a (nessuna rotazione non collaudata).
5. **Misure flag-on server e criteri di rollout** derivati da misura flag-on (A-GR-100-2), ammessi solo dopo il passo 4 (KS-PE-100-2, KS-GR-100-2).

## KS che restano in vigore

KS-PE-100-1, KS-PE-100-2, KS-PE-100-3, KS-GR-100-1, KS-GR-100-2; KS-PE-99-1 (soddisfatta dal passo 1a); KS-GR-99-2 integrata col criterio di rango (A-GR-100-3).
