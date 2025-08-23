package tallyvm_test

import (
	"encoding/hex"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"testing"
	"time"

	"cosmossdk.io/log"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"

	"github.com/sedaprotocol/seda-wasm-vm/tallyvm/v3"
)

func init() {
	tempdir, err := os.MkdirTemp("", "sedad_home")
	if err != nil {
		panic(err)
	}
	tallyvm.TallyVmDir = tempdir
}

func cleanup() {
	os.RemoveAll(tallyvm.TallyVmDir)
}

func TestTallyBinaryWorks(t *testing.T) {
	defer cleanup()

	file := "../test-wasm-files/tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	reveals := "[{\"salt\":[1],\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"salt\":[3],\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"salt\":[4],\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"
	reveals_filter := "[0,0,0]"

	res := tallyvm.ExecuteTallyVm(data, []string{"input_here", reveals, reveals_filter}, map[string]string{
		"CONSENSUS":          "true",
		"VM_MODE":            "tally",
		"DR_TALLY_GAS_LIMIT": "150000000000000",
	})

	t.Log(res)

	assert.Equal(t, "Ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 0, res.ExitInfo.ExitCode)
	assert.NotEmpty(t, res.Result)
	assert.Empty(t, res.Stderr)
	assert.NotEmpty(t, res.Stdout)
	assert.Equal(t, 30944893003750, int(res.GasUsed))
}

func TestTallyBinaryNoArgs(t *testing.T) {
	defer cleanup()

	file := "../test-wasm-files/tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	res := tallyvm.ExecuteTallyVm(data, []string{}, map[string]string{
		"CONSENSUS":          "true",
		"VM_MODE":            "tally",
		"DR_TALLY_GAS_LIMIT": "150000000000000",
	})

	t.Log(res)

	assert.Equal(t, "Not ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 255, res.ExitInfo.ExitCode)
	assert.Empty(t, res.Result)
	assert.NotEmpty(t, res.Stderr)
	assert.NotEmpty(t, res.Stdout)
	assert.Equal(t, 12177280647500, int(res.GasUsed))
}

func TestTallyGasExceeded(t *testing.T) {
	defer cleanup()

	file := "../test-wasm-files/tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	startup_gas := 1_000_000_000_000 * 5
	total_gas := startup_gas + 1_000

	res := tallyvm.ExecuteTallyVm(data, []string{}, map[string]string{
		"CONSENSUS":          "true",
		"VM_MODE":            "tally",
		"DR_TALLY_GAS_LIMIT": strconv.Itoa(total_gas),
	})

	t.Log(res)

	assert.Equal(t, "Not ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 250, res.ExitInfo.ExitCode)
	assert.Empty(t, res.Result)
	assert.NotEmpty(t, res.Stderr)
	assert.Equal(t, total_gas, int(res.GasUsed))
}

func TestTallyMaxBytesExceeded(t *testing.T) {
	defer cleanup()
	tallyvm.TallyMaxBytes = 1

	file := "../test-wasm-files/tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	reveals := "[{\"salt\":[1],\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"salt\":[3],\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"salt\":[4],\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"
	reveals_filter := "[0,0,0]"

	res := tallyvm.ExecuteTallyVm(data, []string{"input_here", reveals, reveals_filter}, map[string]string{
		"CONSENSUS":          "true",
		"VM_MODE":            "tally",
		"DR_TALLY_GAS_LIMIT": "150000000000000",
	})

	t.Log(res)

	assert.Equal(t, "Result larger than 1bytes.", res.ExitInfo.ExitMessage)
	assert.Equal(t, 255, res.ExitInfo.ExitCode)
	assert.Nil(t, res.Result)
	assert.NotZero(t, res.ResultLen)
	assert.Empty(t, res.Stderr)
	assert.NotEmpty(t, res.Stdout)
	assert.Equal(t, 30944893003750, int(res.GasUsed))
}

func TestDrMaxBytesExceededIsFine(t *testing.T) {
	defer cleanup()
	tallyvm.TallyMaxBytes = 1

	file := "../test-wasm-files/tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	reveals := "[{\"salt\":[1],\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"salt\":[3],\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"salt\":[4],\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"
	reveals_filter := "[0,0,0]"

	res := tallyvm.ExecuteTallyVm(data, []string{"input_here", reveals, reveals_filter}, map[string]string{
		"CONSENSUS":          "true",
		"VM_MODE":            "dr",
		"DR_TALLY_GAS_LIMIT": "150000000000000",
	})

	t.Log(res)

	assert.Equal(t, "Ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 0, res.ExitInfo.ExitCode)
	assert.NotEmpty(t, res.Result)
	assert.NotZero(t, res.ResultLen)
	assert.Empty(t, res.Stderr)
	assert.Empty(t, res.Stdout)
	assert.Equal(t, 9237079512500, int(res.GasUsed))
}

func TestUserlandNonZeroExitCode(t *testing.T) {
	defer cleanup()
	tallyvm.TallyMaxBytes = 1024
	tallyvm.TallyMaxStdoutBytes = 512
	tallyvm.TallyMaxStderrBytes = 512

	file := "../test-wasm-files/null_byte_string.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	reveals := "[{\"reveal\":[123,34,98,108,111,99,107,72,97,115,104,34,58,34 ,48,120,57,50,55,55,98,53,53,55,48,48,100,97,57,48,53,48,98,53,53,97,97,54,55,52,48,55,49,57,101,50,53,98,48,48,102,51,57,97,99,99,49,53,102,49,49,98,54,52,48,99,98,56,50,101,52,48,100,97,56,102,56,54,48,100,34,44,34,98,108,111,99,107,78,117,109,98,101,114,34,58,34,48,120,49,52,50,98,98,55,56,34,44,34,102,114,111,109, 34,58,34,48,120,99,48,100,98,98,53,49,101,54,48,55,102,52,57,53,54,57,99,52,50,99,53,99,101,101,50,101,98,51,51,100,99,53,98,97,99,50,56,100,53,34,125],\"salt\":[211,175,124,217,173,184,107,223,93,111,189,56,113,215,248,115,214,157,229,183,30,213,237,186,209,254,246,247,222,155,241,183,157,123,93,180,213,253,57,211,19 0,56,125,189,120,247,93,116],\"id\":\"f495c06137a92787312086267884196ec4476f6faf4bd074eafb289b65de272f\",\"exit_code\":0,\"gas_used\":42369302985625,\"proxy_public_keys\":[]}]"
	reveals_filter := "[0]"

	res := tallyvm.ExecuteTallyVm(data, []string{"0xd66196506df89851d1200962310cc4bd5ee7b4d19c852a4afd0ccf07e636606f", reveals, reveals_filter}, map[string]string{
		"CONSENSUS":             "true",
		"VM_MODE":               "tally",
		"DR_TALLY_GAS_LIMIT":    "150000000000000",
		"DR_REPLICATION_FACTOR": "1",
	})

	t.Log(res)

	assert.Equal(t, "Not ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 1, res.ExitInfo.ExitCode)
	assert.NotEmpty(t, res.Result)
	assert.Equal(t, 12059308161250, int(res.GasUsed))
}

func TestMaxOutputByteLimits(t *testing.T) {
	defer cleanup()
	tallyvm.TallyMaxBytes = 1024
	tallyvm.TallyMaxStdoutBytes = 2
	tallyvm.TallyMaxStderrBytes = 2

	file := "../test-wasm-files/test-vm.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	method := "hello_world"
	method_hex := hex.EncodeToString([]byte(method))

	res := tallyvm.ExecuteTallyVm(data, []string{method_hex}, map[string]string{
		"CONSENSUS":             "true",
		"VM_MODE":               "tally",
		"DR_TALLY_GAS_LIMIT":    "150000000000000",
		"DR_REPLICATION_FACTOR": "1",
	})

	t.Log(res)

	assert.Equal(t, "Ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 0, res.ExitInfo.ExitCode)
	assert.Empty(t, res.Result)
	assert.Equal(t, 11089317466250, int(res.GasUsed))
	assert.Equal(t, res.Stdout[0], "Fo")
	assert.Equal(t, res.Stderr[0], "Ba")
}

func TestMeteringBeforeBranchSources(t *testing.T) {
	defer cleanup()
	tallyvm.TallyMaxBytes = 1024
	tallyvm.TallyMaxStdoutBytes = 512
	tallyvm.TallyMaxStderrBytes = 512

	file := "../test-wasm-files/cache_misses.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	start := time.Now()
	res := tallyvm.ExecuteTallyVm(data, []string{}, map[string]string{
		"CONSENSUS":             "true",
		"VM_MODE":               "tally",
		"DR_TALLY_GAS_LIMIT":    "150000000000000",
		"DR_REPLICATION_FACTOR": "1",
	})
	elapsed := time.Since(start)

	t.Logf("Execution took %s", elapsed)
	t.Log(res)

	assert.Equal(t, "Not ok", res.ExitInfo.ExitMessage)
	assert.Empty(t, res.Result)
	assert.Equal(t, 5000125831250, int(res.GasUsed))
	assert.LessOrEqual(t, elapsed, time.Duration(5000000))
}

func TestMemoryPreallocTooMuch(t *testing.T) {
	defer cleanup()
	tallyvm.TallyMaxBytes = 1024
	tallyvm.TallyMaxStdoutBytes = 512
	tallyvm.TallyMaxStderrBytes = 512

	file := "../test-wasm-files/test-vm.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	method := "memory_fill_prealloc"
	method_hex := hex.EncodeToString([]byte(method))

	res := tallyvm.ExecuteTallyVm(data, []string{method_hex}, map[string]string{
		"CONSENSUS":             "true",
		"VM_MODE":               "tally",
		"DR_TALLY_GAS_LIMIT":    "150000000000000",
		"DR_REPLICATION_FACTOR": "1",
	})

	t.Log(res)

	assert.Equal(t, "Not ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 252, res.ExitInfo.ExitCode)
	assert.Empty(t, res.Result)
	assert.Equal(t, "memory allocation of 44832551 bytes failed\n", res.Stderr[0])
	assert.Equal(t, 12104438591250, int(res.GasUsed))
}

func TestMemoryDynamicTooMuch(t *testing.T) {
	defer cleanup()
	tallyvm.TallyMaxBytes = 1024
	tallyvm.TallyMaxStdoutBytes = 512
	tallyvm.TallyMaxStderrBytes = 512

	file := "../test-wasm-files/test-vm.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	method := "memory_fill_dynamic"
	method_hex := hex.EncodeToString([]byte(method))

	res := tallyvm.ExecuteTallyVm(data, []string{method_hex}, map[string]string{
		"CONSENSUS":             "true",
		"VM_MODE":               "tally",
		"DR_TALLY_GAS_LIMIT":    "150000000000000",
		"DR_REPLICATION_FACTOR": "1",
	})

	t.Log(res)

	assert.Equal(t, "Not ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 252, res.ExitInfo.ExitCode)
	assert.Empty(t, res.Result)
	assert.Equal(t, "memory allocation of 8192000 bytes failed\n", res.Stderr[0])
	assert.Equal(t, 21244868027500, int(res.GasUsed))
}

func setup_n(Fatal func(args ...any), n int) ([][]byte, [][]string, []map[string]string) {
	file := "../test-wasm-files/tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		Fatal(err)
	}

	reveals := "[{\"salt\":[1],\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"salt\":[3],\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"salt\":[4],\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"
	reveals_filter := "[0,0,0]"

	bytesArr := make([][]byte, n)
	argsArr := make([][]string, n)
	envsArr := make([]map[string]string, n)
	for i := range bytesArr {
		bytesArr[i] = data
		argsArr[i] = []string{"input_here", reveals, reveals_filter}
		envsArr[i] = map[string]string{
			"CONSENSUS":          "true",
			"VM_MODE":            "tally",
			"DR_TALLY_GAS_LIMIT": "150000000000000",
		}
	}
	return bytesArr, argsArr, envsArr
}

func TestExecutionGoMultipleParallel(t *testing.T) {
	defer cleanup()
	bytesArr, argsArr, envsArr := setup_n(t.Fatal, 2)

	tallyvm.ExecuteMultipleFromGoInParallel(bytesArr, argsArr, envsArr)
}

func TestExecutionCMultiple(t *testing.T) {
	defer cleanup()
	bytesArr, argsArr, envsArr := setup_n(t.Fatal, 2)

	tallyvm.ExecuteMultipleFromC(bytesArr, argsArr, envsArr)
}

func TestExecutionCMultipleParallel(t *testing.T) {
	defer cleanup()
	bytesArr, argsArr, envsArr := setup_n(t.Fatal, 2)

	tallyvm.ExecuteMultipleFromCParallel(bytesArr, argsArr, envsArr)
}

func copyDir(src string, dst string) error {
	return filepath.Walk(src, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(src, path)
		if err != nil {
			return err
		}
		target := filepath.Join(dst, rel)

		if info.IsDir() {
			return os.MkdirAll(target, info.Mode())
		}

		srcFile, err := os.Open(path)
		if err != nil {
			return err
		}
		defer srcFile.Close()

		dstFile, err := os.OpenFile(target, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, info.Mode())
		if err != nil {
			return err
		}
		defer dstFile.Close()

		_, err = io.Copy(dstFile, srcFile)
		return err
	})
}

func TestCacheInvalidation(t *testing.T) {
	defer cleanup()

	file := "../test-wasm-files/tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	reveals := "[{\"salt\":[1],\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"salt\":[3],\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"salt\":[4],\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"
	reveals_filter := "[0,0,0]"

	res := tallyvm.ExecuteTallyVm(data, []string{"input_here", reveals, reveals_filter}, map[string]string{
		"CONSENSUS":          "true",
		"VM_MODE":            "tally",
		"DR_TALLY_GAS_LIMIT": "150000000000000",
	})
	assert.Equal(t, 0, res.ExitInfo.ExitCode)

	path, currentVersion := tallyvm.GetInvalidateWasmCacheInfo()
	// Make a copy to a different version than the current one
	err = os.Mkdir(filepath.Join(path, "v1.0.0-fake"), 0755)
	if err != nil {
		t.Fatal(err)
	}
	err = copyDir(filepath.Join(path, currentVersion), filepath.Join(path, "v1.0.0-fake"))
	if err != nil {
		t.Fatal(err)
	}

	// create an illegal file to ensure it doesn't break the invalidation
	illegalFilePath := filepath.Join(path, "illegal_file.txt")
	illegalFile, err := os.Create(illegalFilePath)
	if err != nil {
		t.Fatal(err)
	}
	defer illegalFile.Close()

	err = tallyvm.InvalidateWasmCache(sdk.Context{}.WithLogger(log.NewLogger(os.Stdout, log.LevelOption(zerolog.DebugLevel))))
	if err != nil {
		t.Fatal(err)
	}

	// ensure the v1.0.0-fake directory is deleted
	_, err = os.Stat(path)
	if os.IsNotExist(err) {
		t.Fatal("v1.0.0-fake directory was not deleted")
	}

	// ensure the current version directory is not deleted
	_, err = os.Stat(currentVersion)
	if !os.IsNotExist(err) {
		t.Fatal("current version directory was deleted")
	}
}
