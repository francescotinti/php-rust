# S-176 istruttoria «drift della sentinella oracle ORM» (rilievo 5 revisione S-175; NEXT §S-176 p.6)

## Reperto (oracle_user della gamba ORM, s, dai verdetti `s1NN-orm-coppia-verdetto.out`; net = rapporto phpr/oracle)
| sessione | gamba 1 | gamba 2 | fuori banda [4,83;4,94] |
|---|---|---|---|
| S-150 | 5,06 | 5,03 | (banda non ancora esistente) |
| S-154 | 4,98 | 4,99 | — |
| S-155 | 4,98 | 4,93 | — |
| S-157 | 5,00 | 5,00 (replica 5,02) | — |
| S-158 | 5,03 | 5,08 | — |
| S-159 | 4,95 | 4,97 | — |
| S-160 | 5,02 | 5,00 | — |
| S-161 | 5,42 | 5,31 | (outlier dichiarato) |
| S-162 | 4,93 | 4,96 | finestra di FONDAZIONE (ORA_REF 4,885 = media 4,87/4,90) |
| S-163 | 4,91 | 4,90 | — |
| S-164 | 4,92 | 4,91 | — |
| S-165/166 | 4,93 · 4,88 | 4,94 · 4,98 | 4,82 (s166 orm2); R=5 s165 mediana 4,860 ⇒ banda [4,83;4,94] |
| S-171 | 4,94 | 5,01 | 4,95 |
| S-172 | 4,93 | 4,94 | — |
| S-173 | 5,00 | 4,99 | (4,99/5,00 a filo: non segnalata dal copione? t19 «a filo») |
| S-175 | 5,03 | 4,98 | 4,97 |

## Lettura
1. La banda [4,83;4,94] (larghezza 0,11 = 2,2 %) fu fondata su UNA finestra fredda (S-162…S-166: 4,88–4,98, mediana 4,92). Prima di quella finestra (S-150…S-160) l'oracle stava a 4,93–5,08 (mediana 5,00); dopo (S-171…S-175) è tornato lì (4,93–5,03, mediana 4,98). Il «drift» non è una deriva monotona ma il RITORNO al livello storico: la finestra di fondazione era la coda bassa, non il centro.
2. Spread storico delle gambe pulite (S-150…S-175, S-161 escluso): 4,88–5,08 = 0,20 s (4 %), il doppio della banda. Una banda al 2 % su una grandezza che oscilla al 4 % marca «fuori» una gamba ogni due sessioni: E2 (Delta_norm non giudicante) è diventata la regola, non l'eccezione ⇒ la cifra ORM è stata NULLA in S-166, S-171, S-175 senza che il motore c'entrasse.
3. Nessun segnale di causa nel motore o nel binario oracle (8.5.7 invariato); i rapporti net restano in [6,94;7,15] su tutto il periodo, cioè phpr e oracle si muovono INSIEME (stato termico/carico della macchina), che è esattamente ciò che Delta_norm dovrebbe assorbire — se ORA_REF sta al centro.

## Emenda PROPOSTA (si applica SOLO rieseguendo il criterio emendato, REGOLE §3 — prossima coppia ORM: s176-orm-coppia.sh + s176-criterio-orm.md)
- ORA_REF = 4,94 = mediana delle 18 gambe pulite S-162…S-175 (4,88…5,03; 9ª e 10ª ordinate = 4,94/4,94 — rilievo 6 revisione S-176: era scritto 4,96); REF band di phpr ri-normalizzata di conseguenza nel copione (net_norm = net × ORA_REF / oracle_net_gamba, come oggi).
- Banda sentinella = [4,84; 5,04] = ORA_REF ± 2 % (copre lo spread storico 4,88–5,03, esclude l'outlier S-161 5,31/5,42 e una gamba fredda <4,86 come s166-orm2 4,82). Ampiezza 0,20 = 4 %, pari allo spread osservato: una gamba fuori torna a significare «macchina anomala», non «giorno normale».
- Regola 4 (voce fuori banda da >2 sessioni senza rerun blocca la leva): la voce ORM riaperta in S-175 si chiude con la PRIMA coppia sotto il criterio emendato che dia entrambe le gambe in banda; se anche col criterio emendato una gamba esce, l'istruttoria passa alla macchina (carico/termica registrati per gamba: aggiungere `sysctl machdep.xcpm.cpu_thermal_level` e loadavg per gamba nel verdetto).
- Anti-flare (lezione 3 S-175): l'assestamento a STREAK del pair (4 campioni consecutivi <5 % di mediaanalysisd a passo 5 s, retry ×3) entra nel gate per gamba dell'ORM (quiesce_gate), al posto della sola pausa 30 s ×3 — copia dichiarata da s175-pair.sh, nessun'altra modifica al canone.
