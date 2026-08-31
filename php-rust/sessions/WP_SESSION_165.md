# WP_SESSION_165 — PROMOZIONE L-MC1d (pin s165); 5/5 azioni revisione chiuse; banda-layout host-call fondata
**In una frase**: la leva sulle chiamate a metodo è stata costruita, smontata e
rimontata quattro volte contro un braccio di controllo nullo finché ogni morso
di guardia ha avuto un nome (un costo vero rimosso, il layout del binario, la
quantizzazione del cronometro), e alla fine è stata promossa: le chiamate a
metodo semplici saltano l'intero imbuto di dispatch e costano l'8,5% in meno.
**SCOREBOARD** (pin NUOVO **s165 phpr 1fd8757d2f72dc3e + server cf7afe37f29016a8**):
arith 5,4 (5,5↓tick) · prop 5,6 (5,5↑tick) · calls 4,9 (4,7↑tick) · str 4,2 = ·
arr 3,2 = · re 2,6 = (guardie A/B sugli stessi binari: D≈0 — i tick sono del
denominatore oracle, lezione S-164) · giudice proprio **mc2 170,5→155,5 ns/iter
(D=+14,5, −8,5%)** · WP t14 1,761 (riferimento, NON rimisurato: coppia dovuta
S-166) · corpus 1412×2 · **leve spedite: 1 PROMOSSA** · incidenti: 0.

## Esiti secchi
1·L-MC1d «MethodCall.borrow k≤2»: percorso B1 inline (+19,0/+17,5, guardie morse
  da layout run_loop +45 bl) → B2 outline #[inline(never)] (Δ=+4 bl, +16,0) →
  **C null-edit** (giudice-controllo +3,5; attribuisce arrload a unreachable!×2
  e fonda banda-layout missload 8,0 / arrfilter 6,0) → D leva pura +14,5 →
  R=5 emendato **+14,5 riconc. |0,0|** → backtrace/objmap ri-risolte a tick≤1ns
  (quantizzazione) → catena §6 rc=0 (batteria 1748, churn sanato al byte,
  corpus ×2, micro, pin+server da script). unreachable!×2: NON montati
  (costano ~5 ns su arrload — az.rev. S-163 #4 chiusa a verbale di misura).
2·Azioni revisione S-164 5/5: creep arith REFUTATO (Δ=+0,12<0,52, N=250M R=5) ·
  census AL3 Δ=199999 ESATTO (+1 PROVATO = buffer Vec exts via with_capacity) ·
  ORA_REF 4,885 REGGE (R=5 oracle mediana 4,860) + banda sentinella [4,83;4,94]
  ATTIVA · ricetta server provata ×2 (feature axum-server; ipotesi S-164
  corroborata) · vieto rumore>soglia applicato (rc=8 nel copione).
3·Istruttoria dbal ictx CHIUSA: firma = DENOMINATORE (assoluti equivalenti,
  rate 6-13× sul braccio corto; 6/6 oracle vs 0/6 phpr) — emenda per-motore
  proposta per la coppia S-166. Sonda strmap non scattata (nessuna leva strmap).

## ⭐ Lezioni (max 3)
- ⭐⭐ un BRACCIO NULLO (edit semanticamente vuoto) è l'arbitro che scompone i
  morsi di guardia in canali nominati: costo vero / layout / quantizzazione —
  senza, tre round di A/B restavano illeggibili.
- ⭐⭐ le guardie host-call a soglia 4 stanno SOTTO il pavimento di layout del
  binario (LTO fat: ogni edit sposta tutto): la banda-layout si fonda sui
  bracci nulli-per-categoria, mai si allarga ex post.
- ⭐ un'azione di revisione («monta unreachable!») può COSTARE sul fast path:
  si chiude col verdetto di misura, non col dente.
