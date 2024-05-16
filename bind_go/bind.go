package bind_go

/*
#include <../tallyvm/libseda_tally_vm.h>
*/
import "C"

import (
	"fmt"
	"runtime"
	"unsafe"

	"github.com/ebitengine/purego"
)

func getSystemLibrary() string {
	switch runtime.GOOS {
	case "darwin":
		return "../target/debug/libseda_tally_vm.dylib"
	case "linux":
		return "../target/debug/libseda_tally_vm.so"
	default:
		panic(fmt.Errorf("GOOS=%s is not supported", runtime.GOOS))
	}
}

type FfiExitInfo struct {
	ExitMessage string
	ExitCode    int
}

type FfiVmResult struct {
	Stdout    string
	StdoutLen int
	Stderr    string
	StderrLen int
	Result    []byte
	ResultLen int
	ExitInfo  FfiExitInfo
}

func ExecuteTallyVm(bytes []byte, args []string, envs map[string]string) {
	pbindings, err := purego.Dlopen(getSystemLibrary(), purego.RTLD_NOW|purego.RTLD_GLOBAL)
	if err != nil {
		panic(err)
	}

	fmt.Println("go args: ", args)

	argsC := make([]*C.char, len(args))
	for i, s := range args {
		argsC[i] = C.CString(s)
		defer C.free(unsafe.Pointer(argsC[i]))
	}
	var executeTallyVm func([]byte, int, []*C.char, int, []*C.char, []*C.char, int) FfiVmResult
	purego.RegisterLibFunc(&executeTallyVm, pbindings, "execute_tally_vm")
	keys := make([]*C.char, 0, len(envs))
	values := make([]*C.char, 0, len(envs))
	for k := range envs {
		key := C.CString(k)
		keys = append(keys, key)
		defer C.free(unsafe.Pointer(key))
		value := C.CString(envs[k])
		values = append(values, value)
		defer C.free(unsafe.Pointer(value))
	}
	// executeTallyVm(bytes, len(bytes), args, len(args), keys, values, len(envs))
	executeTallyVm(bytes, len(bytes), argsC, len(args), keys, values, len(envs))
}
