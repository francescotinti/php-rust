# Criterio S-135 p.3b — modello del TEMPO di AssignPath (ISTRUTTORIA, stile S-129) — commit PRIMA del run

1. Oggetto: chiudere il budget dei **+183,3 ns/iter** del canale dominante
   della bisezione (m3_int0: `$map[0]=1`, macchineria dim-set) in segmenti
   NOMINATI sul sorgente, unilaterale phpr (indizio per la leva, mai cifra
   comparativa — REGOLE §4).
2. Strumento: build EMENDATA nel worktree s134 (mai pinnabile), modulo
   `s135tp.rs`: UN segmento attivo per run (`PHPR_TP=<seg>`, parse una
   volta), accumulo ns via `Instant`; segmento inattivo = un branch.
   **Calibrazione: seg 9 = span vuoto** nello stesso arm → overhead della
   coppia di letture, sottratto e dichiarato. Giudice: `m3_int0.php`,
   R=3 per segmento, mediana; parità stdout vs oracle.
3. Segmenti: 0 arm AssignPath intero · 1 pop value+pop_keys (Vec) ·
   2 probe as_arrayaccess (incl. base_cell) · 3 path_op intero ·
   4 apply_last intero · 5 set_returning_displaced · 6 gc_note(dropped) ·
   9 span vuoto (calibrazione). Derivate: walk = 3−4−6; chiusura =
   (1+2+3)/0 dichiarata (S-129: ≥90% o modello INCOMPLETO a verbale).
4. Lettura pre-registrata: la leva si nomina sul segmento dominante SE
   >35% del budget arm; UB falsificabile della leva = prezzo misurato del
   segmento (+ spread del giudice). Sotto il 35% ⇒ «morte per N tagli» a
   verbale e la leva NON si forza (nessuna leva di ripiego).
