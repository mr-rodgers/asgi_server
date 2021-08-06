import subprocess


def build_docs():
    subprocess.call(["cargo", "docs", "--no-deps"])
