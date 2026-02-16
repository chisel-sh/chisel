# API Latency Spike Postmortem

## Summary
On Feb 14, 2026, API latency increased by 400% due to a cache eviction storm.

## Impact
- **Duration:** 45 minutes
- **Affected Users:** 15% of traffic
- **Root Cause:** Redis configuration drift

## Timeline
- 14:00 - Alert fired: `high_latency`
- 14:05 - Cache miss rate observed at 85%
- 14:15 - Rollback deployed
