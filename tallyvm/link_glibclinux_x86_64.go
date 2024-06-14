//go:build linux && !muslc && amd64

package tallyvm

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lseda_tally_vm.x86_64
import "C"
