# Team ENGINE — Concilio WP-93 (fase 2)

Relatore: team-engine. Sedie: 8 Stogov (LSP/A-DS51), 7 Leijen (canale mimalloc iter-3), 1 Hoare (sigilli). I tre verbali individuali restano VINCOLANTI; questo file riconcilia e ordina.

## CONVERGENZE

1. **Lezione unica in tre grafie**: la lettera (di un design, di un gate, di una dicitura) va calibrata sul canale reale. Stogov morde con l'oracle vivo (q4a, hook-set, `& ` con spazio); Leijen morde col simbolo non esportato (`mi_heap_get_default` assente dal mimalloc v3 del tree, memcensus.rs r.634-637/1426-1428); Hoare morde con le grafie che eludono (nome-spazio-paren, turbofish, `ProbeWindow :: arm`). Nessuno dei tre contraddice gli altri: è la stessa legge WP-90 applicata a tre superfici.

2. **Le due refutazioni Leijen NON cambiano l'ordine della fase engine — lo CONFERMANO.** L'ordine roadmap (A-DS51 primo item engine) resta: le refutazioni bloccano il filone canale iter-3 (A-DL-52 va riscritto come A-DL-57, e senza Barrier(W) condiviso a due fasi il piano non è nemmeno ESEGUIBILE — A-DL-58), ma non toccano nulla di A-DS51. Il team registra che cambiano l'ordine INTERNO del filone misura: prima A-DL-57/58 (design emendato), e A-DL-59 (join pendenza-invisibile dai raw ESISTENTI `mi_theap_pages`/`mi_theap_bin`) è anticipabile subito perché è analisi, non campagna nuova.

3. **Il vincolo di sequenza A-TH-57-prefisso REGGE col nuovo A-TH-65, ma solo dopo l'emendamento.** La struttura narm⊆ntot per riga è sana; la dicitura «census TOTALE» è falsa alla lettera (grafia fissa, un solo file). A-TH-65 (ERE `ProbeWindow[[:space:]]*::[[:space:]]*arm` + sweep workspace + rami alias in TH49RE_V8) ri-fonda il vincolo sulla rete larga: il prefisso resta il modello giusto (Hoare lo riusa in A-TH-63 per nome-spazio-paren). Conseguenza d'ordine: **KS-TH-93-1 lega i sigilli al canale iter-3** — nessuna cifra m91 con probe VERDICT-grade finché A-TH-63/65 non mordono. I sigilli v10 sono quindi prerequisito del filone misura, NON di A-DS51.

## CONFLITTI

Nessun conflitto frontale. Due confini da dichiarare (non dissensi):

- **A-DL-59 vs KS-TH-93-1**: il join di Leijen usa raw già consumati; KS-TH-93-1 vieta CIFRE DI CAMPAGNA nuove, non l'analisi dei raw. Delibera del team: A-DL-59 eseguibile subito, ma con grado ADVISORY finché i sigilli A-TH-63/65 non mordono; VERDICT-grade solo dopo.
- **Scoping A-DL31**: Leijen restringe il dente (REFUSE solo per collisione ptr fra thread distinti entrambi post-probe; `heaps_total<W+1` = DECLARED). Non collide con Hoare, ma il dente scopato deve mordere sul proprio harness prima dell'uso (legge WP-88), e la modifica va per NOME nel commit.

## ORDINE PROPOSTO per S-92.0

1. **Sigilli v10 (Hoare, A-TH-62..67)** — PRIMO: corto, tutto pre-misurato riga-per-riga nel verbale, e sblocca KS-TH-93-1 che tiene in ostaggio il filone misura. Ogni dente nuovo morde sul proprio harness con decoy same-commit.
2. **A-DS51 fase 1 (Stogov, emendata A-DS58/59/60)** — CORPO della sessione: è il filone PRONTO (refutazione q4a emendabile pre-codice, nessuna dipendenza dai sigilli né dal canale) ed è il primo item engine di roadmap (gate KS-DS-88-3/89-3/91-1..3).
3. **Canale iter-3 (Leijen)** — ULTIMO, perché BLOCCATO dalle due refutazioni capitali: prima riscrittura A-DL-57 (probe on-thread + `mi_heap_of`, mai `mi_heap_get_default`) e A-DL-58 (Barrier(W) a due fasi, timeout fail-closed), poi campagna. Eccezione anticipabile: A-DL-59 come analisi ADVISORY dai raw esistenti, in coda alla fase 1 se resta budget.

Filone bloccato = canale iter-3. Filone pronto = A-DS51. Sigilli = abilitante trasversale.

## PREREQUISITI per NOME

- **Sigilli v10**: A-TH-62 (punto-nudo `//`-in-coda + riga `/*…*/`), A-TH-63 (rete prefisso nome-spazio-paren/turbofish + sweep), A-TH-64 (num-assert su OGNI cattura awk/grep-c), A-TH-65 (ntot ERE + sweep workspace + alias ProbeWindow), A-TH-66 (doc: UC_STATS lost-write, leak fallback, caso durante-proprio-dtor; nth61 ==1), A-TH-67 (`as` fine riga); decoy same-commit per ogni dente; KS-TH-93-2 armato.
- **A-DS51 fase 1**: A-DS58 (sei fixture v4 per NOME: x1 enum-abstract, x2 hook-set, x3 intersection-return-widen, x4 byref-hook-get, x5 utf8-classname, x6 iface-ctor-grandchild, pin length-prefixed in BYTE), A-DS59 (esenzione ctor keyed sul PROTOTIPO; w7 anti-esenzione per NOME nel commit fase 2), A-DS60 (cinque regole formattazione Q3 in unit dedicate); KS-DS-93-1..3 nel gate.
- **Canale iter-3**: A-DL-57 (riscrittura senza simbolo inesistente, A-DL31 scopato post-probe), A-DL-58 (Barrier a due fasi, cardinalità W asserita in-band), A-DL-60 (A-DL-54 = braccio barriera + gemello serializzato, pad cal-anchored 7.801.102 B, rendezvous FUORI bracket), A-DL-59 (join advisory); KS-DL-93-1..3 armati; A-TH-63/65 mordenti (KS-TH-93-1).
