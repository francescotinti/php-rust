#!/usr/bin/env python3
"""park8080.py 127.0.0.1:8080 — S-188: holds the measurement port BETWEEN boots.

Why: a browser tab left on http://127.0.0.1:8080/wp-admin reloads its page (106
static assets, ~every 45 s) whenever the WordPress server is reachable — it hit
4/12 boots of t1 and most P0 boots of t2 (3-minute boots). While the port is
parked, the tab gets a blank 200 page with no scripts, no redirect and no
refresh, so it stops reloading; the harness kills the parker right before each
real boot (its argv carries "127.0.0.1:8080" for the harness's pkill pattern).
Every request is logged to stderr so the parked window keeps evidence of the
foreign client. Generated with AI assistance (Claude Fable 5).
"""
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

BODY = b"<!doctype html><title>parked</title><p>s188: porta in parcheggio fra due boot.</p>"


class Park(BaseHTTPRequestHandler):
    server_version = "s188-park/1"

    def _answer(self):
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=UTF-8")
        self.send_header("Content-Length", str(len(BODY)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("Connection", "close")
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(BODY)

    do_GET = do_POST = do_HEAD = do_PUT = do_DELETE = do_OPTIONS = _answer

    def log_message(self, fmt, *args):  # one line per foreign request, to stderr
        sys.stderr.write("%s park %s\n" % (self.log_date_time_string(), fmt % args))
        sys.stderr.flush()


def main():
    host, port = (sys.argv[1] if len(sys.argv) > 1 else "127.0.0.1:8080").rsplit(":", 1)
    HTTPServer.allow_reuse_address = True
    srv = HTTPServer((host, int(port)), Park)
    try:
        srv.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
