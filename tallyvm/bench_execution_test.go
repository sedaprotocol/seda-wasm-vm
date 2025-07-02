package tallyvm_test

import (
	"os"
	"testing"

	"github.com/sedaprotocol/seda-wasm-vm/tallyvm/v2"
	"github.com/stretchr/testify/assert"
)

func init() {
	tempdir, err := os.MkdirTemp("", "sedad_home")
	if err != nil {
		panic(err)
	}
	tallyvm.TallyVmDir = tempdir
}

// ~21s
func BenchmarkExecutionGo100Times(b *testing.B) {
	defer cleanup()

	file := "../test-wasm-files/tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		b.Fatal(err)
	}

	reveals := "[{\"salt\":[1],\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"salt\":[3],\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"salt\":[4],\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"
	reveals_filter := "[0,0,0]"

	results := make([]tallyvm.VmResult, 100)
	b.ResetTimer()
	for i := 0; i < 100; i++ {
		results[i] = tallyvm.ExecuteTallyVm(data,
			[]string{"input_here", reveals, reveals_filter},
			map[string]string{
				"CONSENSUS":          "true",
				"VM_MODE":            "tally",
				"DR_TALLY_GAS_LIMIT": "150000000000000",
			},
		)
	}
	b.StopTimer()

	// Check results ignore this in the benchmark
	for _, result := range results {
		assert.Equal(b, "Ok", result.ExitInfo.ExitMessage)
		assert.Equal(b, 0, result.ExitInfo.ExitCode)
		assert.NotEmpty(b, result.Result)
		assert.Empty(b, result.Stderr)
		assert.NotEmpty(b, result.Stdout)
		assert.Equal(b, 30944893003750, int(result.GasUsed))
	}
}

// ~23s
func BenchmarkExecutionGo100TimesParallel(b *testing.B) {
	defer cleanup()
	bytesArr, argsArr, envsArr := setup_n(b.Fatal, 100)

	b.ResetTimer()
	results := tallyvm.ExecuteMultipleFromGoInParallel(bytesArr, argsArr, envsArr)
	b.StopTimer()

	// Check results ignore this in the benchmark
	for _, result := range results {
		assert.Equal(b, "Ok", result.ExitInfo.ExitMessage)
		assert.Equal(b, 0, result.ExitInfo.ExitCode)
		assert.NotEmpty(b, result.Result)
		assert.Empty(b, result.Stderr)
		assert.NotEmpty(b, result.Stdout)
		assert.Equal(b, 30944893003750, int(result.GasUsed))
	}
}

// ~3s
func BenchmarkExecutionC100Times(b *testing.B) {
	defer cleanup()
	bytesArr, argsArr, envsArr := setup_n(b.Fatal, 100)

	b.ResetTimer()
	results := tallyvm.ExecuteMultipleFromC(bytesArr, argsArr, envsArr)
	b.StopTimer()

	// Check results ignore this in the benchmark
	for _, result := range results {
		assert.Equal(b, "Ok", result.ExitInfo.ExitMessage)
		assert.Equal(b, 0, result.ExitInfo.ExitCode)
		assert.NotEmpty(b, result.Result)
		assert.Empty(b, result.Stderr)
		assert.NotEmpty(b, result.Stdout)
		assert.Equal(b, 30944893003750, int(result.GasUsed))
	}
}
