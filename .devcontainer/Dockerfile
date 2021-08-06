# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.187.0/containers/python-3/.devcontainer/base.Dockerfile

# [Choice] Python version: 3, 3.9, 3.8, 3.7, 3.6
ARG VARIANT="3.9"
FROM mcr.microsoft.com/vscode/devcontainers/python:0-${VARIANT}

USER vscode

ARG VIRTUALENV_LOC="/home/vscode/asgi-server.venv"

RUN curl -sSL https://raw.githubusercontent.com/pdm-project/pdm/main/install-pdm.py | python - \
    && python -m venv ${VIRTUALENV_LOC} \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && git clone https://github.com/magicmonty/bash-git-prompt.git ~/.bash-git-prompt --depth=1 \
    && echo "\nGIT_PROMPT_ONLY_IN_REPO=1\nsource $HOME/.bash-git-prompt/gitprompt.sh\nsource ${VIRTUALENV_LOC}/bin/activate" >> $HOME/.bashrc

ENV PATH /home/vscode/.local/bin:$PATH
ENV PDM_USE_VENV True