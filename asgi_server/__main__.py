import asyncio

from starlette.applications import Starlette
from starlette.responses import JSONResponse
from starlette.routing import Route
import uvicorn

from . import asgi_server


async def homepage(request):
    return JSONResponse({"hello": "world"})


app = Starlette(
    debug=True,
    routes=[
        Route("/", homepage),
    ],
)


async def app_callable(scope, receive, send):
    print(f"Scope received: {scope}")
    print(f"First event: {await receive()}")
    await send(
        {
            "type": "http.response.start",
            "status": 200,
            "headers": [(b"Content-Type", b"application/json")],
        }
    )
    print("SENT EVENT")
    await send(
        {"type": "http.response.body", "body": b'{"foo": 18}', "more_body": False}
    )
    print("SENT RESPONSE from py")


async def main():
    settings = asgi_server.Settings()
    await asgi_server.start_server(app, settings)


if __name__ == "__main__":
    # uvicorn.run(app_callable, host="127.0.0.1", port=3000)
    asyncio.get_event_loop().run_until_complete(main())
