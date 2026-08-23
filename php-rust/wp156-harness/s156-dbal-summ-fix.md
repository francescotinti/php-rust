# S-156 p.3 — fix estrazione summary dbal (az.rev. S-155 #4): DIAGNOSI CHIUSA

**Meccanismo (verificato sui .txt di s155)**: `summ(){ tr -d '\0' < f | grep … }`
muore a metà stream con `tr: Illegal byte sequence` — l'output phpr di dbal
contiene byte NON-UTF-8 (fail #10 GenericNameParserTest, nomi accentati:
divergenza latin1 già a catalogo) e il `tr` BSD in locale UTF-8 si rifiuta;
la riga `Tests:` sta DOPO il byte illegale ⇒ summary vuota. L'oracle emette
UTF-8 valido ⇒ la sua summary sopravvive. Le failnames sopravvivono perché
precedono il byte illegale (fortuna del corpus, non correttezza).

**Fix per la PROSSIMA copia coppia (da applicare nella copia, dichiarato nel
manifest)**: `summ(){ LC_ALL=C tr -d '\0' < "$1" | grep -aE "^(Tests:|OK)" | tail -1; }`
e stessa protezione `LC_ALL=C … grep -a` sull'estrazione failnames (oggi
corretta solo per posizione del byte illegale).

**Verifica**: `LC_ALL=C tr -d '\0' < orm-out/dbal-phpr1.txt | grep -aE …` →
`Tests: 3921, Assertions: 5550, Errors: 10, Skipped: 626, Incomplete: 13.`
(leg1 s155, prima irrecuperabile). NB: 3921 vs oracle 3929 e Skipped 626 vs
594 = reperto NUOVO reso visibile dal fix — da dichiarare alla prossima
coppia (il companion dbal non arbitra, ma la differenza di conteggio test
va a verbale con la copia emendata).
