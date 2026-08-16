# Criterio S-149 p.1 — tranche-4: census per-NOME-builtin dentro `hostcall` — commit PRIMA del run

1. Oggetto: partizione di `hostcall.n` per NOME di builtin ai DUE siti s148
   (`Op::CallBuiltin` + `Op::CallHostBuiltin`, run.rs): scope-nome RAII negli
   stessi punti del tag ⇒ perimetro IDENTICO a s148; conteggio per-nome SOLO
   con tag corrente == `hostcall` (un grow/frame/gc interno esce dal nome come
   esce dal tag: convenzione s148 EREDITATA).
2. Nativi host.rs/host_reflect: ESCLUSIONE RIDICHIARATA (rett. rev. s148 p.2)
   — restano in `none`; questa tranche nomina dentro hostcall.other 165,6M;
   none.other 94,6M resta secondo candidato CON i nativi dentro.
3. Numeri per nome: n · attr (crosswalk s144 EREDITATO) · other = n−attr · b;
   tabella statica pre-allocata (4096 slot open-addressing, ZERO alloc del
   censimento; chiave = FNV-64 del nome, eguaglianza per hash DICHIARATA;
   overflow e unnamed contati e stampati, attesi 0).
4. Identità: Σ_nomi n + unnamed == hostcall.n della STESSA run (ESATTA,
   stesso hook); r1==r2 ≤1% su n/attr/b per nome delle teste; parità per NOME
   vs baseline16 (pena cifra NULLA); righe s148tag ristampate a corredo
   (attesi hostcall.n ~325,4M e other ~165,6M entro ~1% — scarti dichiarati).
5. Binario census `--features mem-census` (probe, MAI parità, hash a
   verbale); monobinario ×2 repliche; sentinelle stampate non-gate (S-143
   p.1); smoke a esito ESATTO: ≥2 nomi vivi con n≥1 (`str_repeat`, `sprintf`
   — builtin che allocano di certo) E identità p.4 sullo smoke E unnamed==0 E
   overflow==0 (+ smoke s148 EREDITATO: frame/hostcall/arrgrow + identità).
6. Parser `s149-parse.py` committato + golden PRIMA del run; cifre citabili
   SOLO da `s149-tr4-verdetto.out`.
7. Soglia-conteggio INDIZIO EREDITATA s148 p.7 = 24,8M eventi (0,293 s /
   11,8 ns, prezzo pair ALTO — INDIZIO, mai budget): un NOME con other ≥
   24,8M = testa CANDIDATA della sonda-prezzo; sotto soglia solo per FAMIGLIA
   nominata (Σ famiglia ≥ soglia), mai bersaglio-solo; nessuna conversione in
   secondi come cifra di record.
8. Esiti pre-registrati: probe MUTO/identità violata allo smoke ⇒ STOP rc=8,
   niente run; identità p.4 violata sul run ⇒ verdetto NON valido (solo
   osservativo); unnamed+overflow > 1% di hostcall.n ⇒ ranking DECLASSATO a
   indizio; r1≠r2 >1% su una testa ⇒ dichiara e replica.
9. NESSUNA leva si scrive su questo verdetto (decisione leva solo dopo la
   sonda-prezzo, ordine S-149 p.2); cifre sempre con qualifica «tetto su
   binario census».
