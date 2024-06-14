//go:build darwin

package tallyvm

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lseda_tally_vm
import "C"
