<?php
// A-DS8 support unit: pinned in the request's RetainSet on require (S-78.1.5:
// the pin dies with the request; cross-request reuse = thread-local unit
// cache). The static must restart per request regardless.
function lib_tick() { static $n = 0; return ++$n; }
