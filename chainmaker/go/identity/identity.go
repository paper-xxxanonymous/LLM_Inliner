/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package main

import (
	utils "chainmaker.org/chainmaker/contract-utils/address"
	"encoding/json"
	"fmt"
	"github.com/TKOTKCh/contract-sdk-go-wasm/sdk"
	"strings"
)

const (
	paramAdminAddress = "adminAddress"
	paramAddress      = "address"
	keyAdminAddress   = "adminAddress"
)

//go:wasmexport init_contract
func InitContract() {
	ctx := sdk.NewSimContext()
	// 获取参数
	adminAddress, err := ctx.ArgString(paramAdminAddress)
	var adminAddressStr string
	if err != sdk.SUCCESS || len(adminAddress) == 0 {
		adminAddressStr, _ = ctx.GetSenderAddr()
	}
	adminAddresses := strings.Split(adminAddressStr, ",")
	initContract(adminAddresses)
}

//go:wasmexport upgrade
func Upgrade() {
	ctx := sdk.NewSimContext()
	ctx.SuccessResult("Upgrade contract success")
}

// 这个函数不暴露给wasmer，InitContract暴露
func initContract(adminAddresses []string) {
	ctx := sdk.NewSimContext()
	adminAddressBytes, _ := json.Marshal(adminAddresses)
	err1 := ctx.PutStateByte("identity", keyAdminAddress, adminAddressBytes)
	if err1 != sdk.SUCCESS {
		ctx.ErrorResult("set admin address of identityInfo failed")
		return
	}
	err2 := ctx.PutState("identity", "userCount", "0") //这个userCount没什么作用，只是为了和go的identity合约保持一致
	if err2 != sdk.SUCCESS {
		ctx.ErrorResult("set user count of identityInfo failed")
		return
	}
	ctx.EmitEvent("alterAdminAddress", "")
	ctx.SuccessResult("Init contract success")
	return
}

// 由于wasmer虚拟机的写法是直接调用对应函数，不像dockergo那样从一个总的invokeContract中调
// 这里与rust保持一致，不写invokecontract而是直接调，
//
//go:wasmexport addWriteList
func addWriteList() {
	ctx := sdk.NewSimContext()
	paramAddress, _ := ctx.ArgString(paramAddress)
	//paramAddress := "1,2,3"
	var addresses []string
	if len(paramAddress) != 0 {
		addresses = strings.Split(paramAddress, ",")
	}
	if len(addresses) == 0 {
		ctx.ErrorResult("address of param should not be empty")
		return
	}
	for _, address := range addresses {
		if !utils.IsValidAddress(address) {
			ctx.ErrorResult(fmt.Sprintf("addWriteList address[%s,%d] format error", address, len(address)))
			return
		}
		//其实ctx可以直接存string，但dockergo的sdk存的byte
		ctx.PutState("identity", address, "1")
	}
	ctx.SuccessResult("add write list success")
	return
}

//go:wasmexport alterAdminAddress
func alterAdminAddress() {
	ctx := sdk.NewSimContext()
	paramAddress, _ := ctx.ArgString(paramAddress)
	var adminAddress []string
	if len(paramAddress) != 0 {
		adminAddress = strings.Split(paramAddress, ",")
	}
	if len(adminAddress) == 0 {
		ctx.ErrorResult("adminAddress of param should not be empty")
		return
	}

	if !senderIsAdmin() {
		ctx.ErrorResult("sender is not admin")
		return
	}
	adminAddressByte, _ := json.Marshal(adminAddress)
	err := ctx.PutStateByte("identity", keyAdminAddress, adminAddressByte)
	if err != sdk.SUCCESS {
		ctx.ErrorResult("alter admin address of identityInfo failed")
		return
	}
	ctx.EmitEvent("alterAdminAddress", "")
	ctx.SuccessResult("OK")
	return
}

func senderIsAdmin() bool {
	ctx := sdk.NewSimContext()
	sender, _ := ctx.GetSenderAddr()
	adminAddressByte, err := ctx.GetStateByte("identity", keyAdminAddress)
	if err != sdk.SUCCESS || len(adminAddressByte) == 0 {
		ctx.Log(fmt.Sprintf("Get adminAddressList failed, err:%s", err))
		return false
	}
	var adminAddress []string
	_ = json.Unmarshal(adminAddressByte, &adminAddress)
	for _, addr := range adminAddress {
		if addr == sender {
			return true
		}
	}
	return false
}

//go:wasmexport removeWriteList
func removeWriteList() {
	ctx := sdk.NewSimContext()
	paramAddress, _ := ctx.ArgString(paramAddress)
	var addresses []string
	if len(paramAddress) != 0 {
		addresses = strings.Split(paramAddress, ",")
	}
	if len(addresses) == 0 {
		ctx.ErrorResult("address of param should not be empty")
		return
	}
	for _, address := range addresses {
		ctx.DeleteState("identity", address)
	}
	ctx.SuccessResult("remove write list success")
	return
}

//go:wasmexport isApprovedUser
func isApprovedUser() {
	ctx := sdk.NewSimContext()
	paramAddress, _ := ctx.ArgString(paramAddress)
	var addresses []string
	if len(paramAddress) != 0 {
		addresses = strings.Split(paramAddress, ",")
	}
	if len(addresses) == 0 {
		ctx.ErrorResult("address of param should not be empty")
		return
	}

	flag := true
	for _, addr := range addresses {
		val, err := ctx.GetState("identity", addr)
		if err != sdk.SUCCESS || len(val) == 0 {
			flag = false
		}
	}
	if flag {
		ctx.SuccessResult("true")
		return
	} else {
		ctx.SuccessResult("false")
		return
	}
}

//go:wasmexport address
func address() {
	ctx := sdk.NewSimContext()
	addr, err := ctx.GetSenderAddr()
	if err != sdk.SUCCESS {
		ctx.ErrorResult(fmt.Sprintf("get address of sender failed, err:%s", err))
		return
	}
	if len(addr) == 0 {
		ctx.ErrorResult("addr is empty")
		return
	}

	ctx.SuccessResult(addr)
	return
}

//go:wasmexport callerAddress
func callerAddress() {
	var param = make(map[string][]byte)
	ctx := sdk.NewSimContext()
	resp, resultCode := ctx.CallContract("identity", "address", param)
	if resultCode != sdk.SUCCESS {
		ctx.ErrorResult(fmt.Sprintf("call contract failed, err:%s", resp))
		return
	} else {
		ctx.SuccessResult(fmt.Sprintf("call contract success :%s", resp))
		return
	}
}

func main() {

}
