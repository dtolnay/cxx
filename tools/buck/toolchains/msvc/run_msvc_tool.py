#!/usr/bin/env python3

import json
import os
import subprocess
import sys
from typing import List, NamedTuple


class Tool(NamedTuple):
    exe: str
    libs: List[str]
    paths: List[str]
    includes: List[str]


def add_env(env, key, entries):
    entries = ";".join(entries)
    if key in env:
        env[key] = entries + ";" + env[key]
    else:
        env[key] = entries


def main():
    tool_json, arguments = sys.argv[1], sys.argv[2:]
    with open(tool_json, encoding="utf-8") as f:
        tool = Tool(**json.load(f))

    env = os.environ.copy()
    add_env(env, "LIB", tool.libs)
    add_env(env, "PATH", tool.paths)
    add_env(env, "INCLUDE", tool.includes)

    completed_process = subprocess.run([tool.exe, *arguments], env=env)
    sys.exit(completed_process.returncode)


if __name__ == "__main__":
    main()
