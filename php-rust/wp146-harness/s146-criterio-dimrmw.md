# S-146 — criterio rimisura m-dimrmw (az.rev. S-145 #2; pre-registrato PRIMA del run)

1. Oggetto: la regressione m-dimrmw osservata in S-145 (+0,01 s = 1 tick,
   3/3 repliche, dichiarata «tick» senza giudizio) va RISOLTA a densità 10×.
2. Giudice: `m-dimrmw10.php` (NUOVO, 30M iter — stessa statement di
   wp138-harness/m-dimrmw.php); ns/iter = (user − pavimento)/3e7;
   pavimenti med3 PER-binario su `wp97-harness/micro/empty.php`;
   risoluzione 0,33 ns/iter.
3. Bracci: A = stash `phpr-s142` bba8a7346d727e0e (pre-leva) · B = stash
   `phpr-s145` a89faf32c62142f9 (leva FR1); hash verificati in testa allo
   script. DICHIARATO: il contrasto copre la finestra s142→s145 (FR1 è
   l'unico cambio sul cammino dim; dente/census fuori dal binario di parità).
4. Protocollo: smoke R=2 ABAB di riscaldamento DICHIARATO MAI giudicato,
   poi R=5 ABAB giudicati; parità stdout oracle==phpr su ogni run.
5. Soglia di CONFERMA = max(1,0 ns/iter [3× risoluzione], rumore drop-1
   max(A,B), banda-layout 0,67) E segni D=B−A>0 in ≥4/5.
6. Esiti PRE-REGISTRATI: Dmed ≥ soglia e segni ⇒ regressione CONFERMATA ⇒
   leva FR1 in ISTRUTTORIA (il guadagno dimread RESTA — keep-partial-wins) ·
   |Dmed| < soglia ⇒ NON confermata, guardia chiusa a verbale ·
   Dmed ≤ −soglia e segni ≤1/5 ⇒ segno opposto stabile, nessuna regressione.
7. Giudice e bande DENTRO lo script (az.rev. S-145 #3): verdetto nel `.out`,
   rc dal giudizio (0=giudicato; 5=parità rotta; 8=quiescenza; 9=pin).
8. Collaudo pre-firma (az.rev. S-145 #4): giudici esistenti per nome
   (m-dimrmw10.php · empty.php · s129-quiescenza.sh); parità oracle del
   giudice NUOVO verificata nella finestra PRIMA del run; soglia (≥1,0) ≥
   risoluzione (0,33) ✓.
9. Finestra: DOPO la coppia (run sequenziali); lock misura PRESENTE (creato
   a inizio sessione, lo script NON lo tocca — veto trap-EXIT-altrui);
   sentinelle busy nel `.out` (regola incidente 15).
10. Grado: GUARDIA — nessuna cifra va in PERF_MAP da questo A/B; l'esito è
    solo conferma/refutazione della regressione.
