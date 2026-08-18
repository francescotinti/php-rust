#!/bin/bash
# s154-lancio-promo.sh — wrapper argv-neutro: esporta PROMO_SP e lancia la
# catena di promozione L-CE1 col candidato dichiarato.
set -u
export PROMO_SP=/private/tmp/phpr-s154-promo-sp
exec /bin/bash "/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp154-harness/s154-promozione.sh" e634d95cd8bfd475
