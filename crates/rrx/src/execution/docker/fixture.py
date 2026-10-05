#!/usr/bin/python3
"""Account-free Docker protocol fixture; never contacts a Docker daemon."""
import json
import os
from pathlib import Path
import sqlite3
import sys
import time

path = Path(__file__).with_suffix('.json')
state = json.loads(path.read_text())
db = sqlite3.connect('file:' + state['database'] + '?mode=ro', uri=True)
pending = [json.loads(row[0]) for row in db.execute(
    "SELECT body FROM managed_effects WHERE unit_id=? AND state='pending'",
    (state['unit'],))]
cookie = os.environ.get('RRX_PROCESS_COOKIE')
assert os.environ.get('DOCKER_API_VERSION') == '1.48'
if cookie == state['cookie']:
    assert any(effect['kind'] == 'docker_probe' for effect in pending)
    opened = db.execute('SELECT native_effects_open FROM execution_units WHERE id=?',
                        (state['unit'],)).fetchone()
    assert opened == (1,)
else:
    assert any(effect['id'] == cookie and effect['kind'] == 'docker_cleanup'
               for effect in pending)
    opened = db.execute('SELECT native_effects_open,result_finalization_open '
                        'FROM execution_units WHERE id=?', (state['unit'],)).fetchone()
    assert opened == (0, 0)
db.close()
arguments = sys.argv[1:]
assert all('.Env' not in argument for argument in arguments)
if arguments[:1] == ['--context']:
    assert arguments[1] == 'default'
    arguments = arguments[2:]
state['calls'].append(arguments)

def save():
    replacement = path.with_suffix('.next')
    replacement.write_text(json.dumps(state))
    replacement.replace(path)

def emit(values):
    save()
    print('\n'.join(json.dumps(value) for value in values))
    sys.exit(0)

if arguments[:2] == ['context', 'show']:
    save()
    print('default')
elif arguments[:2] == ['context', 'inspect']:
    save()
    print(state.get('transport', 'unix'))
elif arguments[0] == 'version':
    emit([state.get('client', '28.0.0'), state.get('server', '28.0.0'), '1.48'])
elif arguments[0] == 'info':
    emit([state.get('engine', 'fixture-engine')])
elif arguments[1] == 'ls':
    kind = arguments[0]
    selected = state.get(kind + 's', {})
    filters = [argument[6:].split('=', 1) for argument in arguments
               if argument.startswith('label=')]
    assert len(filters) == 4
    save()
    for identity, value in selected.items():
        if state.get('ignore_filter') or all(value['labels'].get(k) == v for k, v in filters):
            print(identity)
elif arguments[1] == 'inspect':
    kind, identity = arguments[0], arguments[-1]
    value = state[kind + 's'].get(identity)
    if value is None:
        save()
        sys.exit(1)
    if value.get('stopped') and state.get('delay_after_kill'):
        save()
        time.sleep(8)
    keys = ['org.rururunx.runtime', 'org.rururunx.project',
            'org.rururunx.task', 'org.rururunx.unit']
    if kind == 'container':
        emit([identity, '/' + value['name'], value['running']]
             + [value['labels'].get(key) for key in keys] + [value['operation']])
    emit([value['labels'].get(key) for key in keys])
elif arguments[:2] == ['container', 'kill']:
    assert len(arguments) == 3
    state['containers'][arguments[2]]['running'] = False
    state['containers'][arguments[2]]['stopped'] = True
    save()
elif arguments[:2] == ['container', 'rm']:
    assert len(arguments) == 3  # no force/volume/global prune
    assert not state['containers'][arguments[2]]['running']
    del state['containers'][arguments[2]]
    save()
else:
    raise AssertionError('unsupported fixture command')
