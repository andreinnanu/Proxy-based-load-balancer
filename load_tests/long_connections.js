import http from "k6/http";
import { sleep } from "k6";

export let options = {
  vus: 100,
  duration: "2s",
};

export default function () {
  let userId = __VU;
  let iter = __ITER;

  let url = `http://127.0.0.1:3000/work`;

  if (userId % 2 == 0) {
    url += "?duration_millis=10000"
  }

  let res = http.get(url);

  console.log(`VU ${userId}, Iteration ${iter}, Status ${res.status}`);

  sleep(0.1);
}