<?php
// A-SK1 (Council WP-80): overlap-observable fixture — each request holds a
// worker for a fixed 400ms, so wall-clock over a volley discriminates
// concurrent service (W workers) from sequential drain (one worker).
usleep(400000);
echo "slept\n";
