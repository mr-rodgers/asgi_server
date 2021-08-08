This is a proof-of-concept asgi 3.0 server implemented in Rust on [Hyper](https://github.com/hyperium/hyper).

## Building

Since this a mixed project, there are two sets of dependencies: Rust and Python. 

1. [Install the Rust toolchain](https://www.rust-lang.org/tools/install), which is required to build the Rust code into a Python extension module that is compatible with your machine. On Linux, use rustup, the official installer: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
2. Install the development headers for Python. On Ubuntu, for example: `sudo apt install python3-dev python-dev`.
3. [Install pdm](https://pdm.fming.dev/#installation), which is used to bundle Python dependencies: `curl -sSL https://raw.githubusercontent.com/pdm-project/pdm/main/install-pdm.py | python -`.
4. Use pdm to install the dependencies, and install the project as editable: `pdm install`
5. Use the dependency binary to build and install the extension module: `pdm run maturin develop`


Installation: Clone this repository and run `pdm install`.
To run: `pdm run asgi-server pkg.module:app`, or `pdm run asgi-server --help` for more options.

## Project details

The project provides ASGI 3.0 bindings for Hyper, and currently supports the following ASGI protocols:

- [x] HTTP Webserver Protocol 2.0
- [ ] Websocket Protocol
- [ ] Lifecycle Protocol

Rust code for the Hyper bindings is found in the `src` folder, while the Python wrapper is found in `asgi_server`.

The Rust code uses pyo3 as the compatibility layer into Python, in the form of an extension module.
In this extension module are two exports:

```python
class Settings:
    host: str
    port: int


def start_server(app, settings: Settings) -> asyncio.Future:
    ...
```

Note: do not run the server with these exports unless you have configured the logging module
first, otherwise this will trigger an application crash. Use `asgi_server.run(...)` instead,
if you prefer to just start the server without configuring all of this yourself. 

### Running the server programattically

The server has a Python interface which can be used to run it, rather than using the command line:

```python
import asgi_server

async def app(scope, receive, send):
    ...

if __name__ == "__main__":
    asgi_server.run(app, host="127.0.0.1", port=5000, log_level="INFO")
```

### Hacking

This project is packaged with [pdm](https://pdm.fming.dev/), which by default installs
packages into a `__pypackages__` folder in the project directory, rather than Python's
*site-packages* folder. Since *pypackages* isn't on the search path, this will likely cause
issues with your IDE and its bundled tools. One workaround is to configure pdm to install
into a virtual-env, by setting the environment variable `PDM_USE_VENV=True` (this has been 
known not to work correctly with conda envs). Alternatively, follow the 
[tips](https://pdm.fming.dev/#enable-pep-582-globally) 
[provided](https://pdm.fming.dev/#use-with-ide) by the pdm project.

If you use vscode, just use the bundled devcontainer, which will launch you into a ready-to-hack
environment.

#### Editing Rust Code

To play with your changes after you edit Rust code, you need to build the extension module, and
then copy it to the Python package's search path as `asgi_server.asgi_server`.

To accomplish this, run the command: `maturin develop` 
