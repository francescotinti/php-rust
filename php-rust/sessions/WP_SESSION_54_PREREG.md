# WP-54 — Pre-registrazione (metodo WP-45 applicato alla CPU), scritta PRIMA di leggere i sample

> Scopo: attribuzione CPU-SECONDI del full-master (run42, binario corrente
> = phpr-wp53, sha256 e75c5abb…) via `sample` a finestre + tabella
> decisionale canale→leva fissata ADESSO, prima di qualunque numero.
> Regola di apertura (prompt utente): si apre SOLO la leva del canale
> quotato più alto; le altre restano quotate a verbale. Lente obbligatoria:
> ns/evento × frequenza, mai conteggi (lezione WP-53).

## Protocollo di misura

- run42 = full-suite detached (run-full-detached.sh phpr), uploads azzerati
  prima; PID master dal pid-file (MAI pgrep nudo: i test possono spawnare
  figli phpr — lezione WP-22 adattata).
- 5 finestre `sample <pid> 30 1` a ~2', 5', 8', 11', 14' dalla partenza
  (copertura early/mid/late: il mix di test cambia lungo la suite).
- Lettura ad ALBERO (lezione WP-41): i secondi si attribuiscono al
  SOTTOALBERO semanticamente proprietario, non alla foglia (es. malloc
  sotto PhpStr::new → canale stringhe E canale malloc, doppia contabilità
  esplicitata).
- Riconciliazione: Σ(canali) sulla finestra media ≈ 100% dei sample entro
  ±10-15%; run42 è run di MISURA (il sampling perturba): il suo master-CPU
  NON sostituisce run40/41 come riferimento pubblicato.

## Tabella decisionale pre-registrata (canale → leva)

| Canale (sottoalbero) | Soglia di apertura | Leva pre-registrata se vince |
|---|---|---|
| malloc/free totale — quota stringhe (PhpStr/ZStr alloc+drop) | ≥8% del campione | Fusione single-alloc PhpStr via `Rc<[u8]>`/`Rc<str>` (safe, niente crate DST) — se il costo stimato > 1 sessione ⇒ diventa il mandato di WP-55, qui solo quota |
| malloc/free — quota args-Vec per call | ≥2% | Fase 2.3: args-Vec pool bounded (modello FramePool 64×512) |
| classify/walk GC (collect_cycles/classify/mark) | ≥8% | Profilo fine del walk: riduzione lavoro DENTRO i corpi (hoist borrow, iterazioni fuse); MAI guardie/bound negli arm caldi (WP-50) |
| borrow/borrow_mut RefCell (check dinamici) | ≥5% | Hoisting dei borrow fuori dai loop interni nei siti top-3 del profilo |
| hashing (FxHash/lookup HashMap) | ≥5% | Pre-hash o IC sui 2-3 siti dominanti |
| dispatch run_loop (overhead match/branch) | qualunque | NESSUNA APERTURA: arco registri CHIUSO (WP-44); micro-op in conteggi = NON riproporre (WP-53) |
| IO / mysqli / syscall / kernel | qualunque | Fuori perimetro (functional-parity; non è CPU nostra) |
| non attribuito / resto | >15% | La riconciliazione NON passa: correggere la tabella PRIMA di ogni scelta (gate Fase 0) |

Ob.3 reflect-cache owner (re-key su (declaring class, mname) dopo la
resolve, host_reflect.rs:416) è ORTOGONALE alla tabella: si esegue se resta
margine, con census reflect (split mock-declared vs inherited); ritorno cap
16384→8192 = decisione utente a dati pronti.

## Predizioni pre-registrate (da falsificare coi numeri)

1. malloc/free aggregato = 12-20% del master-CPU; quota stringhe la metà o
   più (51,8M×2 malloc sul media ⇒ sul full atteso ~4-8× tanto).
2. classify/walk = 6-10% (62,7s census WP-52 su ~740s ≈ 8,5%, già ridotto
   −42,7% in WP-52: atteso residuo ~4-6% se la riduzione ha tenuto).
3. borrow RefCell diffuso ma sotto-soglia come canale proprio (<5%) —
   comparirà DENTRO classify e dispatch.
4. dispatch/interprete (run_loop al netto dei canali sopra) = 35-50%.
5. ns/evento stringhe: 40-80ns/malloc-pair ⇒ fusione single-alloc = −1 
   malloc+free per stringa ⇒ stima −2-4s sul media (59s) e −15-40s sul full
   se le frequenze scalano; è la candidata attesa a vincere la tabella.
6. args-Vec pool: ~1 alloc/call method/static; atteso <2% ⇒ quota a
   verbale, non apertura.

## Pre-registrazione 2 (dopo la tabella Ob.1, PRIMA del census walk/reflect)

Tabella Ob.1 (run42): l'unico canale pre-registrato sopra soglia =
classify/walk (77,4s = 10,0% full; media ~5%). Leva candidata dentro il
canale: **leaf-object bit** (bit conservativo "may_hold_containers" su
Props, mantenuto ai siti di mutazione — forma sanzionata WP-50; pattern
`PhpArray::may_hold_containers` già in-codebase), che salta borrow+iter
per gli oggetti foglia in pass1/pass2/white-rebuild di `gc_classify`.

Soglie di apertura (fissate PRIMA di leggere i contatori walk del census):
- SI implementa se: quota slots-foglia ≥ 35% degli slot oggetto walkati
  (upper bound del lavoro d'iterazione saltabile) ⇒ stima classify −20%+
  ⇒ ~−1,5-3s media-classify e −10-20s full — quota in secondi ≥ di ogni
  altra leva quotata oggi.
- NO se < 20% (quota a verbale, canale resta "ripiego WP-52").
- Zona 20-35%: si implementa SOLO se il costo di mantenimento stimato ai
  siti di mutazione resta fuori dagli arm caldi del run_loop (store di
  prop = sito di mutazione, ammesso da WP-50).
Erosione attesa (da verbalizzare, non misurabile ex-ante): i `&mut`
(get_mut/get_slot_mut) avvelenano il bit conservativamente.

Reflect re-key (Ob.3): si implementa il two-level (resolve-map leggera
(cid,mname)→decl + descrittori su (decl,mname)) se il census mostra
inserts inherited ≥ 50% degli inserts (= i duplicati collassabili
dominano la cardinalità). Sotto il 30%: solo verbale. Il ritorno del cap
16384→8192 resta decisione utente a dati pronti.

## Kill-switch / vincoli

- Niente leve sul dispatch (WP-44), niente elisioni sweep oltre whitelist
  (WP-53), niente guardie negli arm caldi (WP-50).
- Ogni leva eventualmente aperta: gate corpus per NOME (baseline 1421) +
  cargo --release; se emit/GC/layout: full BYTE-ID; giudice = full
  stesso-giorno new vs old phpr-wp53 + ab54 6 round interleaved (guardia
  footprint peak fisico /usr/bin/time -l).
