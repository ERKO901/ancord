#!/usr/bin/env python3
"""Náhrada tunelu pro lokální běh: /{hostname}/cesta -> 127.0.0.1:{port nodu}/cesta.
Použití: local-forward.py PORT hostname=port [hostname=port ...]"""
import socket, sys, threading, urllib.request, urllib.error
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

PORTS = dict(a.split("=") for a in sys.argv[2:])

class H(BaseHTTPRequestHandler):
    def _do(self):
        host, _, tail = self.path.lstrip("/").partition("/")
        port = PORTS.get(host)
        if not port:
            self.send_error(404); return
        if self.headers.get("upgrade", "").lower() == "websocket":
            return self._tunnel(int(port), "/" + tail)
        body = self.rfile.read(int(self.headers.get("content-length") or 0)) or None
        req = urllib.request.Request(f"http://127.0.0.1:{port}/{tail}", data=body, method=self.command)
        if self.headers.get("authorization"):
            req.add_header("authorization", self.headers["authorization"])
        try:
            r = urllib.request.urlopen(req, timeout=10); code, data = r.status, r.read()
        except urllib.error.HTTPError as e:
            code, data = e.code, e.read()
        except Exception:
            code, data = 502, b""
        self.send_response(code); self.send_header("content-length", str(len(data))); self.end_headers(); self.wfile.write(data)
    def _tunnel(self, port, path):
        up = socket.create_connection(("127.0.0.1", port))
        head = f"{self.command} {path} HTTP/1.1\r\n" + "".join(f"{k}: {v}\r\n" for k, v in self.headers.items()) + "\r\n"
        up.sendall(head.encode())
        def pipe(a, b):
            try:
                while (d := a.recv(65536)):
                    b.sendall(d)
            except OSError:
                pass
            for x in (a, b):
                try: x.shutdown(socket.SHUT_RDWR)
                except OSError: pass
        t = threading.Thread(target=pipe, args=(up, self.connection), daemon=True); t.start()
        pipe(self.connection, up); t.join()
        self.close_connection = True
    do_GET = do_POST = do_PUT = do_DELETE = _do
    def log_message(self, *a): pass

ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
