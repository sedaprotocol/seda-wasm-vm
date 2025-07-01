package tallyvm_test

import (
	"os"
	"testing"

	"github.com/sedaprotocol/seda-wasm-vm/tallyvm/v2"
)

func init() {
	tempdir, err := os.MkdirTemp("", "sedad_home")
	if err != nil {
		panic(err)
	}
	tallyvm.TallyVmDir = tempdir
}

// 15.109s
func BenchmarkExecutionGo100Times(b *testing.B) {
	defer cleanup()

	file := "../test-wasm-files/tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		b.Fatal(err)
	}

	reveals := "[{\"salt\":[1],\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"salt\":[3],\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"salt\":[4],\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"
	reveals_filter := "[0,0,0]"

	b.ResetTimer()
	for i := 0; i < 100; i++ {
		tallyvm.ExecuteTallyVm(data,
			[]string{"input_here", reveals, reveals_filter},
			map[string]string{
				"CONSENSUS":          "true",
				"VM_MODE":            "tally",
				"DR_TALLY_GAS_LIMIT": "150000000000000",
			},
		)
	}
}

// 15.082s
func BenchmarkExecutionGo100TimesParallel(b *testing.B) {
	defer cleanup()
	bytesArr, argsArr, envsArr := setup_n(b.Fatal, 100)

	b.ResetTimer()
	tallyvm.ExecuteMultipleFromGoInParallel(bytesArr, argsArr, envsArr)
}

// 3.078s
func BenchmarkExecutionC100Times(b *testing.B) {
	defer cleanup()
	bytesArr, argsArr, envsArr := setup_n(b.Fatal, 100)

	b.ResetTimer()
	tallyvm.ExecuteMultipleFromC(bytesArr, argsArr, envsArr)
}
