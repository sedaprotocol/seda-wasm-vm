package bind_go_test

import (
	"os"
	"testing"

	"github.com/sedaprotocol/seda-wasm-vm/bind_go"
)

func TestCowsay(t *testing.T) {
	foo, _ := os.Getwd()
	t.Log(foo)
	file := "../debug.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	bind_go.ExecuteTallyVm(data, []string{"1", "2"}, map[string]string{
		"PATH": os.Getenv("SHELL"),
	})
}
