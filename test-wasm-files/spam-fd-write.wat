(module
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (memory 1)
  (export "memory" (memory 0))
  (data (i32.const 0) "Hello from WAT! ")
  (func $write_loop
    (local $iovs i32)
    (local $nwritten i32)
    (local.set $iovs (i32.const 100))
    (i32.store (local.get $iovs) (i32.const 0))
    (i32.store (i32.add (local.get $iovs) (i32.const 4)) (i32.const 15))
    (loop $infinite
      (call $fd_write
        (i32.const 1)
        (local.get $iovs)
        (i32.const 1)
        (local.get $nwritten))
      (drop)
      (br $infinite)))
  (export "_start" (func $write_loop))) 