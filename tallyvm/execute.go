package tallyvm

// #include <libseda_tally_vm.h>
import "C"

import (
	"unsafe"
)

type ExitInfo struct {
	ExitMessage string
	ExitCode    int
}

type VmResult struct {
	Stdout    []string
	Stderr    []string
	Result    *[]byte
	ResultLen int
	ExitInfo  ExitInfo
	GasUsed   uint64
}

var LogDir string
var TallyMaxBytes uint

func ExecuteTallyVm(bytes []byte, args []string, envs map[string]string) VmResult {
	// convert config dir to C string
	configDirC := C.CString(LogDir)
	defer C.free(unsafe.Pointer(configDirC))

	argsC := make([]*C.char, len(args))
	for i, s := range args {
		argsC[i] = C.CString(s)
		defer C.free(unsafe.Pointer(argsC[i]))
	}

	keys := make([]*C.char, len(envs))
	values := make([]*C.char, len(envs))
	i := 0

	for k, v := range envs {
		keys[i] = C.CString(k)
		defer C.free(unsafe.Pointer(keys[i]))
		values[i] = C.CString(v)
		defer C.free(unsafe.Pointer(values[i]))
		i++
	}

	var bytesPtr *C.uint8_t
	if len(bytes) > 0 {
		bytesPtr = (*C.uint8_t)(unsafe.Pointer(&bytes[0]))
	}

	var argsPtr **C.char
	if len(args) > 0 {
		argsPtr = &argsC[0]
	}

	var keysPtr **C.char
	var valuesPtr **C.char
	if len(envs) > 0 {
		keysPtr = &keys[0]
		valuesPtr = &values[0]
	}

	result := C.execute_tally_vm(
		configDirC,
		bytesPtr, C.uintptr_t(len(bytes)),
		argsPtr, C.uintptr_t(len(args)),
		keysPtr, valuesPtr, C.uintptr_t(len(envs)),
		C.uintptr_t(TallyMaxBytes),
	)
	exitMessage := C.GoString(result.exit_info.exit_message)
	exitCode := int(result.exit_info.exit_code)

	defer C.free_ffi_vm_result(&result)

	resultLen := int(result.result_len)
	resultBytes := make([]byte, resultLen)
	if resultLen > 0 && exitCode != 255 {
		resultSrc := (*[1 << 30]byte)(unsafe.Pointer(result.result_ptr))[:resultLen:resultLen]
		copy(resultBytes, resultSrc)
	}
	var resultPtr *[]byte
	if exitCode != 255 {
		resultPtr = &resultBytes
	}

	stdoutLen := int(result.stdout_len)
	stdout := make([]string, stdoutLen)
	if stdoutLen > 0 {
		cStrings := (*[1 << 30]*C.char)(unsafe.Pointer(result.stdout_ptr))[:stdoutLen:stdoutLen]
		for i, cStr := range cStrings {
			stdout[i] = C.GoString(cStr)
		}
	}

	stderrLen := int(result.stderr_len)
	stderr := make([]string, stderrLen)
	if stderrLen > 0 {
		cStrings := (*[1 << 30]*C.char)(unsafe.Pointer(result.stderr_ptr))[:stderrLen:stderrLen]
		for i, cStr := range cStrings {
			stderr[i] = C.GoString(cStr)
		}
	}

	return VmResult{
		Stdout:    stdout,
		Stderr:    stderr,
		Result:    resultPtr,
		ResultLen: resultLen,
		ExitInfo: ExitInfo{
			ExitMessage: exitMessage,
			ExitCode:    exitCode,
		},
		GasUsed: uint64(result.gas_used),
	}
}
