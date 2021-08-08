import click
from .run import run


@click.command()
@click.argument("app")
@click.option(
    "-h", "--host", default="localhost", help="Host interface to listen for connections"
)
@click.option("-p", "--port", default=5000, type=int)
@click.option(
    "--log-level",
    default="INFO",
    type=click.Choice(["WARN", "DEBUG", "ERROR", "INFO"], case_sensitive=False),
    help="Control the verbosity of the server.",
)
def main(app, host, port, log_level):
    """Server an ASGI 3 callable

    APP argument should be in format: <pkg>.<module>:<app>"""
    run(app, host=host, port=port, log_level=log_level.upper())


if __name__ == "__main__":
    main()
