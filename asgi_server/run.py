import asyncio
import importlib
import logging
import signal
from typing import Awaitable, Callable, Union

from . import asgi_server


LOG_FORMAT = "[%(asctime)-15s] %(levelname)s %(name)s %(message)s"

ASGIReceiveCallable = Callable[[], Awaitable[dict]]
ASGISendCallable = Callable[[dict], Awaitable[None]]
ASGI3Application = Callable[
    [dict, ASGIReceiveCallable, ASGISendCallable], Awaitable[None]
]


def import_app(name: str) -> ASGI3Application:
    try:
        module_name, app_name = name.rsplit(":", 1)
    except:
        raise ValueError(
            f"Invalid app callable {repr(name)}: must be in the form 'module:app'."
        )

    return getattr(importlib.import_module(module_name), app_name)


def init_logging(log_level):
    logging.basicConfig(format=LOG_FORMAT)
    logging.getLogger().setLevel(log_level)


def run(
    app: Union[str, ASGI3Application],
    host="127.0.0.1",
    port=5000,
    log_level=logging.INFO,
) -> None:
    if isinstance(app, str):
        app = import_app(app)

    init_logging(log_level)

    settings = asgi_server.Settings()
    settings.host = host
    settings.port = port

    task = asgi_server.start_server(app, settings)
    loop = asyncio.get_event_loop()

    def stop_server():
        # fixme asgi lifecycle protocol
        loop.stop()

    loop.add_signal_handler(signal.SIGTERM, stop_server)

    try:
        loop.run_until_complete(task)
    except KeyboardInterrupt:
        stop_server()
