# WP_SESSION_108 — S-108: la coppia WP assolve il lotto-1 (full 1,855) + LOTTO-2 di superistruzioni (prop 8,5→8,0)

**In una frase**: abbiamo rimisurato l'intera suite WordPress nei due modi e
il motore NON ha pagato il codice aggiunto ieri (full 1,855×, il rapporto
migliore mai registrato; parità per NOME intatta); poi abbiamo promosso a
cifra due voci rimaste in dubbio (arr, re) e spedito un SECONDO lotto di
istruzioni fuse — l'accesso alle proprietà degli oggetti e l'aritmetica
composta ora costano meno — con batteria, corpus, fixture e i gate
doctrine/symfony tutti verdi.

**SCOREBOARD** (micro R=5 sul pin finale 3b3d25e2, N emessi):

| giudice | S-107 | S-108 | trend |
|---|---|---|---|
| **aritmetica** | 9,7 | **9,4** | ↓ (W10; A/B +2,6 5/5 sotto soglia 4: direzione firmata) |
| **proprietà** | 8,5 | **8,0** | ↓ −0,5 (W9a+W9b; A/B Δ=+7,33 5/5 SOPRA soglia) |
| **chiamate** | 5,3 | 5,3 | = (W13; A/B +2,5 5/5 sotto soglia: direzione) |
| **stringhe** | 6,2 | 6,2 | = (INVARIANTE atteso: controllo non-regressione ✓) |
| **array** | 3,9 | 3,8 | ~↓ (A/B dentro rumore; cifra arr promossa a parte: +11,67 ns/inner-iter vs S-106) |
| **regex** | 3,4 | 3,5 | ~ (banda tra-sere; cifra re promossa: +13,75 vs S-106, margine 2,50 dichiarato) |

WordPress: **full ON 1,855× / OFF 1,885× RIMISURATI OGGI** (banda [1,86;1,93]:
sotto → leva sbloccata; voce S-105 full-off 1,947 CHIUSA per NOME) · media
2,677/2,747 (resta voce aperta, solo direzione) · parità per NOME ×2 modi.
**Leve perf spedite: 1** (lotto-2). run_loop 288.920 B (+14.728 dichiarato,
bl 29 invariato); coppia WP DOVUTA in S-109 (collaudo icache del lotto-2).
Contatore sessioni-senza-Δ-rapporti: 0.

**Data**: 2026-08-07 (11:3x–14:0x). **Modello**: Fable 5. **Commit**: d384392→(chiusura) pushati.

## Esiti secchi
1·coppia bimodale (criterio PRE, gate per NOME, solo wp_is_stream) → 2·azioni revisore S-107 SALDATE ×4 (arr/re a cifra con N maggiorato; «build gemella» marcata; diff batteria per NOME 1742=1742 VUOTO) → 3·census secondo giro → criterio lotto-2 PRE (af5d31d) + emendamento W9→W9a/W9b PRE-implementazione (74e0d62: prop_get/set possono SOSPENDERE in frame __get/hook — la finestra legale termina lì) → implementazione 4 op (ec03838) → admission PASS (OFF al byte ×6, emissione esatta ×6, Op==48, bl invariato) → A/B PROMOSSO → **PIN S-108 = 3b3d25e2 = il binario dell'A/B** (batteria 1740/0 rc=0; 2 lettere-gate emendate DICHIARANDO: funnel BinarySTDst→BinarySCSCDst, census offset +4; corpus 1415×2 IDENTICO; fixture ×5; ORM 3E/13F per NOME; hk 0E/0F) → regrade server RINVIATO per NOME. Lettera-gate STORICA sanata: build zval-census rotta da S-106 (12 forme classificate in liveness). Nota: «rc da pipe» morso per la 3ª volta (1° giro batteria annullato). NOTA revisore (lente semantica): claim regge 5/6 piste; la guardia W13 NON copre il fold specchio [PushConst,LoadVar,Cmp] — de-ottimizzazione rara (valore identico), sub-claim «emissione lotto-1 invariata» EMENDATO a verbale; azioni in S-109.

## ⭐ Lezioni (max 3)
- ⭐⭐ **Una finestra fusa TERMINA al primo helper sospendibile**: prop_get/prop_set entry possono rientrare nella VM (frame __get/hook) — una finestra che prosegue oltre perde le op residue. Vincolo ora scritto nel criterio e nei doc delle op.
- ⭐⭐ **Un dente cfg-gated che nessuno compila non esiste**: la build zval-census non compilava da S-106 e nessuna batteria se n'è accorta (la feature non è nella batteria). O il dente entra in un gate compilato, o va dichiarato dormiente.
- ⭐ **Conservare il binario dell'A/B (un cp) ha pagato**: la .text delle build gemelle DIVERGE davvero (444k/659k righe) — stavolta il pin È il binario misurato: attribuzione senza riserva.
