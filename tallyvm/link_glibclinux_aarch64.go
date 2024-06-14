//go:build linux && !muslc && arm64

package tallyvm

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lseda_tally_vm.aarch64
import "C"
