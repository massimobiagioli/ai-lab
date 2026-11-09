// k6 load/soak scenario for the edge read path (WP-24).
import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:3000';
const TOKEN = __ENV.TOKEN || '';
const LAB = __ENV.LAB_ID || 'd0000000-0000-7000-8000-000000000001';

export const options = {
  scenarios: {
    edge: {
      executor: 'constant-vus',
      vus: Number(__ENV.VUS || 5),
      duration: __ENV.DURATION || '15s',
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<300'],
  },
};

export default function () {
  const headers = TOKEN ? { Authorization: `Bearer ${TOKEN}` } : {};
  const health = http.get(`${BASE}/api/v1/health/live`);
  check(health, { 'health 200': (response) => response.status === 200 });
  const events = http.get(`${BASE}/api/v1/events?lab_id=${LAB}&after=0&limit=100`, {
    headers,
  });
  check(events, { 'events accepted': (response) => response.status < 500 });
  sleep(1.2);
}
