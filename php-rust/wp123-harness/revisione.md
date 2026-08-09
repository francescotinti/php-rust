# Revisione S-123 — lente SEMANTICA

## Reperto principale (claim 4)
La colonna «v1(unfused)» NON misura «stesso albero, fusione spenta»: è il census S-119 su un PIN DIVERSO (`s123-classifica-report.sh:48-50` legge `wp119-harness/clite-out`), mentre v2 è head 65b3385. La NOTA stessa dimostra il drift cross-pin (re 17→10 = L-RE1, non fusione; `s123-classifica-verdetto.out:18`) e UNA RIGA DOPO dichiara prop 5→3 «effetto-fusione PURO». Il confronto misura fusione+drift s119→head; il controllo pulito (stesso head, mem-census vs mem-census+zval-census) era disponibile e non è stato eseguito. Predizione p.6 attesa ~0-1, osservato 3: grado mancato ⇒ l'attribuzione è un'inferenza, non una misura.

## Reperti secondari
1. **Attribuzioni fini = inferenze aritmetiche, non conteggi per-sito**: gacensus espone SOLO contatori globali (`s123-classifica-report.sh:24`); «3 residui in `$s += $o->x`» e «arr 4,08 ≈ 2 ZStr di chiave» (`s123-classifica-verdetto.out:18`) vanno etichettate INFERENZA finché non esiste un contatore per-sito.
2. **Le guardie del giudice parlano il regime VECCHIO**: BSTOR (`s123-giudice-v2.sh:26,67`) viene da timer 10 ms, N originali, ordine fisso — a N=2M un quanto = 5 ns/iter (re 10,00 ≈ 2 quanti). Nel re2smoke la guardia str è −7,50 invece di SOGLIA_LAYOUT 2,89 (`s123-re2smoke-verdetto.out:4`). Non flippa verdetti (−2,60 tiene anche vs −4,62), ma il «metro sanato» giudica con soglie del metro malato — contro il p.4 («la SOLA soglia-layout che scende a valle»).
3. **arr scalato cambia il misurando**: `s123-gen-scaled.sh:14` scala solo il loop di lettura (build 100k invariato: quota insert 1/61→1/301; N 6M→30M), mentre scoreboard e classifica usano il micro originale (`s123-classifica-report.sh:19`, arr=6M). Stessa etichetta «arr», due workload: SOGLIA_LAYOUT(arr) vale solo per i file scalati.
4. **Storia causale del claim 1 sotto-determinata**: p.8 legge BANDA_V2_str 2,89<5,00 come «era l'ordine», ma ordine, timer e N sono cambiati INSIEME; la diluizione del rumore a 7×N basta da sola. Le bande v2 restano valide per il nuovo regime; la diagnosi «ordine» no.

Vagliate e respinte: P0b come pavimento (coperto da BANDA su 4 binari distinti); archiviazione L-RE2 da smoke (una refutazione non richiede guardie full).

## Azioni S-124
1. Effetto-fusione PURO controllato: stesso head, due build census (mem-census vs +zval-census), delta prop zvclone; solo allora «attribuito alla fusione».
2. Etichettare in NEXT_SESSION le attribuzioni per-sito come INFERENZA, o aggiungere al patch census un contatore per-sito (prop/arr).
3. Sostituire BSTOR nel giudice con le SOGLIA_LAYOUT v2 come unica fonte; ritirare i valori old-regime.
4. Dichiarare il perimetro delle bande v2 (file scalati + timer getrusage); confronti con micro originali richiedono rimisura, arr in testa.
5. Se serve la diagnosi «ordine»: un run di controllo con nuovo timer/N ma ordine FISSO, che separi ordine da timer/N.
