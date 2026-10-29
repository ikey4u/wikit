#!/usr/bin/env python3
"""Exercise dictionary creation, HTTP lookup, one-shot TUI and a real PTY session."""
import json
import os
from pathlib import Path
import select
import subprocess
import tempfile
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "target/debug"


def main():
    with tempfile.TemporaryDirectory(prefix="wikit-smoke-") as tmp:
        directory = Path(tmp)
        source = directory / "sample.txt"
        dictionary = directory / "sample.wikit"
        source.write_text("hello\n<p>A greeting.</p>\n</>\n你好\n<p>中文释义</p>\n</>\n")
        (directory / "sample.toml").write_text("name='Smoke dictionary'\ncss=['a.css','b.css']\njs='a.js, b.js'\n")
        for filename, text in {"a.css": "p{color:red}", "b.css": "p{color:blue}", "a.js": "var x=1", "b.js": "x=2"}.items():
            (directory / filename).write_text(text)
        subprocess.run([str(BIN / "wikit"), "dict", "--create", str(source), "--output", str(dictionary)], check=True, capture_output=True)
        output = subprocess.check_output([str(BIN / "wikit-tui"), "-d", str(dictionary), "hello"], text=True)
        assert "A greeting." in output and "\x1b" not in output
        server = subprocess.Popen([str(BIN / "wikit-server"), "-d", str(dictionary), "--listen", "127.0.0.1:0"], stderr=subprocess.PIPE, text=True)
        try:
            ready, _, _ = select.select([server.stderr], [], [], 10)
            assert ready, "Server startup timed out"
            line = server.stderr.readline()
            base = line.strip().split()[-1]
            assert base.startswith("http://127.0.0.1:")
            with urllib.request.urlopen(base + "/wikit/list", timeout=5) as response:
                catalog = json.load(response)
            assert catalog[0]["name"] == "Smoke dictionary"
            output = subprocess.check_output([str(BIN / "wikit-tui"), "--server", base, "你好", "--json"], text=True, timeout=10)
            assert json.loads(output)[0][0] == "你好"
            print("PASS: CLI creation, local and remote one-shot UTF-8 lookup")
        finally:
            server.terminate()
            server.wait(timeout=5)

        if os.name == "posix":
            import fcntl
            import pty
            import struct
            import termios
            master, slave = pty.openpty()
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
            child = subprocess.Popen([str(BIN / "wikit-tui"), "-d", str(dictionary)], stdin=slave, stdout=slave, stderr=slave, env={**os.environ, "TERM": "xterm-256color"})
            os.close(slave)
            captured = bytearray()
            try:
                deadline = time.monotonic() + 10
                entered = False
                while time.monotonic() < deadline:
                    ready, _, _ = select.select([master], [], [], 0.2)
                    if ready:
                        captured.extend(os.read(master, 65536))
                    if not entered and b"Wikit" in captured:
                        os.write(master, b"hello\r")
                        entered = True
                    # Ratatui may skip unchanged spaces using cursor-position commands.
                    if b"greeting." in captured:
                        break
                assert b"greeting." in captured, "Interactive lookup did not render its definition"
                os.write(master, b"\x03")
                assert child.wait(timeout=5) == 0
                while select.select([master], [], [], 0.2)[0]:
                    try:
                        captured.extend(os.read(master, 65536))
                    except OSError:
                        break
                assert b"\x1b[?1049l" in captured, "Terminal alternate screen was not restored"
                assert b"\x1b[?25h" in captured, "Terminal cursor was not restored"
                print("PASS: real PTY startup, lookup, Ctrl+C and terminal restoration")
            finally:
                if child.poll() is None:
                    child.kill()
                    child.wait()
                os.close(master)


if __name__ == "__main__":
    main()
