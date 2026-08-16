# Criterio S-149 p.2 — sonda-PREZZO pair COLLAUDATA (churn 16–48 B) + regola di decisione leva — commit PRIMA del run

1. Oggetto: prezzo PROPRIO (ns/coppia) del churn della testa hostcall —
   coppia malloc+free a taglia esatta `Vec::<Zval>::with_capacity(1/2/3)`
   (pair16/pair32/pair48, shape s148: ≤16 B 98,8M · ≤48 B 107,9M) + pattern
   `pop_keys` ESATTO (`splitoff3`: push×3 + `split_off(1)` + drop). Probe
   monobinario `--features sonda-price` (nessuna census attiva), segmenti
   NUOVI dentro `__phpr_sonda_b` accanto ai s145 (zcell/arr0 restano INDIZIO
   di confronto, MAI cifra).
2. Giudice proprio: chiavi `s149.price.*_ns` dal file `PHPR_SONDA_OUT`;
   N_PAIR=20M per segmento; R=2 repliche + smoke; banda per chiave =
   [min,max] delle 2 repliche; replica >2% sulla chiave usata in decisione ⇒
   terza replica dovuta; prezzo citato SEMPRE come intervallo, mai punto.
3. Collaudo-nell'atto (smoke, esito ESATTO): stdout `SONDA-OK` E tutte le
   chiavi s145 (11) E s149 (6: 4 prezzi + zval_size + n_pair) presenti, >0
   dove prezzo; `zval_size` stampato ad audit della taglia (attesa 16 B —
   diversa ⇒ shape da RIDICHIARARE, non gate).
4. Igiene: misura di TEMPO ⇒ lock `/private/tmp/phpr-measure.lock` di
   finestra + quiescenza in retry (mutex CI) + sentinelle stampate; script
   copia di s145-sonda-prezzi.sh (manifest s149-sonda-copia.diff).
5. DECISIONE leva (scala S-146 EREDITATA, sintesi p.«Soglia arbitrata»):
   attesa_alta = conteggio del bersaglio (tetto su binario census, tranche-4/
   anatomia) × prezzo_max del segmento PERTINENTE; risoluzione giudice ORM
   0,26–0,30 s ⇒ attesa_alta < 1× ⇒ ZERO codice sul bersaglio-solo; 1×–2× ⇒
   SOLO fetta micro-judged (soglie REGOLE §3); ≥2× (≥~0,6 s) ⇒ scommessa
   suite ammessa. Il costo SOSTITUTIVO della leva va nominato nel criterio
   della leva stessa (mai alloc-removal a costo zero).
6. Esiti pre-registrati: smoke fallito ⇒ STOP rc=8, nessun run di record;
   attesa sotto scala su TUTTI i candidati ⇒ la leva d'obbligo (regola
   ritmo) si sceglie sul ranking tranche-4 (fetta micro-judged sulla testa),
   dichiarandolo.
