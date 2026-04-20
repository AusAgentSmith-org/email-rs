#!/usr/bin/env python3
"""
Generate a WiX v5 fragment for the Vite frontend static files.
Usage: harvest-static.py <staging-dir>

Produces two fragments:
  1. DirectoryRef under STATICDIR with nested Directory declarations
  2. ComponentGroup "StaticFiles" referencing those directories
"""
import os, sys, hashlib, re

def wix_id(path: str) -> str:
    h = hashlib.md5(path.encode()).hexdigest()[:8]
    base = os.path.splitext(os.path.basename(path))[0] or "x"
    n = re.sub(r"[^A-Za-z0-9]", "_", base)[:24]
    return f"{n}_{h}"


def dir_id(rel: str) -> str:
    """Stable WiX ID for a directory relative to static/."""
    if not rel or rel == ".":
        return "STATICDIR"
    h = hashlib.md5(rel.encode()).hexdigest()[:8]
    n = re.sub(r"[^A-Za-z0-9]", "_", rel.replace(os.sep, "_"))[:24]
    return f"DirS_{n}_{h}"


def emit_dirs(children_map: dict, parent_rel: str, indent: int) -> list:
    """Recursively emit <Directory> elements for children of parent_rel."""
    lines = []
    pad = "  " * indent
    for rel in sorted(children_map.get(parent_rel, [])):
        did = dir_id(rel)
        dname = os.path.basename(rel)
        sub_children = children_map.get(rel, [])
        if sub_children:
            lines.append(f'{pad}<Directory Id="{did}" Name="{dname}">')
            lines.extend(emit_dirs(children_map, rel, indent + 1))
            lines.append(f'{pad}</Directory>')
        else:
            lines.append(f'{pad}<Directory Id="{did}" Name="{dname}" />')
    return lines


def main() -> None:
    if len(sys.argv) < 2:
        print("Usage: harvest-static.py <staging-dir>", file=sys.stderr)
        sys.exit(1)

    staging = sys.argv[1]
    static = os.path.join(staging, "static")

    if not os.path.isdir(static):
        print(f"Error: {static} is not a directory", file=sys.stderr)
        sys.exit(1)

    # Build directory tree (rel path → list of child rel paths)
    children_map: dict = {}
    for root, dirs, _ in os.walk(static):
        dirs.sort()
        parent_rel = os.path.relpath(root, static)
        if parent_rel == ".":
            parent_rel = ""
        for d in dirs:
            drel = os.path.relpath(os.path.join(root, d), static)
            children_map.setdefault(parent_rel, []).append(drel)

    # Collect files
    all_files = []
    for root, dirs, files in os.walk(static):
        dirs.sort()
        for fname in sorted(files):
            fpath = os.path.join(root, fname)
            rel_static = os.path.relpath(fpath, static)
            rel_staging = os.path.relpath(fpath, staging).replace(os.sep, "/")
            dir_rel = os.path.relpath(root, static)
            if dir_rel == ".":
                dir_rel = ""
            did = dir_id(dir_rel)
            cid = "CmpS_" + wix_id(rel_static)
            fid = "FilS_" + wix_id(rel_static + "::f")
            all_files.append((cid, fid, did, rel_staging))

    out = [
        '<?xml version="1.0" encoding="utf-8"?>',
        '<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">',
    ]

    # Fragment 1: subdirectory declarations under STATICDIR
    if children_map.get(""):
        out.append("  <Fragment>")
        out.append('    <DirectoryRef Id="STATICDIR">')
        out.extend(emit_dirs(children_map, "", 3))
        out.append("    </DirectoryRef>")
        out.append("  </Fragment>")

    # Fragment 2: components
    out.append("  <Fragment>")
    out.append('    <ComponentGroup Id="StaticFiles">')
    for cid, fid, did, src in all_files:
        out.append(f'      <Component Id="{cid}" Directory="{did}" Guid="*">')
        out.append(f'        <File Id="{fid}" Source="$(var.Staging)/{src}" KeyPath="yes" />')
        out.append(f"      </Component>")
    out.append("    </ComponentGroup>")
    out.append("  </Fragment>")
    out.append("</Wix>")

    print("\n".join(out))


if __name__ == "__main__":
    main()
