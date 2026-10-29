# Revisione adversariale S-186 — lente PROCESSO

Generato con assistenza AI (Claude Fable 5).

**Claim**: apparato della notte pre-registrato e collaudato a secco.

## Ciò che lo invalida

1. **Il mutante M uccide la rotta B per costruzione.** Lo spin è `Instant::now`/`elapsed` in ciclo nel braccio `CallBuiltin` di `run_loop` (s186-leva-build.sh r.94-95). I campioni xctrace cadono nei frame del clock (libsystem_c → libc, libsystem_kernel → kernel = pavimento comune) o nel self di `run_loop` → dispatch: in classe builtin non entra nulla. La quota «≥75 % di ΔM in builtin» (criterio p.6) fallisce qualunque sia la realtà ⇒ MUTO ⇒ rc=4. La build collauda solo user CPU su str.php (r.170), mai l'attribuzione per classe.
2. **Ordine (3) del concilio aggirato nella sostanza.** Cura O(1) SOLO se la guardia morde, «altrimenti s184 com'è». La cura è già committata (f600e32f) e il criterio p.10 esige rc=0 su b2 in ogni caso: «s184 com'è» non esiste più. Stessa forma del kill S-168.
3. **REGOLE §8 violata due volte nella sessione che l'ha firmata**: cap `loc_dente` alzato in 901515b6 e f600e32f, non commit di promozione rc=0.
4. **Punto (1) dell'ordine non eseguito**: nessun `.expected-diff` di fx-mc2 nel repo ⇒ per §5 gate VOID; s186-leva-build.sh r.60 pretende pin == oracle byte-id, forma incompatibile con §5.
5. **A registro**: apparato oltre il timebox per parola dell'utente (29/10) = incidente §1; daemonizer senza mkdir del log (§10); apostrofo in `${VAR:?}`; alle 22:03 le 5 build NON sono in corso (lanciatore in attesa della quiete CI), catena non lanciata; E3 su dev-release non è verdetto; batteria 1752 e corpus NON eseguiti sulla cura O(1) (solo lib 707/0 + 8 fixture).

## Azioni (M non è ancora costruito: costa zero)
1. Collaudo a secco del mutante: xctrace su P e M con un driver a molti builtin + `xc-classi.py`; se ΔM non cade in builtin, spin senza clock in funzione `#[inline(never)]` dal nome catturato dalla regola builtin, oppure emendare p.6 a «ΔM totale ≥ 2,1 s» dichiarando la perdita del test di attribuzione.
2. Emenda firmata all'ordine (3): «cura O(1) incondizionata, b2 = gate di promozione», oppure dire quale commit diventa s184 se b passa.
3. Registrare gli incidenti §8 (×2) e §1 nel verbale di sessione.
4. Committare `fx-mc2.expected-diff` (anche vuoto) e allineare r.60 della build a §5.
5. Batteria 1752 e corpus-gate rc=0 su f600e32f prima di aprire la finestra.

**ESITO: NON REGGE** (azioni 1-2 prima del lancio).
