package tallyvm

// #include <libseda_tally_vm.h>
import "C"

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"sync"
	"unsafe"

	sdk "github.com/cosmos/cosmos-sdk/types"
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

var TallyVmDir string
var TallyMaxBytes uint = 1024
var TallyMaxStdoutBytes uint = 512
var TallyMaxStderrBytes uint = 512

func buildVmSettings() (C.FfiVmSettings, *C.char) {
	// convert config dir to C string
	tallyVmDirC := C.CString(TallyVmDir)
	return C.FfiVmSettings{
		sedad_home:       tallyVmDirC,
		max_result_bytes: C.uintptr_t(TallyMaxBytes),
		stdout_limit:     C.uintptr_t(TallyMaxStdoutBytes),
		stderr_limit:     C.uintptr_t(TallyMaxStderrBytes),
	}, tallyVmDirC
}

type cRequest struct {
	req   C.FfiTallyRequest
	frees []*C.char
}

func newCRequest(
	bytes []byte,
	args []string,
	envs map[string]string,
) cRequest {
	var frees []*C.char

	// convert wasm bytes to C slice
	var wasmBytesPtr *C.uint8_t
	if len(bytes) > 0 {
		wasmBytesPtr = (*C.uint8_t)(unsafe.Pointer(&bytes[0]))
	}

	// convert args to C slice
	argsC := make([]*C.char, len(args))
	for i, s := range args {
		cstr := C.CString(s)
		argsC[i] = cstr
		frees = append(frees, cstr)
	}

	// convert envs to C slices
	keysC := make([]*C.char, len(envs))
	valsC := make([]*C.char, len(envs))
	i := 0
	for k, v := range envs {
		ck := C.CString(k)
		cv := C.CString(v)
		keysC[i], valsC[i] = ck, cv
		frees = append(frees, ck, cv)
		i++
	}

	// allocate C arrays for the pointer slices
	argsArrSize := C.uintptr_t(len(argsC)) * C.uintptr_t(unsafe.Sizeof(argsC[0]))
	argsArrPtr := C.malloc(argsArrSize)
	frees = append(frees, (*C.char)(argsArrPtr))
	argsArr := (*[1 << 28]*C.char)(argsArrPtr)
	copy(argsArr[:], argsC)

	keysArrSize := C.uintptr_t(len(keysC)) * C.uintptr_t(unsafe.Sizeof(keysC[0]))
	keysArrPtr := C.malloc(keysArrSize)
	frees = append(frees, (*C.char)(keysArrPtr))
	keysArr := (*[1 << 28]*C.char)(keysArrPtr)
	copy(keysArr[:], keysC)

	valsArrSize := C.uintptr_t(len(valsC)) * C.uintptr_t(unsafe.Sizeof(valsC[0]))
	valsArrPtr := C.malloc(valsArrSize)
	frees = append(frees, (*C.char)(valsArrPtr))
	valsArr := (*[1 << 28]*C.char)(valsArrPtr)
	copy(valsArr[:], valsC)

	return cRequest{
		req: C.FfiTallyRequest{
			wasm_bytes:     wasmBytesPtr,
			wasm_bytes_len: C.uintptr_t(len(bytes)),
			args_ptr:       (**C.char)(argsArrPtr),
			args_count:     C.uintptr_t(len(argsC)),
			env_keys_ptr:   (**C.char)(keysArrPtr),
			env_values_ptr: (**C.char)(valsArrPtr),
			env_count:      C.uintptr_t(len(keysC)),
		},
		frees: frees,
	}
}

func (r *cRequest) cleanup() {
	for _, p := range r.frees {
		C.free(unsafe.Pointer(p))
	}
}

func buildResultFromC(cResult *C.FfiVmResult) VmResult {
	exitMessage := C.GoString(cResult.exit_info.exit_message)
	exitCode := int(cResult.exit_info.exit_code)
	defer C.free_ffi_vm_result(cResult)

	resultLen := int(cResult.result_len)
	resultBytes := make([]byte, resultLen)
	if resultLen > 0 && exitCode != 255 {
		src := (*[1 << 30]byte)(unsafe.Pointer(cResult.result_ptr))[:resultLen:resultLen]
		copy(resultBytes, src)
	}
	var resultPtr *[]byte
	if exitCode != 255 {
		resultPtr = &resultBytes
	}

	stdoutLen := int(cResult.stdout_len)
	stdout := make([]string, stdoutLen)
	if stdoutLen > 0 {
		cs := (*[1 << 30]*C.char)(unsafe.Pointer(cResult.stdout_ptr))[:stdoutLen:stdoutLen]
		for i, cstr := range cs {
			stdout[i] = C.GoString(cstr)
		}
	}

	stderrLen := int(cResult.stderr_len)
	stderr := make([]string, stderrLen)
	if stderrLen > 0 {
		cs := (*[1 << 30]*C.char)(unsafe.Pointer(cResult.stderr_ptr))[:stderrLen:stderrLen]
		for i, cstr := range cs {
			stderr[i] = C.GoString(cstr)
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
		GasUsed: uint64(cResult.gas_used),
	}
}

func ExecuteTallyVm(
	bytes []byte,
	args []string,
	envs map[string]string,
) VmResult {
	cSettings, configDirC := buildVmSettings()
	defer C.free(unsafe.Pointer(configDirC))

	cr := newCRequest(bytes, args, envs)
	defer cr.cleanup()

	// convert config dir to C string and build request, then call the C function
	result := C.execute_tally_request(cSettings, cr.req)
	return buildResultFromC(&result)
}

func ExecuteMultipleFromGoInParallel(
	bytes [][]byte,
	args [][]string,
	envs []map[string]string,
) []VmResult {
	var wg sync.WaitGroup
	results := make([]VmResult, len(bytes))
	maxProcs := runtime.GOMAXPROCS(-1) - 1

	// Prevent too many goroutines from being created since they each call Cgo
	// which creates a new thread.
	semaphore := make(chan struct{}, maxProcs)
	wg.Add(len(bytes))

	for i := range bytes {
		semaphore <- struct{}{}

		go func(i int) {
			defer func() {
				<-semaphore
				wg.Done()
			}()

			fmt.Println("Executing", i, maxProcs)
			result := ExecuteTallyVm(bytes[i], args[i], envs[i])

			results[i] = result
		}(i)
	}
	wg.Wait()
	return results
}

func ExecuteMultipleFromC(bytes [][]byte, args [][]string, envs []map[string]string) []VmResult {
	cSettings, configDirC := buildVmSettings()
	defer C.free(unsafe.Pointer(configDirC))

	cReqs := make([]cRequest, len(bytes))
	for i := range bytes {
		cReqs[i] = newCRequest(bytes[i], args[i], envs[i])
		defer cReqs[i].cleanup()
	}

	count := len(cReqs)
	size := C.uintptr_t(count) * C.uintptr_t(unsafe.Sizeof(cReqs[0].req))
	cArray := C.malloc(size)
	defer C.free(cArray)
	arr := (*[1 << 28]C.FfiTallyRequest)(cArray)
	for i := 0; i < count; i++ {
		arr[i] = cReqs[i].req
	}

	cResults := C.execute_tally_requests(
		cSettings,
		(*C.FfiTallyRequest)(cArray),
		C.uintptr_t(count),
	)

	results := make([]VmResult, count)
	slice := (*[1 << 30]C.FfiVmResult)(unsafe.Pointer(cResults))[:count:count]
	for i := 0; i < count; i++ {
		results[i] = buildResultFromC(&slice[i])
	}
	return results
}

func ExecuteMultipleFromCParallel(bytes [][]byte, args [][]string, envs []map[string]string) []VmResult {
	cSettings, configDirC := buildVmSettings()
	defer C.free(unsafe.Pointer(configDirC))

	cReqs := make([]cRequest, len(bytes))
	for i := range bytes {
		cReqs[i] = newCRequest(bytes[i], args[i], envs[i])
		defer cReqs[i].cleanup()
	}

	count := len(cReqs)
	size := C.uintptr_t(count) * C.uintptr_t(unsafe.Sizeof(cReqs[0].req))
	cArray := C.malloc(size)
	defer C.free(cArray)
	arr := (*[1 << 28]C.FfiTallyRequest)(cArray)
	for i := 0; i < count; i++ {
		arr[i] = cReqs[i].req
	}

	cResults := C.execute_tally_requests_parallel(
		cSettings,
		(*C.FfiTallyRequest)(cArray),
		C.uintptr_t(count),
	)

	results := make([]VmResult, count)
	slice := (*[1 << 30]C.FfiVmResult)(unsafe.Pointer(cResults))[:count:count]
	for i := 0; i < count; i++ {
		results[i] = buildResultFromC(&slice[i])
	}
	return results
}

func GetInvalidateWasmCacheInfo() (string, string) {
	tallyVmDirC := C.CString(TallyVmDir)
	defer C.free(unsafe.Pointer(tallyVmDirC))

	cResponse := C.invalidate_wasm_cache_info(tallyVmDirC)
	defer C.free_ffi_invalidate_wasm_cache_info(&cResponse)
	path := C.GoString(cResponse.wasm_cache_dirs)
	currentVersion := C.GoString(cResponse.version_name)

	return path, currentVersion
}

func InvalidateWasmCache(ctx sdk.Context) error {
	path, currentVersion := GetInvalidateWasmCacheInfo()
	ctx.Logger().Info("Invalidating WASM cache for versions not matching:", currentVersion)

	vmVersionDirs, err := os.ReadDir(path)
	if err != nil {
		return err
	}

	// loop through the directories, that have the versions as their names
	for _, vmVersionDir := range vmVersionDirs {
		if !vmVersionDir.IsDir() {
			ctx.Logger().Error("Illegal file/folder in Tally WASM Cache Directory:", vmVersionDir.Name())
			continue
		}

		// Check if the folder name does not match the current version
		// if so delete it
		name := vmVersionDir.Name()
		if name != currentVersion {
			if err := os.RemoveAll(filepath.Join(path, vmVersionDir.Name())); err != nil {
				return err
			}
		}

	}

	return nil
}
