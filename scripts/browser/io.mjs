import { spawn } from 'node:child_process';

export async function readInput(limit = 65536) {
  const chunks = []; let size = 0;
  for await (const chunk of process.stdin) {
    size += chunk.length;
    if (size > limit) throw new Error('output_limit');
    chunks.push(chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString('utf8'));
}

// Production callbacks inherit the Rust-owned bridge group. ownGroup is for fixtures.
export function invoke(command, input, timeout, limit = 262144, ownGroup = false, acceptErrorJson = false, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command[0], command.slice(1), { stdio: ['pipe', 'pipe', 'ignore'], detached: ownGroup });
    let chunks = [], size = 0, failure, killTimer, drainTimer, exited = false, settled = false;
    const signal = value => {
      if (exited) return; // Never signal a group after its leader has been reaped by Node.
      try { if (ownGroup) process.kill(-child.pid, value); else child.kill(value); } catch {}
    };
    const stop = reason => { failure ??= reason; signal('SIGTERM');
      if (!exited) killTimer ??= setTimeout(() => signal('SIGKILL'), options.killGrace ?? 1000); };
    const timer = setTimeout(() => stop('timeout'), timeout);
    const abort = () => stop('timeout');
    const finish = code => {
      if (settled) return; settled = true;
      clearTimeout(timer); clearTimeout(killTimer); clearTimeout(drainTimer);
      options.signal?.removeEventListener('abort', abort);
      child.stdin.destroy(); child.stdout.destroy();
      if (failure && !options.acceptFailureJson || code !== 0 && !acceptErrorJson) reject(new Error(failure ?? 'operation'));
      else { try { const parsed = JSON.parse(Buffer.concat(chunks).toString('utf8'));
        if (failure) parsed.failure = failure; resolve(parsed); } catch { reject(new Error('protocol')); } }
    };
    options.signal?.addEventListener('abort', abort, { once: true });
    if (options.signal?.aborted) abort();
    child.once('spawn', () => options.onSpawn?.());
    child.on('error', () => { failure = 'unavailable'; finish(1); });
    child.stdin.on('error', () => stop('protocol'));
    child.stdout.on('data', chunk => {
      size += chunk.length;
      if (size > limit) stop('output_limit'); else chunks.push(chunk);
    });
    child.on('exit', code => {
      exited = true; clearTimeout(timer); clearTimeout(killTimer);
      // A trusted grandchild can hold stdout after the direct child exits. Bound
      // drain, preserve complete JSON if supplied, and let Rust stop the owned group.
      drainTimer = setTimeout(() => { failure ??= 'cleanup'; finish(code); }, 100);
    });
    child.on('close', finish);
    const bytes = JSON.stringify(input);
    if (Buffer.byteLength(bytes) > 65536) stop('output_limit');
    else child.stdin.end(bytes);
  });
}
