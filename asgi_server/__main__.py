from starlette.applications import Starlette
from starlette.responses import JSONResponse
from starlette.routing import Route

from .run import run


async def homepage(request):
    return JSONResponse({"hello": "world"})


app = Starlette(
    debug=True,
    routes=[
        Route("/", homepage),
    ],
)


if __name__ == "__main__":
    # uvicorn.run(app_callable, host="127.0.0.1", port=3000)
    run(app)
