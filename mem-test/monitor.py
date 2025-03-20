#!/usr/bin/env python3
import sys
import subprocess
import time

def get_rss(pid):
    """Get RSS (in KB) for the given process PID using ps."""
    try:
        rss_str = subprocess.check_output(["ps", "-o", "rss=", "-p", str(pid)]).decode().strip()
        return float(rss_str)
    except Exception as e:
        print("Error reading RSS for pid", pid, e)
        return None

def main():
    # Use the first argument as the sleep time for mem-test, defaulting to "5" if not provided.
    sleep_arg = sys.argv[1] if len(sys.argv) > 1 else "5"
    vm_args_json = sys.argv[2] if len(sys.argv) > 2 else "price_feed_tally.json"
    print(f"Running tallyVM every {sleep_arg} seconds")

    # Launch mem-test binary with the sleep argument.
    try:
        proc = subprocess.Popen(["./mem-test", vm_args_json, sleep_arg])
    except Exception as e:
        print("Failed to launch mem-test:", e)
        return

    pid = proc.pid
    print(f"Launched mem-test with PID: {pid}")

    last_rss = None
    while True:
        # Check if mem-test has terminated
        if proc.poll() is not None:
            print("mem-test terminated.")
            break

        rss = get_rss(pid)
        if rss is not None:
            if last_rss is None:
                print(f"Initial RSS value: {rss} KB")
                last_rss = rss
            elif rss != last_rss:
                diff_mb = (rss - last_rss) / 1024.0  # Convert KB difference to MB
                print(f"Last: {last_rss} KB, Current: {rss} KB, Difference: {diff_mb:.2f} MB")
                last_rss = rss

        time.sleep(1)

if __name__ == '__main__':
    main()
