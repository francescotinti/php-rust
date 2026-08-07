# Verbale sedia 4 — HEJLSBERG (compilatori incrementali, codegen, layout) — Concilio WP-108

Fonti lette: NEXT_SESSION · WP_SESSION_106 · SYNTHESIS WP-107 ·
ha1-criterio/ab-verdetto · pin107-gate-verdetto · micro-pin-s106 ·
reg_lower.rs (doc modulo + finestra ST). Indipendente.

## VERDETTO

**H-A1 PROMOSSA A RAGIONE, ma con TRE debiti di forma**: (1) il doc di
modulo insegna ancora la regola morta; (2) il −128 B di run_loop è
narrato, non contabilizzato per-target; (3) «micro su hash₁» (R-1) è un
proxy arith-only travestito da adempimento pieno. Nessuna refutazione
capitale: il criterio pre-registrato, il co-primario strutturale
(11→9 per NOME, OFF invariato, valore=oracle) e il rifiuto di «nessun
flip» sono corretti. L'attribuzione lecita è AL COMMIT (fusione +
rimescolo codegen annesso), mai al meccanismo per-op — il verdetto lo
rispetta e va tenuto così.

## R-HE-108-n (emendamenti dovuti)

- **R-HE-108-1 — doc di modulo reg_lower.rs DA RISCRIVERE, tre siti**:
  (i) righe 20-21 «`LoadSlot` (silent, cold) is never folded» nella
  lista Fold rules — CONTRADDETTA dalla finestra ST (r.475-491);
  (ii) r.190, doc di `fold_slot`, ripete la regola morta; (iii) la
  lista v3 delle forme monomorfe (r.9-12) OMETTE `BinarySTDst` e la
  frase «no stack-lhs source folds» (r.13) non nomina l'eccezione ST
  (lhs-slot/rhs-stack). L'emendamento S-106-R-3 fu dichiarato al sito
  del fold e nel criterio, ma la regola normativa vive in testa al
  modulo: lasciarla in piedi è ESATTAMENTE la Scoperta 4 di S-106
  («le regole invecchiano») ripetuta nello stesso commit che la refuta.
- **R-HE-108-2 — il −128 B esige il conto per-target**: run_loop cala
  con un braccio in più mentre i tre vecchi bracci (LoadSlot/Swap/
  BinaryDst) RESTANO nel match. Il calo viene quindi da decisioni
  d'inliner altrove (drop_slow −2, panic_in_cleanup −2 lo indiziano).
  La lettera D-4 chiedeva diff per-target COMPLETO: l'istogramma bl +
  taglia totale non lo è. «L'inliner ripaga i due dispatch»
  (Scoperta 3) è narrativa causale non provata: declassarla a
  osservazione; il conto taglie per-simbolo prima/dopo si archivia al
  prossimo admission.
- **R-HE-108-3 — R-1 solo PARZIALMENTE onorato**: il «micro su hash₁»
  è il braccio B dell'A/B (arith-only, protocollo ABAB ≠ run-micro).
  Le altre cinque categorie su hash₁ NON sono mai state misurate: il
  controllo churn-layout su prop/calls/str/arr/re manca. Dichiararlo
  in voci aperte, non contarlo come R-1 pieno.

## A-HE-108-n

- **A-HE-108-1** — Punto banda-layout 4,99 vs 4,97: «registrato non
  cifra» è corretto, MA il contrasto attraversa due protocolli di run
  con rumore tra-run storico 3,5 ns/iter: come punto della banda N≥3
  vale solo se ripreso stesso-protocollo. E un contrasto = UN punto:
  la dizione «due punti colti» in NEXT_SESSION va disambiguata.
- **A-HE-108-2** — Istruttoria arith prossima: IncDecSlot+Pop è fusione
  sana (stessi helper, +1 braccio, text-budget a +196/4096 regge);
  **Sweep per-iter NO come «fusione»**: toccare la cadenza Sweep è
  semantica GC, pretende criterio proprio + gate d'ordine-free, non
  finestra di peephole.
- **A-HE-108-3** — Tetto combinatorio della strategia v3: un'op
  monomorfa per FORMA scala linearmente nei bracci ma lo spazio delle
  forme è combinatorio (il Binary(Sub) residuo tenta già la finestra
  ad albero). La prossima istruttoria pre-registri il BUDGET di forme
  (quante op nuove max) o nomini il pivot (selezione superistruzioni
  da census, non peephole ad libitum).

## KS-HE-108-n

- **KS-HE-108-1** — Un emendamento di regola è COMPLETO solo quando il
  testo normativo (doc di modulo) smette di contraddire il codice nello
  stesso commit; emendamento «al sito» con dottrina vecchia in testa =
  emendamento a metà.
- **KS-HE-108-2** — Una misura sostitutiva (proxy) si dichiara COL
  PERIMETRO che non copre, mai col nome dell'adempimento pieno.

## Ordine S-107

RATIFICO la sequenza 1-5 con emendamenti: R-HE-108-1 entra
nell'igiene (punto 1 o 2, costo minuti); l'istruttoria arith (punto 3)
è ammissibile SOLO con A-HE-108-2/3 recepiti nel criterio; §3.15 in
testa non mi tocca e il golden-stesso-commit è la forma giusta.
