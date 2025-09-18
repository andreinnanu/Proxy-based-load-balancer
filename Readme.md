# Proxy based load balancer

## Workers

Each worker runs in its own container.

The /work endpoint simulates computation (can add artificial delay via `duration_millis` query parameter).

## Load Balancer

Forwards incoming requests to available workers.
The list of worker servers is defined in `resources/config.yml`, which is mounted into the load balancer container.

## Load Test Setup

```mermaid
flowchart TD
    subgraph Host
        K6["k6 Load Test Script"]
    end

    subgraph Docker_Network
        LB["Rust LB"]
        NGINX["Nginx"]
        FWD["Forward /work"]
        W1["Worker 1"]
        W2["Worker 2"]
        W3["Worker 3"]
        Wn["Worker N"]
    end

    K6 -->|HTTP GET /work| LB
    K6 -->|HTTP GET /work| NGINX
    
    FWD --> W1
    FWD --> W2
    FWD --> W3
    FWD --> Wn

    LB --> FWD
    NGINX --> FWD
```

Executed from the host machine:

```
docker compose build
docker compose up

# for custom Rust LB
BASE_URL=http://localhost:3000 k6 run load_tests/latency_percentiles.js

# for nginx
BASE_URL=http://localhost:4000 k6 run load_tests/latency_percentiles.js
```

Collects latency percentiles (p50, p90, p99, p99.9), throughput, and error rates.

The percentiles obtained can be fed into `load_tests/plot.py` to create a comparison graph.