// Transport-independent checks against the actual browser's reported command line.
export function chromeArguments(profile, headed, extensionOrigin) {
  return ['--remote-debugging-port=0', '--remote-debugging-address=127.0.0.1', `--user-data-dir=${profile}`,
    '--no-first-run', '--no-default-browser-check', '--disable-background-networking', '--enable-unsafe-extension-debugging', '--enable-automation',
    ...(extensionOrigin ? [`--remote-allow-origins=${extensionOrigin}`] : []),
    ...(headed ? [] : ['--headless=new']), 'about:blank'];
}
export function checkChromeArguments(arguments_, headed, extensionOrigin) {
  const origins = arguments_.filter(argument => argument.startsWith('--remote-allow-origins='));
  if (extensionOrigin ? origins.length !== 1 || origins[0] !== `--remote-allow-origins=${extensionOrigin}` : origins.length !== 0) throw new Error('protocol');
  if (arguments_.some(argument => ['--no-sandbox', '--disable-web-security', '--ignore-certificate-errors'].includes(argument))) throw new Error('protocol');
  const headless = arguments_.some(argument => argument.startsWith('--headless'));
  if (headless === headed) throw new Error('protocol');
  return { session: 'browser_mode', headed: !headless };
}
