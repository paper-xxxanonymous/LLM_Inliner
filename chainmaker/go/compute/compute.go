package main

import (
	"crypto/sha256"
	"fmt"
	"github.com/TKOTKCh/contract-sdk-go-wasm/sdk"
	"math/big"
	"strconv"
)

//go:wasmexport normalStrconv
func normalStrconv() {
	ctx := sdk.NewSimContext()

	var result1 int64
	var result2 int64
	var result3 int
	// 进行 10000 次大数运算
	for i := 0; i < 10000; i++ {
		//返回19088743
		result1, _ = strconv.ParseInt("1234567", 16, 32)
		//返回0
		result2, _ = strconv.ParseInt("1234567", 2, 64)
		//返回1234567
		result3, _ = strconv.Atoi("1234567")
	}
	ctx.SuccessResult(fmt.Sprintf("success bigNumCal: %d %d %d", result1, result2, result3))
}

//go:wasmexport normalCal
func normalCal() {
	result := 0
	for i := 0; i < 1000000; i++ {
		result += i
	}
}

//go:wasmexport hashCal
func hashCal() {

	hashInput := "ChainMaker Performance Test"
	var hashResult [32]byte
	for i := 0; i < 10000; i++ {
		hashResult = sha256.Sum256([]byte(hashInput))
	}
	_ = hashResult
}

//go:wasmexport bigNumCal
func bigNumCal() {
	a := big.NewInt(2)
	exp := big.NewInt(100000)
	mod := big.NewInt(1000000007)
	var result *big.Int

	// 进行 10000 次大数运算
	for i := 0; i < 10000; i++ {
		result = new(big.Int).Exp(a, exp, mod)
	}
	_ = result

}

func main() {}
