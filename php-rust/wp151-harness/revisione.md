# Revisione S-151 — lente PROCESSO (revisore singolo adversariale)

## VERDETTO: REGGE CON RETTIFICA

Verificato e solido: ordine criterio→run rispettato con timestamp (criterio ae67d02 22:10; emenda §5-bis committata 23:16:35, PRIMA di run-r1 23:17:58); dente collaudato sul MECCANISMO VERO (file sintetico 2001 righe nel tree, messaggio esatto, rc=101) nell'atto di armamento; nessun .rs tracciato modificato oltre loc_dente.rs (guardia PREP + git status); cap 80/40 rispettati; incidente 19 contato; commit/push a ogni passo; plenaria con dissensi a registro e 2 quesiti utente dichiarati.

Tre rettifiche dovute:

**R1 — «×2 repliche IDENTICHE» è un sovra-claim.** Identiche solo su C1–C5 (s151tot). La testa hostcall differisce: r1=82.211.375 vs r2=82.211.688 (Δ313), con run sotto CI BUSY dichiarato. Un contatore che varia tra repliche di un run di CONTEGGIO non ha oggi un meccanismo nominato: finché non ce l'ha (o un rerun quiet non lo azzera), la testa hostcall non è citabile come cifra esatta nei criteri S-152. I conteggi C1–C5 restano VALIDI (identità §3, conservazione, r1==r2).

**R2 — la lettura post-hoc ALTERA la parte operativa del §5-bis.** Il ramo che ha morso è «altrove ⇒ aperto, testa nuova»; l'istruttoria a diff sorgente era la conseguenza del ramo ≤1%-da-335,8M, che NON ha morso. La lettura la prescrive comunque («come già previsto dal ramo ≈335,8M» — falso alla lettera) tramite la decomposizione residuo non-backtrace +0,16%, NON pre-registrata; il session file la eleva a «staleness provata». Analisi plausibile, ma va etichettata POST-HOC/indiziaria, non «provata», e il reindirizzo a diff-sorgente va dichiarato come decisione nuova, non derivazione.

**R3 — probe auto-certificato.** Un solo subagente ha scritto strumentazione, attesi, checker e verdetto prep; la guardia pre-run riusa il SUO checker. Mitigazioni reali (enumerazione chiusa dal compilatore, conservazione con walker strong_count indipendente), ma nessun atto di verifica indipendente degli attesi è a verbale.

## Azioni
1. Correggere session file/NEXT: «repliche identiche sui canali C1–C5; testa hostcall Δ313 sotto contesa, meccanismo da nominare».
2. Declassare «staleness provata» a «indiziata (decomposizione post-hoc)»; dichiarare il diff-sorgente S-152 come decisione nuova motivata, non ramo §5-bis.
3. Prima di citare la testa hostcall in un criterio S-152: rerun quiet (CI idle) o meccanismo del Δ313 nominato.
4. Da ora: attesi di smoke verificati da un secondo attore (conduttore o revisore) PRIMA del run di record, a verbale.
5. Registrare il probe conservato anche con copia fuori dal working tree (untracked = un git clean lo cancella).
