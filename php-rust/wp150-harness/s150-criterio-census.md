# Criterio S-150 p.2 — run di CONTROLLO census con workdir ≥100 char (az.rev.2 S-149) — scritto PRIMA del run

1. Ipotesi da provare (rilievo 2 revisione S-149): lo scarto +3,2% di
   hostcall.n del census s149 (335.837.200) vs s148 (325.416.908) è causato
   dalla LUNGHEZZA del path del workdir (s149: ~28 char; s148: ~100+ char) —
   i path dei file entrano nelle stringhe dei frame (debug_backtrace in testa).
2. Run: 1 replica (dichiarato: le repliche s148/s149 furono 0,000%-identiche)
   della suite ORM col probe census CONSERVATO `phpr-census-s149`
   (hash DEVE essere f3a111ac92cac3ef, lo stesso del verdetto tr4), workdir
   con `${#SP}/orm-work` ≥ 100 char; smoke probe EREDITATO (esito ESATTO).
3. Giudizio pre-registrato su hostcall.n (identità hostcall_n==sum+unnamed,
   unnamed=0, overflow=0 obbligatorie, pena VOID):
   - |hostcall.n − 325.416.908| ≤ 1% ⇒ spiegazione REGGE (lo scarto era il path);
   - hostcall.n ≈ 335.837.200 (±1%) o altrove ⇒ spiegazione CADE, scarto da
     RE-ISTRUIRE (resta aperto a verbale).
4. CONTEGGI, mai tempo; parità fail-set vs baseline16 = sentinella non-gate.
