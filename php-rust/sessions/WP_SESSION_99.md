# WP_SESSION_99 — S-99.0: la sessione di sole misure che ha saldato tutti i debiti di collaudo e ha rifondato il criterio del prossimo passo

**In una frase**: abbiamo verificato da cima a fondo che il nostro motore
PHP produce ancora gli stessi identici risultati del PHP ufficiale — sul
sito WordPress completo, sulle sue API e su migliaia di test — abbiamo
scoperto e sostituito un binario del server che non poteva funzionare per
un difetto di costruzione, abbiamo rimisurato quanto siamo più lenti
categoria per categoria, e abbiamo dimostrato con una misura che la
prossima ottimizzazione pianificata va puntata altrove rispetto a dove si
pensava.

**Data**: 2026-08-05 (12:37–14:20). **Modello verificato all'apertura**:
Fable 5. **Ordine eseguito**: Concilio WP-100 per intero (4 punti) + il
condizionale del timebox (sigillo eager + anti-putenv + ri-collaudo
pin). **Commit**: 8d1de3b → … → 4c34f61 → chiusura, tutti su main,
pushati. **Nessuna riga del rollout scritta** (ordine rispettato).

## Ordine eseguito

| # | Esito |
|---|---|
| **1 · Parità server** | 🔵 **pin 365f4d40 REFUTATO**: compilato SENZA `axum-server` (recidiva WP-77.6.5.2.3) — non poteva passare NESSUN collaudo, mai. Pedersen R1 confermato nel modo più forte. Pin ricostruito CON RICETTA → 48a46c690fb85005, **collaudo rc=0 in run unico** sotto env -i: sentinella output-capture PASS (byte-id) + option 413 + restapi 3508 IDENTICI per NOME. `PIN_REGISTRY.md` creato (A-PE-100-3) |
| **2 · Collaudo WP** | **CHIUSO PER NOME**: media 0 failure sui due lati; full conteggi IDENTICI (30472/4558029/86W/73S), unica divergenza = `test_wp_is_stream` ftp GIÀ a catalogo. Coppia peak (`collaudo99.out`): full CPU 1,893× · full peak 2,374× (phpr 1892,56 MiB, PIATTA vs WP-94 −0,45%) · media CPU 2,630× · media peak 2,698×. Il debito regola-n.2 di H-A2+H-B2 è SALDATO; contatore full/media AZZERATO |
| **3 · Ri-baseline sei categorie** | ENTRAMBI i motori, stessa finestra (`micro-rebaseline99.out`): **arith 17,5 · prop 13,8 · calls 8,6 · str 6,9 · arr 4,9 · re 3,8**. Gamba oracle risanata; **H-C (prop) e H-D (calls) RIANIMATE** (entrambe ≫ 5×) |
| **4 · Pre-misura rollout** | Tre gambe (`premisura-rollout99.out`): **(4a)** controfattuale statico BinarySS/SC/Dst: il rimovibile è SOLO carico+match del payload BinOp, ramo costante per sito ⇒ **predizione D_registro = 0, banda [0, 0,5] ns/occ** — un rollout Add nelle forme registro cadrebbe a tavolino; **(4b)** build INT1 (patch solo-misura, target-dir separato, albero ripristinato): **D ri-derivato 6,27 ns/occ = 3,60 call/marshalling (57%) + 2,67 traffico Vec (43%)** — la stima 50-70% di Bak confermata; **(4c)** baseline flag-on stessa finestra: add on 3,53 / off 4,66 · arith on 5,44 / off 7,44 — la gamba flag-on è INVARIATA da S-97.1 (5,43→5,44: emissione davvero bit-identica), il vantaggio flag-on oggi è −26,9% |
| **Timebox · Sigillo** | `seal_reg_lower_mode()` primo atto dei DUE main (A-PE-100-2) + dente anti-putenv a 2 bracci sul funnel vero (set→on rifiutato, unset→off rifiutato). Batteria **1726/0 rc=0**. Rotazione pin COLLAUDATA in sessione: phpr 52330330 (corpus **1418 per NOME IDENTICO off E on**), php-server a838866e (ri-parità rc=0: sentinella+option+restapi). Nessuna rotazione non collaudata è rimasta aperta |

## 🔵 Scoperte

1. **Il pin server dichiarato era incollaudabile per costruzione** — la
   feature `axum-server` è esclusa dai default e la build "effetto
   collaterale" di S-98 non l'aveva; il kill-switch di Pedersen ha fermato
   l'uso di un binario che avrebbe fallito al primo `--axum`.
2. **La composizione "specializzazione × registri" NON esiste dove la si
   cercava**: le forme registro inlineano già `binary_fast` su prestiti —
   niente call, niente marshalling, niente pop/push. Il 57% di D è
   call+marshalling e il 43% è traffico di pila: entrambi assenti lì. Il
   valore del meccanismo H-B2 vive sul percorso PILA (Sub/Mul/cmp restanti)
   e il residuo flag-on (~9,9 ns/op) vive nei corpi non-Binary.
3. **Il perl dei rapporti moriva in silenzio** (`while (<$h>)` dentro
   `map { t($_) }`: $_ alias read-only) — pair99-ratios usciva col solo
   header; stesso codice IDENTICO in pair94.sh che però il suo .out lo
   produsse: voce curiosa ma il fix è nel launcher nuovo.
4. **I fail del phpt-runner sono INDENTATI di due spazi**: la prima
   estrazione prendeva zero righe e il gate urlava DIVERSO su un artefatto
   (conteggi identici, insieme "vuoto"). Il dente ha morso lo strumento,
   non il motore.

## ⭐ Lezioni

- ⭐⭐ **Un pin non costruito con la sua ricetta può essere refutato senza
  eseguire nulla del suo perimetro**: bastava `--axum` per scoprire che
  mancava la feature. Il registro pin con `collaudato:` e il divieto di
  pin-effetto-collaterale sono ora meccanici (PIN_REGISTRY.md).
- ⭐⭐ **La pre-misura ha ucciso a costo zero il criterio sbagliato**: sei
  sedie sospettavano che D=6,07 non fosse ereditabile; la decomposizione
  (una patch da 20 righe + tre binari nella stessa finestra) l'ha PROVATO
  e ha pre-registrato la banda [0, 0,5] per il percorso registro. Terza
  sessione consecutiva in cui il metodo "criterio scritto prima" morde.
- ⭐⭐ **Un gate che urla DIVERSO va prima puntato contro sé stesso**: lista
  estratta VUOTA + conteggi identici = artefatto d'estrazione, non
  regressione (recidiva della classe "forgia che fallisce in silenzio",
  stavolta presa PRIMA di dichiarare il verdetto).
- ⭐ Il collaudo è stato eseguito con la parità server PRIMA (KS-PE-100-1):
  l'ordine invertito dal concilio ha evitato un collaudo VOID by
  construction — e il costo totale è stato ~25 minuti.

## Stato binari e processi

- phpr parità: **52330330873f0132** (HEAD 4c34f61, col sigillo eager;
  stash additivo `phpr-s99-sigillo`; l'hash churna col relink del
  test-funnel: fa fede HEAD). Corpus 1418 per NOME off+on
  (`wp99-harness/evidence/corpus-s99-{off,on}.fails`).
- php-server: **a838866e134b6a20** (`php-server-s99-sigillo`),
  **collaudato: sì** (ri-parità rc=0 14:17). Registro: `PIN_REGISTRY.md`.
- Binari intermedi di sessione: 48a46c69 (`php-server-s99`, collaudato poi
  superseded), INT1 7c8053a1 (solo-misura, MAI spedito, patch in
  `wp99-harness/int1-decomposition.patch`; target-dir
  `/Volumes/Extreme Pro/Claude/phpr-int1-target` cancellabile).
- Nessun processo orfano; uploads ripristinata e verificata (count=3) due
  volte (parity-run e pair99); MySQL wp8 su.
