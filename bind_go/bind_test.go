package bind_go_test

import (
	"fmt"
	"os"
	"testing"

	"github.com/sedaprotocol/seda-wasm-vm/bind_go"
	"github.com/stretchr/testify/assert"
)

func TestCowsay(t *testing.T) {
	file := "../debug.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	res := bind_go.ExecuteTallyVm(data, []string{"1", "2"}, map[string]string{
		"ENV_1": "1",
	})

	assert.Equal(t, "Ok", res.ExitInfo.ExitMessage)
	assert.Equal(t, 0, res.ExitInfo.ExitCode)
	assert.Empty(t, res.Result)
	assert.Empty(t, res.Stderr)
	assert.Equal(t, 1, len(res.Stdout))
	assert.Equal(t, "Currently our args are _start,1,2\nChecking against: 1\nWhat is this! 1\n", res.Stdout[0])
	fmt.Println(res)
	t.Log(res)
}
