package bind

import (
	"github.com/ebitengine/purego"
)

func execute_tally_vm(bytes []byte, args []string, envs map[string]string) FfiVmResult {
	pbindings, err := purego.Dlopen(getSystemLibrary(), purego.RTLD_NOW|purego.RTLD_GLOBAL)
	if err != nil {
		panic(err)
	}
	// inputs:
	// 1. the bytes of the wasm binary
	// 2. the arguments to the wasm module
	// 3. the array of sizes of each argument
	// 4. the number of arguments
	// 5. the environment variables
	// 6. the number of environment variables
	var executeTallyVm func([]byte, int, []string, []int, int, map[string]string, int) FfiVmResult
	purego.RegisterLibFunc(&executeTallyVm, bindings, "execute_tally_vm")
}

func getSystemLibrary() string {
	switch runtime.GOOS {
	case "darwin":
		return "./target/release/libbindings.dylib"
	case "linux":
		return "./target/release/libbindings.so"
	default:
		panic(fmt.Errorf("GOOS=%s is not supported", runtime.GOOS))
	}
}