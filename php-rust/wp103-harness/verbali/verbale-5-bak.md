# Verbale sedia 5 — Bak (VM V8/HotSpot: alloc-rate, path caldi, dispatch) — Concilio WP-103

**Oggetto**: S-101 + bozza §S-102. **Mandato**: refutare.

## VERDETTO

S-101 REGGE nei conteggi (census-grade) e nelle due promozioni (criteri
pre-registrati, A/B interleaved, controllo positivo `recv_clone_prop`
90M→0). NON regge la cifra «26,6% = meccanica della pila operandi» come
attribuzione stabilita: è profile-grade su simboli inlined, lo stesso
strumento che ha già sovracontato ~2× su H-C1b. §S-102.2 e §S-102.3 sono
ammessi SOLO con gli emendamenti sotto. Ordine S-102 nel complesso: sano.

## 1. La lettura «26,6%» — direzione sì, cifra no

`as_slice/len/pop/push` attribuiti come innermost via atos -i sono
istruzioni DENTRO i corpi handler: il campione che cade su un `Vec::len`
inlined può essere skid del PC, bounds-check che il compilatore ha fuso
col dispatch, o stallo di cache imputato all'istruzione sbagliata. Il
confine 21,2% dispatch ↔ 26,6% pila è quindi POROSO nei due sensi. Il
controfattuale CONTATO che distingue: il census push/pop di S-102.2, ma
per SITO-OPCODE (non solo per categoria) e per PRIMITIVA (push / pop /
len / as_slice / expect separati) — perché una forma slot-diretta elimina
alcune primitive e non altre. Il costo/accesso fa fede SOLO dall'A/B di
una leva che rimuove K transiti contati: Δ_misurato/K. In più serve la
leva-nulla di controllo: stesso numero di op, traffico pila invariato ⇒
Δ atteso ≈ 0; se non è ≈0, il 26,6% contiene dispatch travestito.

## 2. Drop-glue su scalari: predizione confermata, la leva ne segue

`drop_in_place<Zval>` 12,9% con SOLI Long in circolo conferma la mia
predizione WP-102: il glue paga discriminante+call anche quando non c'è
nulla da droppare. Census: ~11 drop/iter × ~2,0 ns (contabile, quindi
banda LARGA per la sovrastima registrata). **Leva NOMINATA: H-C2 —
fast-out scalare del drop**, gemella di H-C1a: guardia `#[inline]` sul
discriminante ai siti caldi di morte (pop/sweep/overwrite), `forget`
implicito per le varianti banali, glue vero solo per contenitori.
Atteso dal canale contato: −11 call/iter, banda **[8, 22] ns/iter**
(pavimento 8 = metà del contabile, per la lezione 2×) ⇒ prop 11,5 →
[10,3, 11,0]. Sotto pavimento = si registra, non si spedisce.

## 3. Slot-diretti per i Prop-op: il precedente Add pesa

Il rollout Add nelle forme registro è caduto a tavolino ([0, 0,5]): la
tesi «togliere il round-trip di pila paga» è GIÀ stata refutata una volta
su arith. Perché prop dovrebbe differire? Solo se il census mostra che i
transiti/iter dei Prop-op superano quelli che Add avrebbe tolto, E ogni
variante nuova di opcode aggiunge corpi al run_loop (branch mai-preso =
+2,9%, WP-38; la tariffa non-costante di WP-98 taglia in ENTRAMBE le
direzioni). Dump-diff come primo giudice (già in bozza: bene) — ma non
basta: serve il pavimento pre-registrato e il collaudo delle categorie
NON toccate.

## 4. La «tariffa» a tre punti

Arith 9,9 · prop 9,67 → 8,94: tre punti in [9, 10] NON fanno una legge —
fanno l'impronta del pavimento condiviso (2-3 accessi Vec + dispatch per
op) su due giudici che condividono la stessa meccanica. WP-98 l'ha già
detto: non è una tariffa costante. Uso ammesso: sanity-check a posteriori.
Uso VIETATO: predire il gettito di una leva (i pavimenti vengono dai
canali contati + A/B, mai da ns/op × op tolte).

## Emendamenti

- **A-BA-103-1**: il census pila (S-102.2) conta per SITO-OPCODE e per
  PRIMITIVA (push/pop/len/as_slice/expect distinti), con assert
  conteggi↔dump statico (KS-KL-101-3 esteso alla pila).
- **A-BA-103-2**: ogni atteso derivato dalla quota 26,6% si dichiara a
  banda con pavimento = METÀ del contabile (sovrastima 2× registrata
  su H-C1b); atteso puntuale da quota% = vietato.
- **A-BA-103-3**: iscrivere **H-C2 (drop fast-out scalare)** col canale
  contato ~11 drop/iter, banda [8, 22] ns/iter, misurata DA SOLA prima
  del cumulo.
- **A-BA-103-4**: prima di ogni forma slot-diretta, leva-nulla di
  controllo (op invariate, pila invariata) per tarare il rumore
  dell'attribuzione dispatch↔pila.

## Kill-switch

- **KS-BA-103-1**: nessuna forma Prop-op slot-diretta si spedisce senza
  dump-diff PRIMA + pavimento pre-registrato; Δ < pavimento all'A/B
  interleaved ⇒ NON si spedisce (precedente Add [0, 0,5]).
- **KS-BA-103-2**: ogni variante nuova di opcode passa il collaudo delle
  categorie NON toccate (arith/str/arr in banda sul binario cumulativo);
  regressione fuori banda ⇒ revert della forma.
- **KS-BA-103-3**: qualunque atteso costruito dalla tariffa ns/op invece
  che dal canale contato ⇒ misura VOID.

## Refutazioni capitali

**Nessuna.** La cifra 26,6% è marcata grade=PROFILE nei suoi file e la
bozza S-102 già subordina la leva al census contato: il metodo ha retto;
gli emendamenti stringono i bulloni, non riparano una falla.
