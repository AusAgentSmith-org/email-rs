#!/usr/bin/env python3
"""
Generate a WiX v4/v5 fragment for the Vite frontend static files.
Usage: harvest-static.py <staging-dir>
  staging-dir: directory containing a 'static/' subdirectory

The fragment defines ComponentGroup "StaticFiles" with Directory="STATICDIR".
Files in subdirectories use the Subdirectory attribute (WiX v4+ feature).
"""
import os, sys, hashlib, re

def wix_id(path: str) -> str:
    h = hashlib.md5(path.encode()).hexdigest()[:8]
    base = os.path.splitext(os.path.basename(path))[0] or "x"
    n = re.sub(r"[^A-Za-z0-9]", "_", base)[:24]
    return f"{n}_{h}"


def main() -> None:
    if len(sys.argv) < 2:
        print("Usage: harvest-static.py <staging-dir>", file=sys.stderr)
        sys.exit(1)

    staging = sys.argv[1]
    static = os.path.join(staging, "static")

    if not os.path.isdir(static):
        print(f"Error: {static} is not a directory", file=sys.stderr)
        sys.exit(1)

    lines = [
        '<?xml version="1.0" encoding="utf-8"?>',
        '<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">',
        "  <Fragment>",
        '    <ComponentGroup Id="StaticFiles">',
    ]

    for root, dirs, files in os.walk(static):
        dirs.sort()
        for fname in sorted(files):
            fpath = os.path.join(root, fname)
            rel_from_static = os.path.relpath(fpath, static)
            rel_from_staging = os.path.relpath(fpath, staging).replace(os.sep, "/")
            subdir = os.path.dirname(rel_from_static).replace(os.sep, "/")

            cid = "CmpS_" + wix_id(rel_from_static)
            fid = "FilS_" + wix_id(rel_from_static + "::f")
            src = f"$(var.Staging)/{rel_from_staging}"

            subdir_attr = f' Subdirectory="{subdir}"' if subdir and subdir != "." else ""
            lines.append(f'      <Component Id="{cid}" Directory="STATICDIR" Guid="*"{subdir_attr}>')
            lines.append(f'        <File Id="{fid}" Source="{src}" KeyPath="yes" />')
            lines.append(f"      </Component>")

    lines += [
        "    </ComponentGroup>",
        "  </Fragment>",
        "</Wix>",
    ]

    print("\n".join(lines))


if __name__ == "__main__":
    main()
