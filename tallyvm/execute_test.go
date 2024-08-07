package tallyvm_test

import (
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/sedaprotocol/seda-wasm-vm/tallyvm"
)

func TestTallyBinary(t *testing.T) {
	file := "../tally.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	reveals := "[{\"salt\":[1],\"exit_code\":0,\"gas_used\":\"200\",\"reveal\":[2]},{\"salt\":[3],\"exit_code\":0,\"gas_used\":\"201\",\"reveal\":[5]},{\"salt\":[4],\"exit_code\":0,\"gas_used\":\"202\",\"reveal\":[6]}]"
	reveals_filter := "[0,0,0]"

	res := tallyvm.ExecuteTallyVm(data, []string{"input_here", reveals, reveals_filter}, map[string]string{
		"CONSENSUS": "true",
		"VM_MODE":   "tally",
	})

	assert.Equal(t, "Ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 0, res.ExitInfo.ExitCode)
	assert.NotEmpty(t, res.Result)
	assert.Empty(t, res.Stderr)
	assert.NotEmpty(t, res.Stdout)
	fmt.Println(res)
	t.Log(res)
}
