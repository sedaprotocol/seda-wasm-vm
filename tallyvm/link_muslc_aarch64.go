//go:build linux && muslc && arm64

package tallyvm

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lseda_tally_vm_muslc.aarch64
import "C"
