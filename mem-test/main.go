package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"time"

	"github.com/sedaprotocol/seda-wasm-vm/tallyvm/v3"
)

type Config struct {
	WasmFile string          `json:"wasm_file"`
	Method   string          `json:"method"`
	Args     json.RawMessage `json:"args"`
}

func main() {
	if len(os.Args) != 3 {
		fmt.Println("Usage: <config.json> <sleep_seconds>")
		os.Exit(1)
	}

	sleepSeconds, err := strconv.Atoi(os.Args[2])
	if err != nil || sleepSeconds <= 0 {
		fmt.Println("Invalid sleep seconds:", os.Args[2])
		os.Exit(1)
	}

	configData, err := os.ReadFile(os.Args[1])
	if err != nil {
		fmt.Println("Error reading config file:", err)
		os.Exit(1)
	}

	var cfg Config
	if err := json.Unmarshal(configData, &cfg); err != nil {
		fmt.Println("Error parsing config file:", err)
		os.Exit(1)
	}

	// Validate that cfg.Args is valid JSON.
	if !json.Valid(cfg.Args) {
		fmt.Println("Invalid JSON in args")
		os.Exit(1)
	}

	wasmData, err := os.ReadFile(cfg.WasmFile)
	if err != nil {
		fmt.Println("Error reading wasm file:", err)
		os.Exit(1)
	}

	// Convert the method to hex and prepend it to the args slice.
	methodHex := hex.EncodeToString([]byte(cfg.Method))
	args := []string{methodHex, string(cfg.Args)}

	tallyvm.TallyMaxBytes = 1024

	for {
		res := tallyvm.ExecuteTallyVm(wasmData, args, map[string]string{
			"CONSENSUS":          "true",
			"VM_MODE":            "tally",
			"DR_TALLY_GAS_LIMIT": "150000000000000",
		})
		fmt.Printf("VMExitCode: %d\n", res.ExitInfo.ExitCode)
		fmt.Printf("VMExitMessage: %s\n", res.ExitInfo.ExitMessage)
		time.Sleep(time.Duration(sleepSeconds) * time.Second)
	}
}
