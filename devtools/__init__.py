import typer

from .build_docs import build_docs
from .serve_docs import serve_docs

devtools = typer.Typer(name="devtools")
devtools.command()(build_docs)
devtools.command()(serve_docs)
