# Revisione S-114 — lente SEMANTICA (revisore singolo, REGOLE §7)

**Verifiche fatte**: log 8cf0b61→0b04d7b; pickaxe `PHPR_S114_BALLAST` (solo nel criterio: il codice della zavorra NON è in alcun commit; 7fe8d4f^=2fb3d1f = soli script). Diff 2c18b2e riletto riga per riga contro prop_get_entry (run.rs:571-627), prop_set_entry (636-727), BinaryTCPropSetPop (4321-4341), write_property_at (oop.rs:84-102), read_slot (arrays.rs:829), lazy_prop_access (mod.rs:12907). Letti s114-ab-la.sh, s114-admission-nulla.sh, i tre verdetti, l'istruttoria. 8bb395c verificato = commit ORIGINALE S-113.

**Esito rilettura L-A**: guardie verbatim (IC, cid+1, lazy, enum, typed_refs, Undef); ordine operandi binary_fast identico (lhs=prop, rhs=const); miss sempre conservativo (Ref, Undef, INIT_PROPS, binary_fast None) senza stato toccato; ip+2 corretto (pre-incremento run.rs:1135); pista proxy-inizializzato dissolta: `lazy` resta Some sul proxy inizializzato, quindi la guardia lazy copre. Nessuna divergenza trovata.

**Il punto che ridimensiona**: la fedeltà del B misurato non fu mai verificata A RUNTIME. I dump di admission sono ciechi PER COSTRUZIONE su un peephole runtime-only; nessun verbale documenta un test di batteria che eserciti il double-hit; `user_cpu` scarta l'output (`>/dev/null`) e l'A/B non lo confronta mai. Il «verbatim» regge sulla sola lettura del codice — che oggi controfirmo — non sull'apparato. Sulla zavorra: sorgente mai versionato ⇒ «guardia mai settata» non riverificabile dal repo.

**Verdetti**:
1. **REGGE nel merito** (braccio Throw mai eseguito nei micro puliti ⇒ nullità runtime solida), **RIDIMENSIONATO sulla verificabilità**: zavorra irrecuperabile da git.
2. **REGGE** (verbatim controfirmato da rilettura indipendente; «direzione firmata» NON riqualificata). Ridimensionata la catena di evidenza: zero parità di output, copertura del sentiero fuso non stabilita.
3. **RIDIMENSIONATO**: «identici oggi ⇒ ieri ambientale» è abduzione con UN rerun per tree: non distingue one-off ambientale da test flaky; senza log S-113 anche il comando di ieri è inconoscibile. «Nessuna lettera-gate» preclude l'ipotesi flaky. Giusto il sanamento dei log.

**Azioni**:
1. A/B: aggiungere `diff` output candidato-vs-pin per ogni micro, dentro lo script, gate su rc.
2. Ogni candidato misurato (anche da scartare) committato su ramo o patch nei raw: mai più sorgente irrecuperabile.
3. Leve runtime-only: admission con caso PHP mirato che esercita hit E miss del sentiero fuso, output confrontato al pin.
4. Batteria: riformulare in «non attribuibile a H-P1 con l'evidenza disponibile» e ripetere N≥3 su 8bb395c per stanare flakiness prima di dire chiuso.
5. Criterio prossimo: pre-registrare che spread_A oltre soglia invalida la MISURA (rerun automatico), non la leva.

---
**Recepimento in S-114 stessa**: az.2 → patch zavorra ricostruita e VERIFICATA per hash (v. s114-zavorra.patch + commit); az.4 → verdetto batteria e rotazione riformulati («non attribuibile a H-P1 con l'evidenza disponibile», flaky non escluso, N≥3 in apertura); az.5 → recepita nel §S-115 punto 1 (spread invalida la misura → rerun, non la leva). Az.1 e az.3 → pre-registrate come vincoli del criterio emendato S-115.
