# Verbale Sedia 1 — Hoare (design linguaggio/runtime Rust, safe-only) — Concilio WP-103

**Oggetto**: S-101 (H-C1a split gc_note; H-C1b move ricevitore) + bozza §S-102.
**Mandato**: refutare.

## VERDETTO: CON EMENDAMENTI

Nessuna refutazione capitale: H-C1b è sound (verificato sul codice: l'handle
arriva owned dal `pop`; i sentieri magic/hook clonano `target` nel frame
PRIMA del `continue`, quindi nessun distruttore può anticipare; l'ultimo drop
non si sposta; fixture 04/12/13 + full per NOME lo sigillano). H-C1a è
semantica-neutra per costruzione e il census-control byte-identico lo prova.
Le promozioni stanno sui loro criteri. Ma il DEBITO DI FORMA che H-C1a
congela è reale e va nominato prima che S-102 tocchi proprio quella regione.

### A-HO-103-1 — «Predicato unico esaustivo per specie contenitore»
La guardia di `gc_note` (mod.rs:3909) è un `matches!` POSITIVO su
Object|Ref|Array|Closure; `gc_note_slow` termina con `_ => {}` (mod.rs:4011);
la STESSA lista di 4 varianti è duplicata una TERZA volta nella scansione
capture delle closure (mod.rs:3999-4002). Zval ha 14 varianti: una variante
nuova (o una promozione a contenitore di una esistente) COMPILA OVUNQUE in
silenzio — `matches!` non gode di exhaustiveness, il wildcard inghiotte.
Risposta alla domanda del mandato: SÌ, l'enumerazione decade in silenzio, e
il tripwire S-100 dei match esaustivi su `Op` qui NON esiste. Rimedio:
`Zval::is_gc_container(&self) -> bool` scritto come match SENZA wildcard
(tutte le 14 varianti nominate), usato da guardia, da `gc_note_slow` in testa
e dalla scansione capture — variante nuova ⇒ errore di compilazione, e i tre
siti non possono più derivare.

### A-HO-103-2 — «Generator è un buco pre-esistente, ora documentato come perimetro»
`Generator` non è né nella guardia né nei bracci del corpo: il `_ => {}` lo
inghiottiva PRIMA di H-C1a (perimetro fedelmente conservato — non imputa la
leva). Ma in Zend un generator È un oggetto e fa root come tale; qui tiene
frame e catture. O si porta evidenza per NOME che i generator sono notati da
un ALTRO canale (birth-track), o si scrive la fixture
generator-tiene-l'ultimo-ref / generator-in-ciclo con ordine __destruct
atteso PRIMA. Il commento «same perimeter» oggi traveste il buco da scelta.

### A-HO-103-3 — «Il rimedio feature-check è monco e non morde da solo»
`cargo check --features zval-census` protegge UNA feature; nel gc_note path
convivono almeno `gc-census` e `zval-census` (+ op-census storica). In
batteria serve la matrice nominata o `--all-features`, e va detto che
check ≠ run: protegge l'esaustività, NON gli assert conteggi↔nomi (quelli
mordono solo eseguendo). Inoltre — lezione A-HE-102-1 — il dente si
dichiara verde solo dopo averlo visto MORDERE (il rosso storico BinaryAdd
è la sua polarità).

### A-HO-103-4 — «S-102 punto 3 collide con il divieto WP-102»
Il candidato «PropGet/PropSet a slot-operando che leggono il ricevitore
dallo slot senza pila» NON può essere un MOVE: lo slot conserva la
proprietà (il NON-riproporre lo dice già: il move non riapre il divieto
perché l'handle era poppato — qui non c'è pop). Le forme possibili sono
DUE e vanno nominate nel criterio: clone-dallo-slot (3 clone → 1, guadagno
contato) oppure borrow, che riapre il VIETATO e esige il sigillo di tipo
A-MA-102-3, non un borrow nudo. Il controfattuale «3 coppie × 2 ns» è
scritto come se il clone sparisse: con clone-dallo-slot ne resta uno.

### Kill-switch
- **KS-HO-103-1**: nessuna leva S-102 che tocca Zval o la pila operandi
  entra in misura PRIMA che `is_gc_container` esaustivo (senza wildcard)
  sia in tree e la batteria verde.
- **KS-HO-103-2**: se la forma del punto 3 attraversa un confine di op con
  un borrow di slot senza sigillo di tipo ⇒ reject senza appello.
- **KS-HO-103-3**: il check di batteria sulle feature si promuove solo
  dopo un morso dimostrato (rosso provocato e rientrato) sulla matrice
  completa delle feature, non su una sola.

### Refutazioni capitali
Nessuna.
