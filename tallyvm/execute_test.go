package tallyvm_test

import (
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/sedaprotocol/seda-wasm-vm/tallyvm"
)

func init() {
	tempdir, err := os.MkdirTemp("", "sedad_home")
	if err != nil {
		panic(err)
	}
	os.Setenv("SEDAD_TALLYVM_HOME", tempdir)
}

func cleanup() {
	tempdir := os.Getenv("SEDAD_TALLYVM_HOME")
	os.RemoveAll(tempdir)
	os.Unsetenv("SEDAD_TALLYVM_HOME")
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
		"CONSENSUS":    "true",
		"VM_MODE":      "tally",
		"DR_GAS_LIMIT": "2000000",
	})

	assert.Equal(t, "Ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 0, res.ExitInfo.ExitCode)
	assert.NotEmpty(t, res.Result)
	assert.Empty(t, res.Stderr)
	assert.NotEmpty(t, res.Stdout)
	fmt.Println(res)
	t.Log(res)
}

func TestTallyBinaryNoArgs(t *testing.T) {
	defer cleanup()

	file := "../tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	res := tallyvm.ExecuteTallyVm(data, []string{}, map[string]string{
		"CONSENSUS":    "true",
		"VM_MODE":      "tally",
		"DR_GAS_LIMIT": "2000000",
	})

	assert.Equal(t, "", res.ExitInfo.ExitMessage)
	assert.Equal(t, 255, res.ExitInfo.ExitCode)
	assert.Empty(t, res.Result)
	assert.NotEmpty(t, res.Stderr)
	assert.NotEmpty(t, res.Stdout)
	fmt.Println(res)
	t.Log(res)
}

func TestTallyGasExceeded(t *testing.T) {
	defer cleanup()

	file := "../tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	res := tallyvm.ExecuteTallyVm(data, []string{}, map[string]string{
		"CONSENSUS":    "true",
		"VM_MODE":      "tally",
		"DR_GAS_LIMIT": "123",
	})

	assert.Equal(t, "", res.ExitInfo.ExitMessage)
	assert.Equal(t, 250, res.ExitInfo.ExitCode)
	assert.Empty(t, res.Result)
	assert.NotEmpty(t, res.Stderr)
	fmt.Println(res)
	t.Log(res)
}
