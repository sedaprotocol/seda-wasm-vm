(module
  ;; Import WASI functions
  (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))

  ;; Create memory with maximum possible pages (65536 pages = 4GB)
  (memory (export "memory") 65536)

  ;; Export _start function required by WASI
  (func $start (export "_start")
    i32.const 0
    call $proc_exit
  )
)
