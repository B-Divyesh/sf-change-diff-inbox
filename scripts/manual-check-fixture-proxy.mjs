import { createServer } from 'node:http';

// The claim suite points the Rust HTTP client at this process through
// HTTP_PROXY. The watched URL remains a normal public URL, while the fixture
// keeps the manual-check outcome deterministic and does not reach the network.
const port = Number(process.env.MANUAL_CHECK_FIXTURE_PORT || 4175);
let requests = [];
let scriptRuns = 0;
let challengeRequests = 0;
let failedCheckRequests = 0;

function reset() {
  requests = [];
  scriptRuns = 0;
  challengeRequests = 0;
  failedCheckRequests = 0;
}

function json(response, value) {
  response.writeHead(200, {'content-type': 'application/json; charset=utf-8'});
  response.end(JSON.stringify(value));
}

const server = createServer((request, response) => {
  const target = new URL(request.url || '/', 'http://fixture.invalid');
  if (target.pathname === '/__fixture/reset') {
    reset();
    json(response, {ok:true});
    return;
  }
  if (target.pathname === '/__fixture/network-log') {
    json(response, {requests});
    return;
  }
  if (target.pathname === '/__fixture/script-status') {
    json(response, {scriptRuns});
    return;
  }
  if (target.pathname === '/__fixture/challenge-status') {
    json(response, {challengeRequests, scriptRuns});
    return;
  }
  requests.push({host:target.host, path:target.pathname, method:request.method});
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
  if (target.pathname === '/failed-check-baseline') {
    failedCheckRequests += 1;
    if (failedCheckRequests === 1) {
      response.writeHead(200, {'content-type': 'text/html; charset=utf-8'});
      response.end('<!doctype html><main>First reliable baseline for failure recovery.</main>');
      return;
    }
    if (failedCheckRequests === 2) {
      response.writeHead(503, {'content-type': 'text/plain; charset=utf-8'});
      response.end('Temporary fixture failure');
      return;
    }
    response.writeHead(200, {'content-type': 'text/html; charset=utf-8'});
    response.end('<!doctype html><main>Recovered fixture after the failed check.</main>');
    return;
  }
  if (target.pathname === '/network-boundary') {
    response.writeHead(200, {'content-type': 'text/html; charset=utf-8'});
    response.end('<!doctype html><main>Network boundary fixture.</main>');
    return;
  }
  if (target.pathname === '/script-free-source') {
    response.writeHead(200, {'content-type': 'text/html; charset=utf-8'});
    response.end('<!doctype html><main>Static source without script execution.</main><script>fetch("http://example.com/__fixture/script-ran")</script>');
    return;
  }
  if (target.pathname === '/__fixture/script-ran') {
    scriptRuns += 1;
    response.writeHead(204);
    response.end();
    return;
  }
  if (target.pathname === '/access-challenge') {
    challengeRequests += 1;
    response.writeHead(403, {'content-type': 'text/html; charset=utf-8'});
    response.end('<!doctype html><main>Challenge required</main>');
    return;
  }
  if (target.pathname === '/api/v1/products/change-diff-inbox/verify') {
    json(response, {valid:false, reason:'invalid'});
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
