package tallyvm_test

import (
	"os"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/sedaprotocol/seda-wasm-vm/tallyvm/v2"
)

func init() {
	tempdir, err := os.MkdirTemp("", "sedad_home")
	if err != nil {
		panic(err)
	}
	tallyvm.LogDir = tempdir
	tallyvm.TallyMaxBytes = 1024
}

func cleanup() {
	os.RemoveAll(tallyvm.LogDir)
}

func TestTallyBinaryWorks(t *testing.T) {
	defer cleanup()

	file := "../tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	reveals := "[{\"salt\":[1],\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"salt\":[3],\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"salt\":[4],\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"
	reveals_filter := "[0,0,0]"

	res := tallyvm.ExecuteTallyVm(data, []string{"input_here", reveals, reveals_filter}, map[string]string{
		"CONSENSUS":          "true",
		"VM_MODE":            "tally",
		"DR_TALLY_GAS_LIMIT": "300000000000000",
	})

	t.Log(res)

	assert.Equal(t, "Ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 0, res.ExitInfo.ExitCode)
	assert.NotEmpty(t, res.Result)
	assert.Empty(t, res.Stderr)
	assert.NotEmpty(t, res.Stdout)
	assert.Equal(t, uint64(5002255745075), res.GasUsed)
}

func TestTallyBinaryNoArgs(t *testing.T) {
	defer cleanup()

	file := "../tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	res := tallyvm.ExecuteTallyVm(data, []string{}, map[string]string{
		"CONSENSUS":          "true",
		"VM_MODE":            "tally",
		"DR_TALLY_GAS_LIMIT": "300000000000000",
	})

	t.Log(res)

	assert.Equal(t, "", res.ExitInfo.ExitMessage)
	assert.Equal(t, 255, res.ExitInfo.ExitCode)
	assert.Empty(t, res.Result)
	assert.NotEmpty(t, res.Stderr)
	assert.NotEmpty(t, res.Stdout)
	assert.Equal(t, uint64(5000005633275), res.GasUsed)
}

func TestTallyGasExceeded(t *testing.T) {
	defer cleanup()

	file := "../tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	res := tallyvm.ExecuteTallyVm(data, []string{}, map[string]string{
		"CONSENSUS":          "true",
		"VM_MODE":            "tally",
		"DR_TALLY_GAS_LIMIT": "123",
	})

	t.Log(res)

	assert.Equal(t, "", res.ExitInfo.ExitMessage)
	assert.Equal(t, 250, res.ExitInfo.ExitCode)
	assert.Empty(t, res.Result)
	assert.NotEmpty(t, res.Stderr)
	assert.Equal(t, uint64(123), res.GasUsed)
}

func TestTallyMaxBytesExceeded(t *testing.T) {
	defer cleanup()
	tallyvm.TallyMaxBytes = 1

	file := "../tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	reveals := "[{\"salt\":[1],\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"salt\":[3],\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"salt\":[4],\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"
	reveals_filter := "[0,0,0]"

	res := tallyvm.ExecuteTallyVm(data, []string{"input_here", reveals, reveals_filter}, map[string]string{
		"CONSENSUS":          "true",
		"VM_MODE":            "tally",
		"DR_TALLY_GAS_LIMIT": "300000000000000",
	})

	t.Log(res)

	// t.Log(tallyvm.LogDir)
	// // read file contents from LogDir/sedavm_logs/log.2025-01-14
	// logfile, err := os.ReadFile(tallyvm.LogDir + "/sedavm_logs/log.2025-01-14")
	// if err != nil {
	// 	t.Fatal(err)
	// }
	// t.Log(string(logfile))

	assert.Equal(t, "Result larger than 1bytes.", res.ExitInfo.ExitMessage)
	assert.Equal(t, 255, res.ExitInfo.ExitCode)
	assert.Nil(t, res.Result)
	assert.NotZero(t, res.ResultLen)
	assert.Empty(t, res.Stderr)
	assert.NotEmpty(t, res.Stdout)
	assert.Equal(t, uint64(5002255745075), res.GasUsed)
}
