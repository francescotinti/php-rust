# Team-FEDELTÀ (Stogov + Pedersen) — Concilio WP-105, fase 2
Relatore: team-FEDELTÀ. Fonti: verbale-8-stogov.md, verbale-6-pedersen.md.

## (a) Scoperte Stogov: catalogo vs ordine S-104
**A catalogo SUBITO (prima di ogni fix — «una regola falsa a catalogo è peggio dell'assenza»):**
- §3.12 rititolato coi TRE regimi (A-ST-105-1): (i) op-fallisce+weak ⇒ UNDEF ri-coercito allo zero del tipo; (ii) strict_types=1 **CONSERVA** (parità phpr già oggi); (iii) op-riesce+verify-fallisce (`.=` su typed int) **CONSERVA**. Il censimento 4/4 era tutto nel regime (i).
- §3.13: la marca porta (unit, line), non la sola riga — divergenza NUOVA provata su HEAD (include ⇒ file del flush; eval ⇒ pseudo-file assente). §3.13 NON è chiudibile senza fixture include+eval (A-ST-105-3).

**Lavoro d'ordine S-104 (solo se si sceglie il punto-fedeltà):**
- Fix §3.12 = replica della catena UNDEF→verify-weak per tipo dichiarato (A-ST-105-2); KS-ST-105-1: senza bracci strict e `.=` nel gate il fix NON atterra.
- Fix generator = get_gc COMPLETO (frame sospeso, $this, yield-from, bracci running/closed; A-ST-105-4); KS-ST-105-2: verde sulla sola cattura è VOID senza le fixture sorelle CV/$this.
- A-ST-105-5 (assert ai make-ref): igiene, non bloccante.

## (b) Colpi Pedersen: MINIMO vs PIENO del prossimo pin
**Nel MINIMO che grada il pin post-leva (bloccano il grading ⇒ apparato ammesso):**
- A-PE-105-1: cbE×2 consecutive, cbE a server freddo, cbE nell'interleave workers=2; fino ad allora registro = «cross-richiesta stesso-worker», MAI «cross-worker».
- A-PE-105-3: stash meccanico nel launcher (copia keyed-by-hash + sha256 + path nel `.done`; niente copia ⇒ niente «gradato», KS-PE-105-2).
- A-PE-105-4: confine giudicato con `curl -sD` (status+header stabili, carve-out volatili) su cb2/cbE — oggi si grada solo il body.

**Nel PIENO (prima di ogni cifra, KS-PE-105-1):** option 413 + restapi 3508 per NOME, env -i, ×2 modi; più A-PE-105-5 (evidenza copertura di entrambi i worker, o la gamba D si rinomina). Non spendere il PIENO su 31aa7c2e: si grada PIENO il pin che porterà le cifre.

**Catalogo (igiene §5):** divergenza symlink-docroot in `PHPR_DIVERGENCES_FROM_PHP.md` (A-PE-105-2) — decisione esplicita, non commento.

## (c) Conflitti
Nessuno sostanziale. Convergenza forte sul principio «fedeltà o assenza CONSAPEVOLE»: Stogov lo applica al catalogo (§3.12/§3.13), Pedersen al registro pin e al symlink. Entrambi confermano la spina dorsale perf (ordine 1→2→3) e relegano i fix di fedeltà a scelta esplicita.

## (d) Priorità S-104 (fronte fedeltà)
1. Igiene timeboxata: correzioni di CATALOGO (tre regimi §3.12, unit §3.13 + probe include/eval nei fixture-gate, symlink) — costano probe, non apparato.
2. Launcher: A-PE-105-1/-3/-4 SOLO al momento di gradare il pin post-leva (blocca ⇒ ammesso; prima no).
3. Se si apre il punto-fedeltà: generator get_gc completo O fix §3.12 a catena UNDEF, ciascuno coi propri kill-switch; mai fix parziali «verdi per fortuna della fixture».
