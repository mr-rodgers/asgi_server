import os
import subprocess
from pathlib import Path

import typer

DOCS_HOST = os.environ.get("ASGI_SERVER_DEV_DOCS_HOST", "localhost")
DOCS_PORT = os.environ.get("ASGI_SERVER_DEV_DOCS_PORT", 3080)


def serve_docs():
    target_dir = Path("target") / "doc" / "asgi_server"

    if not target_dir.is_dir():
        subprocess.call(["devtools", "build-docs"])

    subprocess.call(
        [
            "python",
            "-m",
            "http.server",
            DOCS_PORT,
            "--directory",
            str(target_dir.parent),
            "--bind",
            DOCS_HOST,
        ]
    )
