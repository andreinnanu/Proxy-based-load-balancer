# Proxy based load balancer

## Workers

Dummy workers for testing purposes.

Docker compose can be used to deploy multiple replicas of the workers:

    docker compose up --scale worker=9

The ports allocated for the replicas are hardcoded in the `compose.yml` file. Update it for more than 9 replicas.

## Load Balancer

Start the load balancer: `cargo run`.
By default, it listens on `127.0.0.1:3000`. 

To use a custom address, use the `--addr` option. Ex: `cargo run -- --addr 127.0.0.1:7777`.

At this stage of development, the balancer it's actually just a proxy which redirects all the incoming requests to the first worker (hardcoded - `0.0.0.0:8081`) and logs them.


