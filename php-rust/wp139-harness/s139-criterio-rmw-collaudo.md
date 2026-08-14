# Criterio S-139 p.3 — az.rev. S-138: collaudo CALDO della leva FD1-ext RMW (pin s138) — commit PRIMA dei run

1. Oggetto: chiusura del buco di collaudo (revisione S-138: le 21 fixture non
   eseguivano il fast path nemmeno una volta). NON è una leva: la semantica non
   è in discussione (attacchi S-138 byte-id); si collauda il CALDO a criterio.
2. Fedeltà (az.rev. #1 + #4 + #5): `wp138-harness/fixtures-rmw.php` EMENDATO
   (sezione CALDA H1–H8 con loop ≥4 + v.21 con coda mono-classe post-alternanza)
   + `attacco-rmw-hot.php` + `attacco-rmw-keys.php` INTEGRATI nell'harness
   (recuperati dallo scratchpad S-138): **BYTE-ID pin s138 vs oracle** (`cmp`
   rc=0 per ciascuno dei 3 file). Divergenza NUOVA ⇒ incidente + istruttoria;
   le divergenze del PIENO già catalogate restano per NOME.
3. Gate hit (az.rev. #2): build DIAGNOSTICA SEPARATA (feature cargo `ic-stats`,
   MAI nel pin — veto strumentazione-nei-sorgenti-del-pin) con contatori
   fill/hit per FieldAssignOp e FieldIncDec emessi a shutdown sotto env
   `PHPR_IC_STATS=1`. Gate PRE-REGISTRATO sul run di fixtures-rmw.php emendato:
   per CIASCUNA famiglia `fill ≥ 1` E `hit ≥ 6` (soglia minima prudenziale;
   i conteggi esatti a verbale). La build diagnostica NON produce cifre di
   tempo (solo conteggi); la fedeltà del p.2 si giudica sul PIN, mai su di essa.
4. Equivalenza rw (az.rev. #3): istruttoria sui rami di `field_write_walk`
   (rw=false, usato dal fast) vs `field_set_op` (rw=true, pieno) — esito = o
   PROVA di equivalenza in perimetro argomentata per NOME dei rami, o
   allineamento con fix; a verbale, non assunta.
5. Ordine: TUTTO dopo la chiusura della finestra di misura (coppia WP + ORM):
   niente build né LSP mentre le gambe girano. Se la finestra sfora la
   sessione, i punti 3–4 passano a S-140 DICHIARANDOLO (le fixture emendate e
   questo criterio restano committati come vincolo).
