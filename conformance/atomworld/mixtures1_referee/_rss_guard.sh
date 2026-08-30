#!/bin/bash
# Kill a probe that is about to take the machine down with it.
# Other lanes share these 31 GB; a measurement that costs a sibling campaign its
# pool is not a measurement worth having.
PAT="$1"; LIMIT_KB="${2:-6000000}"
while true; do
  pid=$(pgrep -f "$PAT" | head -1)
  [ -z "$pid" ] && { echo "[$(date -Is)] $PAT gone"; exit 0; }
  rss=$(awk '/VmRSS/{print $2}' /proc/$pid/status 2>/dev/null)
  avail=$(awk '/MemAvailable/{print $2}' /proc/meminfo)
  if [ -n "$rss" ] && [ "$rss" -gt "$LIMIT_KB" ]; then
    echo "[$(date -Is)] KILL $PAT pid=$pid rss=${rss}kB over ${LIMIT_KB}kB"
    kill -9 "$pid"; exit 1
  fi
  if [ "$avail" -lt 2000000 ]; then
    echo "[$(date -Is)] KILL $PAT pid=$pid: MemAvailable ${avail}kB below 2GB"
    kill -9 "$pid"; exit 2
  fi
  sleep 5
done
