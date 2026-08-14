# s138-criterio-sonda-rmw-v2 — attribuzione RMW, seconda sonda (PRE-REGISTRATO; la v1 è NON CHIUSA a verbale e non si reinterpreta)

1. La v1 cade su DUE difetti dello STRUMENTO, entrambi nominati dai suoi
   stessi numeri: (a) gate nm mal calibrato — `field_rmw_fast` è OUTLINED
   ANCHE nel candidato (due call-site; `field_assign_fast` con un sito resta
   inlined): probe e candidato erano OMOGENEI, il gate assumeva "0 simboli";
   (b) ε (prelude_skip + fill A OGNI ITERAZIONE nel pieno del candidato — il
   fill ri-deriva la resolve) stimato ≤10, coerente coi fatti a ~33: lo
   scarto +32,7 è l'ε stesso, non un buco d'identità. La v2 NON stima ε: lo
   ELIMINA (fill+predicato saltati sotto `PHPR_TP_FULL=1`), così arm_full è
   il pieno a FORMA-PIN dallo stesso binario.
2. Gate nm v2 (fondato): `field_assign_fast` ASSENTE in probe E candidato;
   `field_rmw_fast` nello STESSO stato in probe e candidato (oggi: outlined
   in entrambi) — l'omogeneità è il gate, non un'assunzione di inline.
3. Residuo dichiarato ε′ = ingresso in `field_rmw_fast` fino al kill-switch
   (call + check bool cachato) sul ramo full: nominato, ~pochi ns, tetto 4.
4. Misura su m-dimrmw: identica alla v1 (R=3 fast/full/cal, mediana,
   seg9 sottratto). IDENTITÀ: |(arm_full − arm_fast) − D_AB 173,3| ≤ 13,3 +
   ε′ 4 = 17,3 → eccedenza fuori-modello ATTRIBUITA per NOME (read-walk
   field_value + secondo preludio + write-walk field_set_op + fill assente
   nel fast) ⇒ promozione SBLOCCATA. Fuori → NON CHIUSA, promozione FERMA
   (leva resta a catalogo, misurata e non promossa).
5. Gate invariati dalla v1: parità fixtures-rmw probe==candidato · parità
   stdout vs oracle ogni run · inerzia (probe senza env vs candidato, R=3,
   soglia max(0,012, drop-1)) · quiescenza separata · lock · pgrep
   rust-analyzer pre-finestra. Verdetto `s138-sonda-rmw-v2-verdetto.out`.
