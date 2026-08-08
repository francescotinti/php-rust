# Verbale sedia Matsakis — S-116/117, lente ownership/aliasing/borrow

## VERDETTO: CONCORDO CON EMENDAMENTI

A subito è giusto per la ragione giusta (ripara il METRO, non il divario); B come regime regge solo con una clausola anti-tassa; D come selezione va bene con traduzione di ownership obbligatoria; **«C solo se dopo A+B» è l'errore**: l'aritmetica già disponibile dimostra che A+B non bastano.

## ROTTA DALLA MIA LENTE (3 sessioni)

Ordine: **A (S-117) → B+C-incrementale come unico treno (S-118/119) → D come fabbrica di vagoni**. C non è riserva: è la sorgente dei vagoni grossi.

L'aritmetica che refuta «C in riserva»: prop è 7,6× ⇒ oracle ≈14 ns/iter, phpr ≈107; il 3× vale 42 ⇒ servono **−65 ns/iter**. A rende 5-15% (−5..−16), L-A −26..−29: somma −31..−45. Il resto (~25-35 ns) è esattamente il costo invariante 9-10 ns/op del lifecycle Zval (S-103): nascita/morte/refcount. Nessun treno di peephole lo tocca; lo tocca solo C. Ma C ha una scala incrementale SAFE-only, non è big-bang:
- **C1 borrow-non-clone**: eliminare i clone Rc dei ricevitori sui path IC-hit (H-P1 ne valeva ~3 ns su UN sito; il census dei call-site caldi enumera i siti restanti). Prestito `&` al posto del clone = zero costo, il borrow checker lo garantisce.
- **C2 arena per-richiesta per Zval transitori** dietro indici generazionali (slotmap-style, safe): il refcount sparisce dal path caldo per i valori che non escano dal frame; il drop diventa bulk-free a request_end (compatibile col binding output-capture PRIMA del reset).
- **C3 rappresentazione** (NaN-box): **safe SOLO su indici**, mai su puntatori (il roundtrip puntatore↔f64 esige unsafe ⇒ vietato dal sigillo). C3 resta l'ultima carta, previa decisione utente.

**Mossa concreta S-117 (rotta A, ridimensionata alla piattaforma)**: BOLT **non esiste su Mach-O/AArch64** — la raccomandazione lo cita a vuoto. Pipeline reale: PGO (`-Cprofile-generate/use` con profilo da sei micro + held-out + WP media) + LTO fat + `codegen-units=1` + **ld64 `-order_file`** per il layout deterministico. Primo esito da giudicare: **banda leve-nulle RIMISURATA (N=2) sul binario PGO** — il claim di A è che la banda scende; l'uplift micro è il secondo esito.

## EMENDAMENTI

- **R1 (piattaforma)**: sostituire BOLT con PGO+LTO+order_file come sopra. Misura: banda nulla N=2 sul nuovo binario; meter-riparato se max(banda) ≤ 5 ns/iter (oggi 10).
- **R2 (incommensurabilità)**: A cambia il metro ⇒ **tutte le bande e i binari conservati (052ea417, nulla2) DECADONO**; L-A si rigiudica ricompilando la patch 2c18b2e sotto la nuova pipeline. Vietato trascinare soglie pre-PGO.
- **R3 (anti-tassa nel treno B)**: il treno passa solo se il NETTO per OGNI categoria ≥ −banda(cat) del nuovo metro. Tre campioni L-A (−6,5/−7,0/−6,5) oltre le due nulle (−5,5): le tasse sono reali e SI SOMMANO; un treno di 5 vagoni può regredire calls di 5-7 ns mascherandolo nell'aggregato.
- **R4 (D con traduzione di ownership)**: le tecniche Zend vivono di aliasing che il borrow checker vieta (zval* mutati in place, HashTable auto-puntante). Ogni vagone D dichiara PRIMA la strategia: indice / borrow / RefCell; RefCell sul path caldo = refcount travestito, ammesso solo con A/B che ne dimostri il costo.
- **R5 (C1 nel treno)**: S-118 istruisce C1 col census e mette i siti borrow-non-clone come vagoni del primo treno, accanto a L-A.

## KILL-SWITCH

- **A**: se dopo PGO+order_file la banda nulla non scende ≤ 5 e l'uplift mediano globale < 2% ⇒ A chiusa (si tengono i guadagni gratis, si torna al metro attuale).
- **B**: due treni consecutivi bocciati per accumulo tasse ⇒ regime treno sospeso, si va a C2 diretto.
- **C2**: se il prototipo arena su UNA categoria non rende ≥ banda(prop) ⇒ si ferma prima di toccare la rappresentazione; C3 mai senza mandato utente esplicito.

## APPARATO minimo

Solo lo script di build PGO in `scripts/` (ricetta nel pin, REGOLE §2: il pin PGO nasce collaudato-nell'atto) — mezzo pomeriggio, blocca l'oggetto perché senza di esso nessun A/B S-117 è riproducibile.
