import matplotlib.pyplot as plt

percentiles = ['p50', 'p90', 'p99', 'p99.9']

# Load test results
rust_lb = [191.62, 248.17, 407.08, 565.33]
nginx = [181.71, 342.3, 503.49, 662.28]

plt.figure(figsize=(8,5))
plt.plot(percentiles, rust_lb, marker='o', linestyle='-', label='Rust LB')
plt.plot(percentiles, nginx, marker='o', linestyle='-', label='Nginx')

plt.xlabel('Latency percentiles')
plt.ylabel('Latency (ms)')
plt.title('Latency Percentiles: Rust LB vs Nginx')
plt.legend()
plt.grid(True, linestyle='--', alpha=0.5)

plt.tight_layout()
plt.show()
