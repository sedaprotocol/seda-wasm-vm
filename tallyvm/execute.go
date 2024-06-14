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
	Stdout   []string
	Stderr   []string
	Result   []byte
	ExitInfo ExitInfo
}

func ExecuteTallyVm(bytes []byte, args []string, envs map[string]string) VmResult {
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

	result := C.execute_tally_vm(
		(*C.uint8_t)(unsafe.Pointer(&bytes[0])), C.uintptr_t(len(bytes)),
		(**C.char)(unsafe.Pointer(&argsC[0])), C.uintptr_t(len(args)),
		(**C.char)(unsafe.Pointer(&keys[0])), (**C.char)(unsafe.Pointer(&values[0])), C.uintptr_t(len(envs)),
	)
	exitMessage := C.GoString(result.exit_info.exit_message)
	exitCode := int(result.exit_info.exit_code)

	defer C.free_ffi_vm_result(&result)

	resultLen := int(result.result_len)
	resultBytes := make([]byte, resultLen)
	if resultLen > 0 {
		resultBytes = (*[1 << 30]byte)(unsafe.Pointer(result.result_ptr))[:resultLen:resultLen]
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
		Stdout: stdout,
		Stderr: stderr,
		Result: resultBytes,
		ExitInfo: ExitInfo{
			ExitMessage: exitMessage,
			ExitCode:    exitCode,
		},
	}
}
