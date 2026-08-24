# Revisione S-157 (revisore singolo, lente PROCESSO)

## VERDETTO: REGGE CON RILIEVI — la sostanza della leva (D=+22,0, riconciliazione UB, conferma post-pin 5/5, catena rc=0) sopravvive, ma la clausola «guardie ok» del claim tace due rc=5 e tre morsi smoke mai dichiarati.

## Rilievi
1. **Morsi smoke occultati + STOP bypassato (grave).** `s157-al1smoke-verdetto.out` chiude «ESITO: GUARDIA MORDE» con TRE regressioni (objdatains −5,0<−4,0; objallocni −11,7<−10,0; re −5,8<−5,0) e `ab-out/al1smoke.rc`=5; gli attesi blind (s157-smoke-atteso-al1.md p.4 «nessun morso atteso», p.6 «rc 2/5/9 = STOP») pre-registravano lo stop. Si è proseguiti al R=5 senza emenda dichiarata e il verbale (WP_SESSION_157.md, Esito 3) cita solo il morso objchurn del R=5. I tre morsi rientrano al R=5, ma il record mostra uno STOP pre-registrato scavalcato in silenzio. Andava contato (incidenti fermi a 19).
2. **Arbitrato objchurn fuori copia-gate.** Anche `ab-out/al1.rc`=5; la promozione poggia su `s157-objchurn-arbitrato.sh`+`m-objchurn12.php`, che NON compaiono in `s157-copia-gate.out` (solo pair/orm/lanci/ab). REGOLE §3 (az.rev. S-154): «arbitrato/derivato di guardia = COPIA DICHIARATA (copia-gate) verificata PRIMA del run». Il commento «estratto dichiarato» nello script non è un copia-gate. La ri-risoluzione a N=12M in sé è conforme a §3.
3. **Header dei verdetti mentono sul braccio A.** `s157-al1{smoke,}-verdetto.out` stampano «A=42efea3e» ma il gate reale è c19079d3 (`s157-ab-al1.sh:46`); inoltre l'arbitrato a contenuto ha accettato 92 B/4 cluster (incluso banner mimalloc __DATE__) contro i «48 B LC_UUID+firma» del criterio p.6 — divergenza già nota dal rilievo S-156, quindi l'atteso byte-id era pre-registrato già sapendolo improbabile.
4. **ORM: normalizzazione post-hoc e LSP non nel .out di record.** Il criterio orm p.5 giudicava in secondi assoluti; la refutazione usa rapporto/Δ oracle-normalizzato, comparatore nato DOPO il segnale (l'indagine stessa lo ammette: «EMENDA PROPOSTA S-158»). L'ordine p.5 (indagine prima della leva) è rispettato (commit 0345dac 02:22 < 2e209d8 02:54). Ma Antigravity vivo «per l'intera finestra» è dichiarato solo in `s157-indagine-hd2.out`, non nel `.out` della coppia di record (precedente incidente 15, REGOLE §3).

## Azioni
1. Dichiarare a verbale i 3 morsi smoke e il rc=5 scavalcato; sottoporre all'utente il conteggio come incidente 20.
2. Copia-gate obbligatorio per OGNI script di arbitrato/derivato prima del run; manifest retroattivo per objchurn12.
3. Header dei verdetti: identità bracci stampata dall'hash misurato, mai da stringa fissa; regione banner mimalloc pre-registrata nel prossimo criterio gemello.
4. Recepire nel criterio ORM S-158 il giudizio oracle-normalizzato PRIMA della coppia; sentinella language-server registrata nel .out di ogni misura di record.
5. Regolare nel criterio il ramo «morso allo smoke»: arbitrato dedicato PRIMA del R=5, o stop.
