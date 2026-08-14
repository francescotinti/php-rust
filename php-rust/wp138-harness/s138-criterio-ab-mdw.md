# s138-criterio-ab-mdw — chiusura eccedenza FD1 sul giudice del MODELLO (PRE-REGISTRATO, prima dei numeri)

1. IPOTESI STRUTTURALE (dal verdetto v2, scarto +17,0 con strumento pulito:
   inline ✓, inerzia 0,000 ✓): l'atteso 34,9 della v1/v2 era aritmetica
   CROSS-GIUDICE — arm 118,2 misurato su m-dimwrite MENO D 83,3 misurato su
   objdatains (giudice DIVERSO: ctor per-iter, mix di statement). La chiusura
   si cerca con un A/B tra binari VERI sul giudice del modello, ZERO probe.
2. Bracci: pin s135 stash `phpr-old-target/release/phpr-s135` (6518a1e14a266d52)
   vs pin s136 canonico (1e14793ec0d9650c); giudice `m-dimwrite.php` (3e6 iter,
   denominatore dal sorgente); ABAB interleaved R=5; user CPU;
   ns/iter = user/3e6. Pavimenti: stesso giudice e stessa CLI sui due bracci —
   differenza di pavimento dichiarata nulla per costruzione, non sottratta.
3. Smoke = prime 2 ripetizioni con early-stop a segno opposto (atteso D>0,
   s136 più veloce).
4. D_mdw = med5(s135) − med5(s136) in ns/iter; rumore = max(4, drop-1
   simmetrico sui due insiemi).
5. IDENTITÀ-PREZZO (gate del blocco): |D_mdw − UB 69,6| ≤ 13,3 → i prezzi del
   modello dim-write CHIUDONO sul LORO giudice ⇒ **blocco dim-write RIMOSSO**;
   l'eccedenza objdatains +13,7 è RIATTRIBUITA a cross-giudice (resta apertura
   per NOME, senza blocco). Fuori banda (in ciascuna direzione, dichiarata) ⇒
   NON CHIUSA, blocco PERSISTE.
6. COERENZA-ARM (informativa, MAI gate — contrasto cross-strumento con banda
   di non-omogeneità ≥2 ns/segmento, az.rev. S-137 #2): arm_v2 51,9 + D_mdw
   vs arm_pre 118,2 (sonda s136-tempo), scarto a verbale.
7. Guardie: parità stdout vs oracle su OGNI run di OGNI gamba; quiescenza gate
   SEPARATO; lock misura ATTIVO; pgrep rust-analyzer PRIMA della finestra;
   hash di ENTRAMBI i binari verificati nel runner (mismatch = abort rc=9).
8. Verdetto in `s138-ab-mdw-verdetto.out`; rc autoritativo da file, MAI da pipe.
