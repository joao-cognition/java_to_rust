// Traffic mix shared by every benchmark phase (see bench/run.sh):
//   40% GET /owners/{id}, 20% GET /owners?lastName=<prefix>, 15% GET /vets,
//   10% GET /pettypes, 10% GET /pets/{id}, 5% POST /owners
import http from 'k6/http';
import { check } from 'k6';

const BASE = __ENV.BASE_URL || 'http://localhost:9966/petclinic';
const PHASE = __ENV.PHASE || 'fixed';
const RATE = parseInt(__ENV.RATE || '500', 10);
const DURATION = (__ENV.DURATION || '60') + 's';
const VUS = parseInt(__ENV.VUS || '64', 10);

const OWNER_IDS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const PET_IDS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
const LAST_NAMES = ['Davis', 'Franklin', 'Rodriquez', 'McTavish', 'Coleman', 'Black', 'Escobito', 'Schroeder', 'Estaban', 'D'];

const scenarios = {
  fixed: {
    executor: 'constant-arrival-rate',
    rate: RATE,
    timeUnit: '1s',
    duration: DURATION,
    preAllocatedVUs: 64,
    maxVUs: 512,
  },
  peak: {
    executor: 'constant-vus',
    vus: VUS,
    duration: DURATION,
  },
};

export const options = {
  scenarios: { load: scenarios[PHASE] || scenarios.fixed },
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(50)', 'p(90)', 'p(95)', 'p(99)'],
  discardResponseBodies: false,
};

const params = { headers: { 'Content-Type': 'application/json' } };
const pick = (arr) => arr[Math.floor(Math.random() * arr.length)];

export default function () {
  const r = Math.random();
  let res;
  if (r < 0.40) {
    res = http.get(`${BASE}/api/owners/${pick(OWNER_IDS)}`, { tags: { name: 'GET /owners/{id}' } });
  } else if (r < 0.60) {
    res = http.get(`${BASE}/api/owners?lastName=${pick(LAST_NAMES)}`, { tags: { name: 'GET /owners?lastName=' } });
  } else if (r < 0.75) {
    res = http.get(`${BASE}/api/vets`, { tags: { name: 'GET /vets' } });
  } else if (r < 0.85) {
    res = http.get(`${BASE}/api/pettypes`, { tags: { name: 'GET /pettypes' } });
  } else if (r < 0.95) {
    res = http.get(`${BASE}/api/pets/${pick(PET_IDS)}`, { tags: { name: 'GET /pets/{id}' } });
  } else {
    const body = JSON.stringify({
      firstName: 'Load',
      lastName: 'Loadtest',
      address: '1 Benchmark Way',
      city: 'Madison',
      telephone: '6085550000',
    });
    res = http.post(`${BASE}/api/owners`, body, { ...params, tags: { name: 'POST /owners' } });
  }
  check(res, { 'status is 2xx': (x) => x.status >= 200 && x.status < 300 });
}

export function handleSummary(data) {
  const file = __ENV.SUMMARY_FILE || 'stdout';
  return { [file]: JSON.stringify(data, null, 2) };
}
