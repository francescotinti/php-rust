# Revisione S-130 — lente PROCESSO

## Reperto principale
**Il verdetto di promozione committato dichiara il pin server s130 «stashato, registrato», ma la riga di PIN_REGISTRY.md (7fb79069, 14:36:41) è tuttora NON committata: il tree è sporco a fine sessione, dopo la rotazione (14:56).** La catena «completa rc=0» pubblica quindi un atto (registro) la cui evidenza vive solo nel working tree — violazione di REGOLE §10 (commit+push a ogni passo) e dello spirito del §2 (registro nello stesso atto). Se il tree si perde, il pin server s130 non ha registro. Non invalida l'A/B di F4, ma incrina «catena di promozione completa» esattamente dove il claim è più forte.

## Reperti secondari
1. **«fixture chain: rc=0 (7/6 gate verdi)»**: denominatore «/6» cablato e stantio dal S-122 (gate preg aggiunto, b3099d4); la frazione impossibile ricorre in s127, s127b e s130 senza che nessuno l'abbia mai riconciliata. Il conteggio è un grep, non un inventario per NOME (contro il principio §5); il session file dichiara «incidenti: 0» tacendo l'anomalia.
2. **Quiescenza senza traccia per-run**: il criterio F4 p.5 la esige «prima di ogni run», ma sopravvive un solo `quiesce.rc` (mtime 14:46:40, coevo alla sola sonda E1a); i verdetti smoke/A/B non citano l'rc di quiescenza. Non provo la violazione — provo che il gate non lascia evidenza.
3. **Criterio E1a p.2 pre-registra una certificazione falsa** («provengono dal solo cammino FieldAssign — i conteggi c1 lo certificano»), smentita dal suo stesso controllo (k=4 su objalloc); l'UB pre-registrato E1a·(k−1)/k (98,8) è sostituito in lettura da 31–35.

## Vagliate e respinte
- **Bande circolari/non derivabili**: ricomputate ESATTE dai 7 rawA committati S-129 (drop-1: objmap 0,02 s→6,7; objalloc 6,7; objchurn 13,3). E non decisive: objmap/objalloc D=−3,3 passavano anche il vecchio default −4.
- **Formula rumore su misura**: col criterio VECCHIO (rumoreB pieno = 20,0) D=+80,0 promuoveva comunque — le emende non portano il peso dell'esito.
- **Ordine commit**: criterio 13:50:32 → codice 13:50:56 → mtime raw 13:55:41+ → verdetti 14:00/14:11 → promozione 14:37. Pulito.
- **Catena non invariata**: diff s129→s130-promozione = soli tag/path. La respinta della riga «67%» è coperta dalla lettura-per-differenza pre-registrata (s129-criterio-tempo p.4) ed è anti-promozionale: lettura onesta.

## Azioni S-131
1. Committare+push la riga PIN_REGISTRY server s130; pin-server.sh verifichi tree pulito post-atto.
2. Fixture chain: lista gate CONGELATA per NOME nello script di promozione; fallire su inventario diverso.
3. Header di ogni verdetto A/B stampi file+valore dell'rc di quiescenza.
4. Emendare la p.2 del criterio E1a; quota ctor/statement via sonda per call-site, con rerun.
5. Dichiarare nel session file la deviazione «7/6» ed emendare «incidenti: 0».
