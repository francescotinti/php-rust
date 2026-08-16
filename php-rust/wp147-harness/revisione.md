# Revisione S-147 — lente SEMANTICA

## Reperto principale
Il claim scivola dal BERSAGLIO alla FAMIGLIA. Il kill KS-146-1 è valido sul
ponte pre-registrato {LoadSlot, LoadVar} (0,216 s < 0,293 s), ma «ZERO codice
sulla famiglia borrow-first slot» NON è ciò che il verdetto stesso dice: la
famiglia FR1-ext (ponte + ThisPropGet 0,108 + FieldIsset 0,090 = 0,414 s) sta
a 1,41× soglia → fascia «fette micro-judged», e il concilio (sintesi §Ordine
p.2) ordina che FR1-ext PROCEDA sui bersagli già nominati. Letto come chiusura
di famiglia, il claim contraddice il proprio verdetto (riga «Famiglia FR1-ext
ESTESA … 1,41x») e il deliberato. Il kill uccide il ponte, non la famiglia.

## Reperti secondari
1. Deviazione dal criterio p.1-ii: il ponte doveva includere i «fused-load
   visti nel dump»; LoadVarPushConst (0,028 s) è FUORI. Col criterio:
   0,244 s → 0,83× — il kill regge, ma la cifra pubblicata sottostima.
2. Confine «origine slot-load» non nominato: `This` (13,8 M obj, 0,047 s) è
   una lettura di slot di frame a tutti gli effetti. Ponte-criterio + This =
   0,291 s ≈ 0,99× soglia: il perimetro slot completo SFIORA la soglia. Prima
   di dire «slot morto» in S-148, il confine va dichiarato per nome.
3. Attribuzione (census.rs record → s147_note_dispatch) e `fused=false` sotto
   census (run.rs ~4519) gonfiano i siti-load rispetto al pin: direzione
   conservativa per il kill (banda = tetto), quindi accettabile — ma il 1,27 s
   è un tetto del binario census, non del pin: citarlo come tale.

## Vagliate e respinte
- (c) Prezzo per-coppia: un borrow evita clone E drop → risparmio corretto.
- (d) Soglia su sola leg2 pulita: 41,60 × 0,007 = 0,291 s; 0,216 (e 0,244)
  restano sotto — verdetto invariato.
- (e) take_str 7,4 M ⊂ ponte str (25,3 M); convenzioni tenute separate come
  da criterio p.7; 0,029 s ≪ soglia → «a fortiori» regge.
- (f) Il 69,5% è quota memcpy per-movimento (KS-B4), non del gap; tetto
  1,52 s e census 1,27 s coerenti.

## Azioni S-148
1. Riformulare il claim a verbale: kill sul PONTE; FR1-ext resta in fascia
   micro-judged e procede come da concilio p.2.
2. Ricalcolare il ponte secondo criterio (con LoadVarPushConst): 0,244 s.
3. Dichiarare per nome il confine slot-load (This dentro o fuori, con motivo);
   se dentro, registrare che il perimetro completo vale ~0,99× soglia.
4. Nel census ORM futuro, marcare le cifre come «tetto su binario census»
   (fused=false), non del pin.
