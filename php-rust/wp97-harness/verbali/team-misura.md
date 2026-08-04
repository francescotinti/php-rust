# Team-misura (Bak · Leijen · Gregg) — nota di riconciliazione WP-97

## Convergenze

1. **Nessuna refutazione di merito: F1/F2 e la prosecuzione reggono.** Conteggi esatti, deterministici, negativo che morde (Bak, Leijen); la decisione sopravvive perfino a un canale sovrastimato 2× (Gregg §3, Bak "robustezza": safe cade in MEDIA 0,95% ma la strada lunga resta preferita).
2. **La banda P3 è SCREEN, non VERDICT.** Il canale viene da prof95-media.out (R=1, auto-dichiarato SCREEN); grado del prodotto = fattore più debole (Gregg §1-2 + refutazione di grado; Bak: banda ALTA è decisione SCREEN-grade; Leijen KS-DL-97-2 sul lato footprint). Contaminazione nei raw: `grade=VERDICT` copre SOLO i conteggi → A-BG-97-1.
3. **Il canale è un tetto: i drop DEALLOCANTI non sono eliminati, solo anticipati.** TakeSlot evita la coppia transiente inc/dec, non la free finale (Bak §bande; Leijen A-DL-97-3; Gregg §4 asimmetria guard/store Undef). Δ sotto banda con controllo positivo verde = «modello di costo sbagliato», da nominare ex-ante (A-DL-97-3, A-LB-97-3, KS-BG-97-3).
4. **Protocollo F4 comune**: controllo positivo su binario census SEPARATO dal cronometro, stesso HEAD (A-LB-97-2 ≡ A-BG-97-4); predizione SCRITTA prima del run; R e tetto spread A/A dichiarati prima, esito UNDECIDED se lo spread copre il floor (A-BG-97-3, KS-BG-97-1); identità `takes+fallbacks=would_take` (A-LB-97-2, KS-LB-97-1/2, KS-BG-97-2); ns/evento prima di F4 (A-BG-97-2, regola WP-53/54).
5. **Nucleo `_str` favorito nella decisione di perimetro F3**: rischio distruttori zero e canale riuso stringhe massimo (Leijen A-DL-97-2; Bak A-LB-97-4/KS-LB-97-3; Gregg A-BG-97-5: banda RI-derivata 0,84–1,21%, non ereditata).

## Conflitti

- **Enfasi CPU vs footprint**: Leijen esige che F4 misuri ANCHE il peak fisico con predizione firmata (A-DL-97-1, canale CoW A-DL-97-2, churn purge A-DL-97-4); Bak e Gregg trattano F4 come misura di tempo. Non contraddittorio ma additivo: costo di protocollo extra da accettare o motivatamente rifiutare.
- **Corpo caldo nuovo**: solo Bak refuta la frase di design95 e impone il tetto Δ bracci caldi ≤ 0 con predizione `nm -S` (A-LB-97-1, KS-LB-97-4); Leijen/Gregg non lo toccano.
- **Reazione al falsificatore alto**: Gregg → profilo di coppia obbligatorio (KS-BG-97-3); Leijen → prima candidata il canale CoW non contato (A-DL-97-2). Ordine di attribuzione da fissare.

## Priorità proposte per WP-96

1. Declassare la banda P3 a SCREEN nei raw/companion (A-BG-97-1) prima di ogni uso.
2. Decisione perimetro F3 a inizio sessione col conto dei raw; se `_str`, ri-derivare la banda (A-LB-97-4 + A-BG-97-5).
3. F3 sotto tetto corpi caldi + op-census invariante (A-LB-97-1).
4. F4: protocollo completo — identità contatori, binari separati, spread A/A ex-ante, ns/evento, predizione footprint firmata (A-LB-97-2, A-BG-97-2/3/4, A-DL-97-1).
5. Falsificatori nominati prima di misurare: sotto-banda e sopra-2×banda con reazioni pre-decise (A-LB-97-3, A-DL-97-2/3, KS-BG-97-3).
