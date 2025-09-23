import http from "k6/http";
import { Trend } from "k6/metrics";

let responseTime = new Trend("response_time");

const BASE_URL = __ENV.BASE_URL || "http://localhost:3000";

export let options = {
    stages: [
        { duration: "2m", target: 1000 },
        { duration: "2m", target: 1000 },
        { duration: "30s", target: 0 },
    ],
    thresholds: {
        "http_req_duration": ["p(50)<200", "p(90)<400", "p(99)<1000", "p(99.9)<2000"],
    },
};

export default function () {
    const res = http.get(`${BASE_URL}/work?duration_millis=50`);
    responseTime.add(res.timings.duration);
}
