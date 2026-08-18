# WP_SESSION_154 — L-CE1 SPEDITA (class_exists no-alloc −20%); coppia t5+ORM @ s153; FUORI-UB di BT2 SPIEGATO; pin NUOVO s154
**In una frase**: misurato il binario su WordPress e Doctrine (stabile il
primo, più veloce del previsto il secondo), spiegato al centesimo il guadagno
della cura backtrace, e spedita una cura che azzera le allocazioni del
controllo-esistenza-classi, promossa con tutti i collaudi verdi.

**SCOREBOARD** (pin NUOVO s154 bddc050320a6af4c + server b3cf348f69739edc):
arith 5,5 → · prop 5,6 ↑ · calls 4,8 ↑ · str 4,3 → · arr 3,3 ↑ · re 2,5 ↓
(±1 tick) · WP t5 1,757 COMPATIBILE (N=5, banda 0,013) · media 2,450–2,485 ·
ORM 7,051–7,073 · dbal 7,440–7,450 (tutti @ s153: coppia @ s154 DOVUTA →
S-155 p.1) · corpus 1412×2 ZERO flip · batteria 1748/0/2 · **leve perf
spedite: 1** (L-CE1; tentate 1) · incidenti 19 (=).

## Esiti secchi
1·Coppia t5 @ s153 COMPATIBILE (leg1 esclusa per firma; parità 6/6); ORM:
  Δ=[+0,72;+0,77] GIÙ FUORI RUMORE, OLTRE-attesa dichiarata, magnitudine
  NON ripartita (oracle −1,4% di giornata; citabile 7,10→7,05).
2·Sonda k post-BT2 rc=0: k_new=13 ESATTO (fisso 3 + 5/frame; pool chiavi e
  type = 0 alloc); FUORI-UB spiegato: alloc 214–221 + hash/memcpy ≈46–52 ≈
  D 266,7; il modello blind sbagliava sul lato PRE (chiavi ~2 alloc, campi
  doppi). 3·Testa hostcall rifondata: debug_backtrace 6,149M == 473k×13
  ESATTO; residuo 60,87M (Δ0,04%); class_exists 9,74M · get_declared_classes
  4,56M (denominatori p.3). 4·**L-CE1 VINTA e SPEDITA** (LcKey al posto di
  to_vec+to_ascii_lowercase, hit k 2→0): A/B R=5 D=+22,0 (111→89), riconc.
  2,0; guardia backtrace morsa a 1 tick esatto → ARBITRATO N=600k D=+0,0
  REFUTATO (emenda §6-bis); promo rc=0 (identità candidato a CONTENUTO 48 B
  LC_UUID+firma, emenda t2; dente A4 → cap 25712 dichiarato; fixture 10/10 +
  fx-ce byte-id ×3; conferma post-pin D=+22,0 5/5). Bracci: gemelloA
  2023cbb9 · ce1-cand e634d95c. 5·Reperto: pin s153 NON cold-riproducibile
  (banner mimalloc da cache calda + LC_UUID). 6·Sonda ce-count post-CE1
  DOVUTA (S-155). 7·Incidenti 19 (=) DIFESO: i morsi = gate fail-closed che
  funzionano (tag bruciati agli atti); il rcb STALE letto dal monitor (promo
  t2) NON ha orientato esiti di record — a verbale qui.

## ⭐ Lezioni (max 3)
- ⭐⭐ Il pin può NON essere cold-riproducibile (cc-fingerprint cieco a
  SOURCE_DATE_EPOCH; LC_UUID/firma dal target-path): identità chiusa a
  CONTENUTO con cluster NOMINATI, mai cap cieco.
- ⭐⭐ Guardia su giudice quantizzato (tick 66,7) con soglia 4 ns = sotto-
  risoluta: il morso da ±1 tick a rumore 0 si arbitra RI-RISOLVENDO N.
- ⭐ Le attese-conteggio si fondano su k MISURATI, non su enumerazione a
  occhio (il modello blind BT2: −12 alloc/call, tutto sul lato pre).
