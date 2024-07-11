//go:build linux && muslc && arm64

package tallyvm

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lwasmvm_muslc
import "C"
