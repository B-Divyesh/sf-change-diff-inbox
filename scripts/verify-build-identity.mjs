const expected = process.env.EXPECTED_BUILD_SHA;
const healthUrl = process.env.HEALTH_URL || 'http://127.0.0.1:18080/health';

if (!/^[0-9a-f]{40}$/.test(expected || '')) {
  throw new Error('EXPECTED_BUILD_SHA must be a full lowercase 40-character Git SHA');
}

const response = await fetch(healthUrl);
if (!response.ok) {
  throw new Error(`/health returned ${response.status}`);
}

const health = await response.json();
if (health.status !== 'ok' || health.build !== expected) {
  throw new Error(`unexpected health identity: ${JSON.stringify(health)}`);
}

console.log(`build identity verified: ${health.build}`);
