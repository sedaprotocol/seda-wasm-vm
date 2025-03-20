# Mem Testing Tools

Useful to see if memory leaking in a long running process that calls the tallyVM. 

## Running

1. Build the `test-vm.wasm` with `make build`. It's output to `target/debug/wasm32-wasip1/test-vm.wasm`
1. Make sure the `test-vm.wasm` is in the `mem-test` directory.
1. Make sure you are in the `mem-test` directory.
1. Run `go build` to generate the `mem-test` binary.
1. You can then run `./monitor.py [time_in_seconds] [vm_args_json_path]`. The `json` file is so you can easily change the wasm, method, and args to it.
   1. The time in seconds is optional and defaults to 5.
   1. The json path is also optional and defaults to the `price_feed_tally.json` by default.
1. You can `ctrl-c` to quit at any time. 

## Reading

You can see what the output looks like here:

```
./monitor.py 1
Running tallyVM every 1 seconds
Launched mem-test with PID: 2719980
Initial RSS value: 6560.0 KB
Last: 6560.0 KB, Current: 36892.0 KB, Difference: 29.62 MB
Last: 36892.0 KB, Current: 58520.0 KB, Difference: 21.12 MB
Last: 58520.0 KB, Current: 63700.0 KB, Difference: 5.06 MB
Last: 63700.0 KB, Current: 64456.0 KB, Difference: 0.74 MB
Last: 64456.0 KB, Current: 66864.0 KB, Difference: 2.35 MB
Last: 66864.0 KB, Current: 68392.0 KB, Difference: 1.49 MB
Last: 68392.0 KB, Current: 71368.0 KB, Difference: 2.91 MB
Last: 71368.0 KB, Current: 74384.0 KB, Difference: 2.95 MB
Last: 74384.0 KB, Current: 79324.0 KB, Difference: 4.82 MB
```

You see how often it's running the tallyVM.
It tells you the PID of the process for the `mem-test`.

It reads out the initial value of the [RSS](https://en.wikipedia.org/wiki/Resident_set_size) when the process first starts.
After that it checks for the difference in the RSS value for that process every second showing:
The last value, the current value, and the difference.
The difference will be positive if the memory being used is growing, and negative if the memory being used shrinks.

Eventually the field should stabilize.