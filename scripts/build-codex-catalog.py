"""Build the M0 Codex schema contract catalog from an openai/codex tag checkout (design §8b).

The input is the precomputed `--experimental` JSON schema export of a codex tag:
  codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst
It is read as data only; nothing in the checkout is executed.

Usage:
  python3 -I scripts/build-codex-catalog.py <checkout> <methods> <responses> [--expect <subtree-digest>]

<methods> and <responses> are comma-separated. The catalog JSON is written to stdout; the
subtree digest goes to stderr. With --expect the run fails unless the digest matches exactly
(the reproduction check of the previous basis).

Rule:
- sources: ClientNotification, ServerNotification and ServerRequest (whole roots),
  ClientRequest_used (only the branches of <methods>, in ClientRequest.json order), and each
  response root in <responses>.
- only schema positions are walked: the root, the members of the schema maps (properties,
  patternProperties, definitions, $defs, dependentSchemas), the single subschemas
  (additionalProperties, additionalItems, items, contains, propertyNames, not, if, then, else,
  unevaluatedProperties, unevaluatedItems) and the subschema lists (allOf, anyOf, oneOf,
  prefixItems, array-form items). Every other keyword value (default, enum, const, required,
  type, format, numeric/string/array bounds) is a value position and is kept verbatim, including
  any nested `title`, `description`, `examples` or `$ref` keys inside it.
- annotation keywords (description, title, examples, $schema) are removed only as keywords of a
  schema position; each root's `definitions` is removed.
- a keyword outside the schema and value sets above fails the run, so a new keyword needs an
  explicit rule change (and the reproduction check) instead of a silent default.
- local_definition_digests is the transitive local $ref closure of each root, following `$ref`
  only where it is a keyword of a schema position, keyed by definition name, valued by the
  SHA-256 of the compact sorted-key JSON of the stripped definition; shared_definitions maps each
  digest to that definition.
- subtree digest: SHA-256 of json.dumps(catalog, sort_keys=True, separators=(",", ":"),
  ensure_ascii=False) encoded as UTF-8.
"""
import hashlib
import json
import sys
from compression import zstd
from pathlib import Path

EXPORT = 'codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst'
STRIP = {'description', 'title', 'examples', '$schema'}
SCHEMA_MAPS = {'properties', 'patternProperties', 'definitions', '$defs', 'dependentSchemas'}
SCHEMA_ONE = {'additionalProperties', 'additionalItems', 'contains', 'propertyNames', 'not', 'if',
              'then', 'else', 'unevaluatedProperties', 'unevaluatedItems'}
SCHEMA_LIST = {'allOf', 'anyOf', 'oneOf', 'prefixItems'}
VALUES = {'$ref', 'type', 'enum', 'const', 'default', 'required', 'format', 'minimum', 'maximum',
          'exclusiveMinimum', 'exclusiveMaximum', 'multipleOf', 'minLength', 'maxLength', 'pattern',
          'minItems', 'maxItems', 'uniqueItems', 'minProperties', 'maxProperties', 'readOnly',
          'writeOnly', 'deprecated'}
KIND = ('offline public observation only; source-local refs resolve through '
        'local_definition_digests into shared_definitions; not a runtime validator')


def subschemas(k, x):
    """(key, subschema) pairs of a schema-position keyword; None for a value-position keyword."""
    if k in SCHEMA_MAPS:
        return [(n, s) for n, s in x.items()]
    if k in SCHEMA_ONE or (k == 'items' and not isinstance(x, list)):
        return [(None, x)]
    if k in SCHEMA_LIST or k == 'items':
        return [(i, s) for i, s in enumerate(x)]
    if k in VALUES:
        return None
    sys.exit(f'unknown schema keyword {k}')


def strip(s):
    if not isinstance(s, dict):
        return s  # boolean schema
    out = {}
    for k, x in s.items():
        if k in STRIP:
            continue
        sub = subschemas(k, x)
        if sub is None:
            out[k] = x
        elif k in SCHEMA_MAPS:
            out[k] = {n: strip(v) for n, v in sub}
        elif isinstance(x, list):
            out[k] = [strip(v) for _, v in sub]
        else:
            out[k] = strip(x)
    return out


def canon(v):
    return json.dumps(v, sort_keys=True, separators=(',', ':'), ensure_ascii=False)


def local_refs(s, acc):
    if not isinstance(s, dict):
        return acc
    r = s.get('$ref')
    if r is not None:
        if not isinstance(r, str) or not r.startswith('#/definitions/'):
            sys.exit(f'non-local ref {r}')
        acc.add(r[len('#/definitions/'):])
    for k, x in s.items():
        for _, v in subschemas(k, x) or ():
            local_refs(v, acc)
    return acc


def build(files, methods, responses):
    shared = {}

    def source(name, select=None):
        full = files[name]
        defs = {k: strip(v) for k, v in (full.get('definitions') or {}).items()}
        root = strip(full)
        root.pop('definitions', None)
        if select:
            root = select(root)
        seen, todo = set(), sorted(local_refs(root, set()))
        while todo:
            n = todo.pop()
            if n in seen:
                continue
            if n not in defs:
                sys.exit(f'{name}: unresolved #/definitions/{n}')
            seen.add(n)
            todo.extend(sorted(local_refs(defs[n], set()) - seen))
        local = {}
        for n in sorted(seen):
            h = hashlib.sha256(canon(defs[n]).encode()).hexdigest()
            shared[h] = defs[n]
            local[n] = h
        return {'schema': root, 'local_definition_digests': local}

    def used(root):
        keep = [o for o in root['oneOf'] if o['properties']['method']['enum'][0] in methods]
        missing = set(methods) - {o['properties']['method']['enum'][0] for o in keep}
        if missing:
            sys.exit(f'ClientRequest lacks {sorted(missing)}')
        return {**root, 'oneOf': keep}

    def find(name):
        key = next((k for k in files if k == name or k.endswith('/' + name)), None)
        if key is None:
            sys.exit(f'export lacks {name}')
        return key

    sources = {
        'ClientNotification': source(find('ClientNotification.json')),
        'ServerNotification': source(find('ServerNotification.json')),
        'ServerRequest': source(find('ServerRequest.json')),
        'ClientRequest_used': source(find('ClientRequest.json'), used),
    }
    for r in responses:
        sources[r] = source(find(r + '.json'))
    return {'kind': KIND, 'source_schemas': sources, 'shared_definitions': dict(sorted(shared.items()))}


def main():
    args = sys.argv[1:]
    expect = None
    if '--expect' in args:
        i = args.index('--expect')
        expect = args[i + 1]
        del args[i:i + 2]
    checkout, methods, responses = Path(args[0]), args[1].split(','), args[2].split(',')
    raw = (checkout / EXPORT).read_bytes()
    doc = json.loads(zstd.decompress(raw))
    files = {k: (json.loads(v) if isinstance(v, str) else v) for k, v in doc['json_schema'].items()}
    catalog = build(files, methods, responses)
    digest = hashlib.sha256(canon(catalog).encode()).hexdigest()
    print(f'export_sha256={hashlib.sha256(raw).hexdigest()} export_files={len(files)} '
          f'sources={len(catalog["source_schemas"])} shared={len(catalog["shared_definitions"])} '
          f'subtree_digest={digest}', file=sys.stderr)
    if expect is not None and digest != expect:
        sys.exit(f'reproduction check failed: {digest} != {expect}')
    json.dump(catalog, sys.stdout, indent=1, ensure_ascii=False)
    sys.stdout.write('\n')


if __name__ == '__main__':
    main()
