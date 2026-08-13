# Revisione S-133 — lente PROCESSO (revisore singolo)

## Reperto principale
Il claim tecnico regge (A/B, gate e hash verificati: ri-hashati i quattro stash — phpr-s132=6af6e497, phpr-s133=c87439a9, server s132=ad17a10d, s133=d447f828, tutti conformi). Ma la riparazione dell'incidente si appoggia a un registro MARCIO: in PIN_REGISTRY.md la tabella «phpr» non registra pin phpr da S-104 — da s109 pin-server.sh appende le righe SERVER in coda al file, cioè dentro la tabella phpr (colonne disallineate incluse). Il pin phpr s133 c87439a9 esiste a registro solo come inciso dentro la riga server; l'emendamento dell'incidente ha toccato UNA riga server dichiarando il registro sanato. Inoltre «PROMOZIONE COMPLETA rc=0» è stata committata (6621b46) con phpr-s132 ancora «in ricostruzione»: la ricostruzione e la rinomina sono avvenute A MANO, fuori da scripts/pin-*.sh (regola: pin/stash SOLO via quegli script), e la verifica al byte vive come prosa in WP_SESSION_133.md, senza artefatto rc. Riparato nel merito, tamponato nel metodo.

## Reperti secondari
1. s133-promozione.sh committato «corretto per il record» porta ancora commenti stantii (pin s124, pin-phpr.sh s129): stessa malattia del sed cieco, sopravvissuta alla cura.
2. objalloc D=+46,7 > UB modello 35,4: ~11 ns del guadagno promosso senza meccanismo nominato; nessun disasm prima/dopo (lezione WP-104) su un cammino caldo.
3. Sonda: 2 predizioni su 4 sbagliate (entrate 2 vs 4; datains TOT 6 vs 9) — dichiarate, ma la clausola di ridisegno copriva solo lo split 2+2: il k=9 di S-130 resta smentito e non ri-derivato.
4. Gate teardown: 7 vettori pinnano la parità ALLA RISOLUZIONE 7 — il buco «by construction» è ristretto, non chiuso; ogni claim di assenza oltre i 7 vettori eccederebbe.
5. Riferimento full ON-ONLY = UN punto (N=1, dichiarato): il «DENTRO il riferimento s131» confronta un punto senza banda.

## Vagliate e respinte
- Ordine commit: criterio dbff54a PRECEDE sonda/leva (59c18c7, e08772f) ✓.
- Soglia 6,7 e bande 13,3/10,0: ricomputate dalle gambe B raw di s132-ab-lo1 ✓.
- Esclusione leg2-on: gate 1,5× per motore (486/215=2,26), firma presente, criterio riusato ✓.
- Rinomina = stash non collaudato: lo smoke viaggia coi byte, hash riconfermati ✓.
- Verdetto promo con etichette s132: record grezzo fedele allo script committato pre-run ✓.
- CI sospesa/push differiti: dichiarati, commit per-step p.1–p.11 presenti ✓.

## Azioni S-134
1. Sanare PIN_REGISTRY: righe server fuori dalla tabella phpr, riga phpr per s132/s133 con evidenza; correggere pin-server.sh perché appenda nella sua sezione.
2. Gate con rc per gli stash: script che ri-hasha i quattro binari pinnati contro il registro, accodato alla catena delle fixture.
3. Dente di collaudo delle copie dichiarate: diff col template che elenca per NOME le righe divergenti attese (quotate e commenti inclusi).
4. Nominare l'eccedenza D>UB su objalloc: disasm/conteggio chiamate prima/dopo su prop_set_entry.
5. Dichiarare a catalogo la risoluzione del gate teardown (parità provata sui 7 vettori, non oltre).
