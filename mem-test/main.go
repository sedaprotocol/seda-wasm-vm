package main

import (
	"encoding/hex"
	"fmt"
	"os"
	"strconv"
	"time"

	"github.com/sedaprotocol/seda-wasm-vm/tallyvm/v2"
)

func main() {
	// read the number of seconds to sleep from the command line
	// default to 5 seconds if no argument is provided
	if len(os.Args) != 2 {
		fmt.Println("Usage: sleep <seconds>")
		os.Exit(1)
	}
	seconds, err := strconv.Atoi(os.Args[1])
	if err != nil {
		fmt.Println("Invalid number of seconds:", os.Args[1])
		os.Exit(1)
	}

	file := "./test-vm.wasm"
	data, err := os.ReadFile(file)
	if err != nil {
		panic(err)
	}

	method := "price_feed_tally"
	method_hex := hex.EncodeToString([]byte(method))
	reveals := "[{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":200,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,50,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":198,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,52,53,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":201,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,50,56,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":199,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,55,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":202,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,48,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":197,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,52,49,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":200,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,53,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":203,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,57,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":196,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,51,125]},{\"salt\":[115,101,100,97,95,115,100,107],\"exit_code\":0,\"gas_used\":201,\"reveal\":[123,34,112,114,105,99,101,34,58,32,49,49,50,57,57,51,54,125]}]"
	// reveals_hex := hex.EncodeToString([]byte(reveals))
	consensus := "[0,0,0,0,0,0,0,0,0,0]"

	tallyvm.TallyMaxBytes = 1024

	for {
		tallyvm.ExecuteTallyVm(data, []string{method_hex, reveals, consensus}, map[string]string{
			"CONSENSUS":          "true",
			"VM_MODE":            "tally",
			"DR_TALLY_GAS_LIMIT": "150000000000000",
		})

		time.Sleep(time.Duration(seconds) * time.Second)
	}
}
