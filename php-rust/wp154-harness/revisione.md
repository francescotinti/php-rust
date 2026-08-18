# Revisione S-154 — revisore singolo, lente PROCESSO

## VERDETTO: REGGE CON RILIEVI

L'ordine degli atti è pulito e verificabile in git: criterio 20:24:11 (6bbed35) → edit 20:26:50 (cea0e8f) → attesi promossi dal secondo attore 20:30:49 (ef63870) → smoke 20:39 → R=5+emenda §6-bis 20:56:19 → arbitrato 20:56:49 → promo; ogni emenda è committata PRIMA del run che la usa (t2: 21:05:29<21:05:33; dente: 21:12:11< avvio t3 21:12:16 < pin 21:13:00). Tentativi bruciati agli atti. rcb=0 autoritativo. Il claim regge.

## Rilievi

1. **Arbitrato §6-bis = macchinario nuovo non collaudato.** s154-arbitrato-bt.sh (64 righe) scritto e usato in 30 secondi, senza copia-diff né secondo attore — proprio l'atto che ha DECISO la promo. Coperto dalla dichiarazione in-atto (rev. S-112) e da esiti ispezionabili (parità 1200000==1200000, D=+0,0 A=B=450,0), quindi non invalida; ma è sotto lo standard applicato al giudice, e la lettera di REGOLE §2 non lo copre solo perché non è pin/stash/golden.
2. **Il rimedio resta tick-limitato.** A N=600000 il rumore drop-1 è 16,7 = 1 tick ⇒ soglia effettiva 16,7, non 4: l'arbitrato refuta il morso da −66,7 ma non certifica assenza di regressioni 4–16,7 ns. La diagnosi «sotto-risoluta» dell'emenda si applica anche all'arbitrato.
3. **STOP prevedibili bruciati.** t1 è morto su un meccanismo già nominato alle 19:59 (pin non cold-riproducibile) e non incorporato nella copia delle 20:58; t2 sul dente A4 che aveva morso identicamente in S-153. Gate che mordono = funzionamento, non incidenti per la lettera; ma il 19 (=) andava difeso nel verbale, e la lettura del rcb stale dal monitor non ha lasciato traccia nel record: se ha orientato un esito dichiarato, si conta.
4. **Identità a contenuto: residuo nominabile, non nullo.** Il classificatore etichetta LC_UUID per euristica (s<4096, ≤16 B) senza verificare l'offset reale del comando, e «firma» copre tutto __LINKEDIT (anche symtab). Qui il rischio è quasi nullo: dimensioni uguali, 48 B totali, e tutti i gate funzionali + conferma post-pin (D=+22,0, 5/5) misurati sul PIN stesso.
5. **Bracci copiati a mano** (gemelloA, ce1-cand): lettera di §2 tesa; sostanza retta dalla verifica hash a ogni uso (2023cbb9/e634d95c).

## Azioni

1. Norma in §3: ogni arbitrato/derivato di guardia nasce come copia dichiarata con diff e verifica meccanica PRIMA del run.
2. S-155: ri-risolvere davvero backtrace (N tale che tick ≤ soglia/4, ≥2,4M iter) sui bracci di record.
3. Al criterio, pre-dichiarare i gate che si sa morderanno (cap dente per righe aggiunte; identità a contenuto se il candidato nasce in target dedicato).
4. Estendere pin-phpr.sh con modalità braccio/candidato (copia+hash+registro in un atto).
5. Nel verbale: una riga che difende il conteggio incidenti e mette agli atti la lettura stale di rcb.
