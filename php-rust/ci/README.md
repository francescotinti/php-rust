# CI locale phpr (decisione utente 2026-08-12, S-132)

Ogni `git push origin main` arriva ANCHE al bare locale
`/Volumes/Extreme Pro/Claude/phpr-ci/repo.git` (secondo push-URL di origin);
il hook `post-receive` accoda il commit in `phpr-ci/queue/`, launchd
(`com.phpr.ci`, WatchPaths + StartInterval 600) lancia `ci/ci-runner.sh` che
consuma la coda UNO alla volta: **build release → batteria (cargo test) →
corpus-gate**, con target dir SEPARATA `phpr-ci/target` (mai php-rust-output).

Esiti: `phpr-ci/out/<sha12>/status` (OK · build-FAIL · batteria-FAIL ·
corpus-FAIL · skipped-busy) + log per passo + riga in `phpr-ci/CI_FEED.log`
+ notifica macOS. In sessione: Monitor su CI_FEED.log; a inizio sessione il
pre-flight legge il feed per gli esiti arretrati.

**Che cosa NON è**: gate di record. Pin, stash e promozione restano SOLO sugli
script canonici (REGOLE §2/§5/§6). La CI è allarme precoce per-commit.

**Mutex con le misure**: il runner aspetta (fino a 4 h) se vede orchestratori
di misura vivi (pair109 / harness/s1NN-* / watchdog) o il lock convenzionale
`/private/tmp/phpr-measure.lock`; in direzione opposta il gate di quiescenza
delle misure fallisce già da sé se cargo/rustc sono in volo (la misura NON
parte durante un job CI). Le ricette future possono creare il lock per
proteggere l'intera finestra multi-gamba.

**Dichiarato**: corpus-gate.sh ha i riferimenti congelati su path cablato del
repo canonico — la CI di un commit che cambia i congelati li giudica con quelli
del working repo, non del checkout. La fixture-chain e i gate ORM/http-kernel
NON sono in CI v1 (dipendono da binari pinnati/ambienti: restano in promozione).

Install (già eseguito; per reinstallare):
```
cp ci/post-receive "/Volumes/Extreme Pro/Claude/phpr-ci/repo.git/hooks/" && chmod +x "/Volumes/Extreme Pro/Claude/phpr-ci/repo.git/hooks/post-receive"
git remote set-url --add --push origin https://github.com/francescotinti/php-rust.git
git remote set-url --add --push origin "/Volumes/Extreme Pro/Claude/phpr-ci/repo.git"
cp ci/com.phpr.ci.plist ~/Library/LaunchAgents/ && launchctl bootstrap "gui/$(id -u)" ~/Library/LaunchAgents/com.phpr.ci.plist
```
Rollback: `launchctl bootout gui/$(id -u)/com.phpr.ci` + `git remote set-url --delete --push origin ".*phpr-ci.*"`.
