# WP_SESSION_134 — LEVA IC non-plain SPEDITA (pin s134) + 4 az.rev. processo + coppia 4/4 pulita

**In una frase**: ora il motore ricorda dove sta una proprietà tipizzata dopo la
prima scrittura (una cache al posto di 5 ricerche ripetute) — creare oggetti è
~14% più veloce (objalloc 7,5→6,6) e il registro/collaudo dei binari è sanato.

**SCOREBOARD** (pin NUOVO **s134 61896da1**3654fd00 + server s134 461bfb55; micro gate promozione):
**arith 5,4 ↘ · prop 5,5 = · calls 5,0 ↗(denom.) · str 4,2 = · arr 3,2 = · re 2,5 =**
· **WP full ON-ONLY = 1,769 (N=2 CONCORDI — prima banda propria) @ pin s134**
(off 1,748–1,789; media 2,405–2,467; peak 1818–1884, bordo alto persiste)
· **leve spedite: 1 (IC non-plain)** · incidenti: 0 nuovi (storici 11).

## Esiti secchi
1·**Az.rev. S-133 4/4 SPEDITE**: PIN_REGISTRY sanato (13 righe server rientrate,
  righe phpr s132/s133/s134; pin-server.sh appende nella SUA sezione, fail-closed,
  verificato dal vivo in promo) · gate `stash` in catena s109 (9 gate, ri-hash 4
  pinnati vs registro, BITE 0/1/3/9 esatti) · `scripts/copia-gate.sh` (diff vs
  manifest AL BYTE; replay incidente sed S-133 MORDE rc=1) · teardown a catalogo
  (parità SUI 7 vettori, non oltre) · commenti stantii s133-promozione sanati.
2·**Eccedenza s133 NOMINATA dal disasm** (az.rev. #4, s134-eccedenza-lettura.md):
  12 siti bl invariati (10→resolved), la resolve rimossa era la SECONDA di una
  coppia back-to-back — costo di sito non modellato dal prezzo medio 17,7;
  magnitudine +11,3 NON ripartita. Lettura → scelta leva (dispatch = rischio H-C2).
3·**LEVA SPEDITA** (criterio 9585374 → codice 70f078a): bit NP/TY nello slot del
  PropIc esistente; fill dal cammino pieno solo con fatti di classe provati (no
  hook, no `__set` — load-bearing per typed-unset —, asym ok, non readonly,
  key==name); hit = guardie plain + coerce typed in ordine invariato. Sonde 10
  bilaterali: 3 diff TUTTI pre-esistenti (byte-id al pin s133). A/B R=5:
  **objalloc D=+136,7** (soglia 26,7) · **objdatains D=+133,3** (13,3) ·
  riconciliazioni in banda · eccedenza vs modellata 35,4 PRE-dichiarata ·
  guardie 8/8 · disasm: nessun flip (resolve 21=21, magic 10+2) · promozione
  rc=0 (1746/0/2, corpus 1414 ×2, 9 gate, ORM 16 nomi, hk 0E/0F) → **pin s134**.
  Submicro: objalloc **6,6** (813,3) · objdatains 6,4 · churn 7,4 · dropdef 7,9
  · allocni 7,9 · objmap 17,3 =.
4·**Coppia WP t1: 4/4 gambe PULITE** (prima volta dal s131): on-only 1,769 N=2
  concordi; la leva NON muove WP (atteso) → REPORT_GAP_134. dbal/ORM: finestra
  insufficiente per la ricetta pulita ⇒ rimisura DOVUTA in testa a S-135.
## ⭐ Lezioni (max 3)
- ⭐⭐ Dichiarare PRIMA i componenti non prezzati trasforma un «FUORI UB» in
  riconciliazione: l'eccedenza D−35,4 era attesa e nominata, non un'anomalia.
- ⭐⭐ Il dente delle copie (manifest AL BYTE) ha collaudato 3 copie in un'ora:
  il costo del collaudo è crollato sotto il costo dell'incidente che previene.
- ⭐ pin-phpr.sh non scrive il registro (la riga phpr si aggiunge a mano):
  candidato az.rev. per chiudere la malattia alla radice.
