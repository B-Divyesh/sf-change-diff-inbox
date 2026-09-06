import { createServer } from 'node:http';

// The claim suite points the Rust HTTP client at this process through
// HTTP_PROXY. The watched URL remains a normal public URL, while the fixture
// keeps the manual-check outcome deterministic and does not reach the network.
const port = Number(process.env.MANUAL_CHECK_FIXTURE_PORT || 4175);

const server = createServer((request, response) => {
  const target = new URL(request.url || '/', 'http://fixture.invalid');
  if (target.pathname === '/robots.txt') {
    response.writeHead(200, {'content-type': 'text/plain; charset=utf-8'});
    response.end('User-agent: *\nAllow: /\n');
    return;
  }
  if (target.pathname === '/manual-check-fixture') {
    response.writeHead(200, {'content-type': 'text/html; charset=utf-8'});
    response.end('<!doctype html><main><h1>Fixture API limits</h1><p>Daily checks capture this known baseline.</p></main>');
    return;
  }
  response.writeHead(404, {'content-type': 'text/plain; charset=utf-8'});
  response.end('Fixture route not found');
});

server.listen(port, '127.0.0.1');

function close() {
  server.close(() => process.exit(0));
}

process.on('SIGTERM', close);
process.on('SIGINT', close);
