# Revisione S-158 (revisore singolo, lente MISURA)

## VERDETTO: REGGE CON RILIEVI — segno e promozione solidi (5/5+5/5, guardie piatte, identità e ordine di pre-registrazione verificati in git), ma la cifra +29,0 non è stabile e il canale alloc spiega meno di metà del delta.

## Rilievi
1. **La cifra +29,0 non regge da sola.** Conferma post-pin su binari a contenuto identico: D=+21,0 con rumore 5,0 — scarto 8,0 > rumore, assorbito da un «drift-tree» mai quantificato. Il valore onesto è l'intervallo [21;29] a segno fermo.
2. **Attribuzione indebolita.** UB-alloc 13,8: in finestra il surplus non-alloc è ~15 ns/iter (>50% del delta); al post-pin scende a +2,2 sopra UB+rumore. Etichetta corretta (§4): direzione+meccanismo firmati, magnitudine non ripartita. La sonda dovuta ha l'orologio del §4: >2 sessioni senza rerun blocca la leva successiva.
3. **Banda smoke senza denti.** [+7;+21] dichiarata, smoke +28,5 fuori, prosecuzione «dichiarando» pre-registrata. Seconda banda consecutiva superata senza conseguenze: così la banda non falsifica nulla.
4. **Giudice 2/6 nomi** (~46% delle alloc di famiglia, method_info a solo cache-hit): claim correttamente scopato al giudice, ma l'estrapolazione a famiglia deve usare il prezzo meccanismo ~7 ns/chiamata, non i 14,5 misurati (surplus incluso). La guardia hostargs piatta copre il match cresciuto, non i 4 nomi non misurati.
5. **ORM: NON RISOLTA costruita su gambe contese.** Entrambi gli estremi di Δ_norm vengono dalle due gambe segnalate (phpr1 ictx=3956 ≈3× la gemella; oracle2=1897 vs 129): la voce [34,90;35,25] merita nota di contesa. ORA_REF=4,94 è coerente (stessa finestra del riferimento; oracle di giornata 4,97/5,02), ma il modello moltiplicativo 1:1 è assunzione non validata a N=2.
6. **Attacchi respinti**: (d) i 91 B/4 cluster sono metadati a lunghezza invariata (UUID; stringa data mai letta nel run; code-sign in LINKEDIT): layout identico, nessun cammino causale sulla misura. (e) a user-CPU l'unico cammino selettivo è contesa cache correlata alla fase ABAB; alternanza AB/BA e raw per-coppia stabili lo escludono in pratica. Nota: REGOLE §5 dice «oggi 1414», il congelato reale è 1412.

## Azioni
1. Sonda del surplus non-alloc su m-refl entro S-160, con rerun su finestra nuova; a registro la leva vale [21;29].
2. In PERF_MAP/roadmap estrapolare la famiglia a ~7 ns/chiamata, etichetta §4 esplicita.
3. Replica ORM con gambe non segnalate prima di chiudere l'attesa; nota di contesa sulla voce registrata.
4. Banda smoke vincolante (fuori banda ⇒ arbitrato prima del R=5) o non chiamarla banda.
5. Emendare REGOLE §5: congelato 1412, flip dichiarati.
