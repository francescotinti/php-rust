# REGOLA «SET CHE SCENDE» (A-KL-104-1, Concilio WP-104 — SCRITTA, vincolante)

Contesto: il gate corpus è per NOME (mai un conteggio). Il caso «un test
del riferimento fail INIZIA A PASSARE» (il set SCENDE) era gestito a
buon senso in S-102 (015.phpt); qui la procedura diventa regola.

## La regola

1. **Un set che scende è comunque uno STOP del gate**, mai un «verde
   migliorato»: il gate confronta per NOME e un nome in meno è un
   mismatch. Vietato dichiarare verde l'esito e proseguire.
2. La discesa diventa MIGLIORIA solo con TUTTI e quattro:
   a. **attribuzione** — il passaggio è spiegato da un cambio NOMINATO
      (commit/fix) e la spiegazione è scritta nel verbale di sessione;
   b. **verifica bimodale sul test nuovo-passante** — byte-id all'oracle
      nei DUE modi, sul PIN che si sta giudicando;
   c. **riferimento aggiornato NELLO STESSO momento documentale** della
      dichiarazione (sessione+rotazione), mai in silenzio;
   d. **artifact di ri-giudizio ARCHIVIATO contro il riferimento NUOVO**
      (KS-KL-104-2: un `.done` rosso contro il riferimento vecchio non è
      MAI citabile verde — serve il SUO artifact, come
      `wp102-harness/corpus-gate/riverdetto-ref1417.txt`).
3. Se anche UNO dei quattro manca, il riferimento NON si tocca e la
   discesa resta una VOCE APERTA per NOME nella rotazione.
4. Un set che scende E sale nello stesso run (un nome sparito + uno
   comparso) non è MAI una migliora: è due mismatch (il conteggio nudo
   non li vede — per questo il gate è per NOME, A-KL-103-2).

Precedente applicato: S-102, `nullsafe_operator/015.phpt` (1418→1417) —
tutti e quattro i punti soddisfatti a posteriori; la refutazione Klabnik
(RC sanata) nasceva dal punto (d) mancante al momento della citazione.
