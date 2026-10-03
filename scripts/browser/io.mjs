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

// No shell, background leader, detached child, or global process lookup.
export function invoke(command, input, timeout, limit = 262144, ownGroup = false, acceptErrorJson = false) {
  return new Promise((resolve, reject) => {
    const child = spawn(command[0], command.slice(1), { stdio: ['pipe', 'pipe', 'ignore'], detached: ownGroup });
    let chunks = [], size = 0, failure, killTimer;
    const signal = value => { try { if (ownGroup) process.kill(-child.pid, value); else child.kill(value); } catch {} };
    const stop = (reason) => { failure ??= reason; signal('SIGTERM'); killTimer ??= setTimeout(() => signal('SIGKILL'), 1000); };
    const timer = setTimeout(() => stop('timeout'), timeout);
    child.on('error', () => { clearTimeout(timer); reject(new Error('unavailable')); });
    child.stdin.on('error', () => stop('protocol'));
    child.stdout.on('data', chunk => {
      size += chunk.length;
      if (size > limit) stop('output_limit'); else chunks.push(chunk);
    });
    child.on('close', code => {
      clearTimeout(timer);
      clearTimeout(killTimer);
      if (failure || code !== 0 && !acceptErrorJson) reject(new Error(failure ?? 'operation'));
      else { try { resolve(JSON.parse(Buffer.concat(chunks).toString('utf8'))); } catch { reject(new Error('protocol')); } }
    });
    const bytes = JSON.stringify(input);
    if (Buffer.byteLength(bytes) > 65536) stop('output_limit');
    else child.stdin.end(bytes);
  });
}
