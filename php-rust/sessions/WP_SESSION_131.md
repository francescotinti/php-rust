# WP_SESSION_131 — riferimento WP rifatto su s130 + modello prop_step + E1-KO SPEDITA (pin s131)

**In una frase**: la velocità di WordPress è stata rimisurata sul motore nuovo
(rapporto confermato ~1,76–1,80, tutte le gambe pulite grazie al giro di
riscaldamento), una sonda ha dato un nome a ogni nanosecondo della scrittura di
proprietà, e la cura che ne è uscita — cercare il nome della proprietà UNA volta
invece di cinque — è stata promossa su tutta la catena di collaudo.

**SCOREBOARD** (pin NUOVO **s131 ff66cb84** + server s131 97ed6e06; micro gate promozione):
**arith 5,6 ↗ · prop 5,6 = · calls 4,9 ↘ · str 4,3 = · arr 3,2 ↘ · re 2,5 =**
(mosse ≤0,1 = jitter denominatore) · **WP full = 1,757–1,797 PULITO** (RIMISURATO
@ s130; da rifare su s131: E1-KO morde FieldAssign) · **leve spedite: 1 (E1-KO)** ·
incidenti: 1 app. (gate quiescenza auto-morso da «phpr» nell'argv dell'arbitro —
fermo PRIMA di ogni misura; fix path neutro, a verbale). 2026-08-11.

## Esiti secchi
1·**Coppia WP su s130** (criterio pre-registrato 04d1afa; warm-up media-only
  dichiarato): 4/4 gambe PULITE (mediane ictx/s PER MOTORE 1248/173), quiescenza
  rc=0 ×5 in header · **full 1,757–1,797** (user-only 1,766–1,809) · **media
  user-only 2,441–2,479** · peak 1788–1887 MiB · parità per NOME 4/4 → REPORT_GAP_131.
2·**Modello prop_step ACQUISITO** (rc=0, chiusura 93–94%): E−E2 166,9 = prop_step
  interno 130,7 (guardie 49,4 · defer 37,0 · key+op 34,3 · borrow 1,5 · altro 8,5)
  + dispatch 36,3; resolve statement 40,3 su 4 siti + prop_indirect_guard (≈0,
  enumerata); ctor 70,8 (17,7/resolve). Az.rev. S-130 #4 chiusa CON rerun.
3·**LEVA E1-KO SPEDITA** (criterio 76f9251 PRE-registrato → codice b87d7fa):
  resolve-once in field_write_prop_step (key0/declared0/denied0 ≡ per costruzione)
  · smoke R=2 D=+45,0 vs 6,7 · R=5 D=+23,3 vs 13,3 (rumore drop-1 10,0/13,3),
  guardie 9/9 · promozione COMPLETA rc=0 (batteria 1746/0/2 inventario identico,
  corpus 1414, fixture per NOME, ORM 16 nomi, hk 0E/0F) → **pin s131**.
4·**Micro-ORM su s131**: objdatains **7,5** (1220,0, −33) · churn **8,3** (1490,0)
  · dropdef **8,8** · alloc 7,8 = (ctor non toccato ✓) · allocni 9,8 · objmap 17,3.
5·**Az.rev. S-130 tutte chiuse**: #1 pin-server committa il registro nell'atto
  (provato in catena: 7b36a02) · #2 inventario fixture per NOME (niente più «7/6»)
  · #3 rc quiescenza in ogni header · #4 emenda E1a + quota call-site · #5 già s130.
## ⭐ Lezioni (max 3)
- ⭐⭐ Un modello per-blocchi con conteggi deterministici NOMINA la leva e ne predice
  l'UB: E1-KO promossa alla prima uscita, D dentro la forchetta del modello.
- ⭐⭐ Il warm-up leg dichiarato assorbe l'effetto prima-di-sequenza: 4/4 pulite dove
  S-128/S-129 scartavano le prime gambe.
- ⭐ Un gate a pgrep -f si auto-morde se l'argv del lancio contiene il pattern: path neutri.
