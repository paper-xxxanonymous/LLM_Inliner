package main

import (
	"github.com/TKOTKCh/contract-sdk-go-wasm/sdk"
)

const (
	paramToken    = "tokenId"
	paramFrom     = "from"
	paramTo       = "to"
	paramMetaData = "metadata"
	paramAmount   = "amount"
	trueString    = "true"
)

//go:wasmexport init_contract
func InitContract() {
	ctx := sdk.NewSimContext()
	ctx.SuccessResult("Init contract success")
	return
}

//go:wasmexport upgrade
func Upgrade() {
	ctx := sdk.NewSimContext()
	ctx.SuccessResult("Upgrade contract success")
	return
}

//go:wasmexport buyNow
func buyNow() {
	ctx := sdk.NewSimContext()

	tokenId, _ := ctx.ArgString(paramToken)
	from, _ := ctx.ArgString(paramFrom)
	to, _ := ctx.ArgString(paramTo)
	metadata, _ := ctx.ArgString(paramMetaData)

	if tokenId == "" || from == "" || to == "" {
		ctx.ErrorResult("tokenId/from/to should not be empty")
		return
	}

	// 添加白名单
	args := map[string][]byte{
		"address": []byte(from + "," + to),
	}
	resp, code := ctx.CallContract("identity", "addWriteList", args)
	if code != sdk.SUCCESS || string(resp) != "add write list success" {
		ctx.Log("[buyNow] addWriteList failed: " + string(resp))
		ctx.ErrorResult("[buyNow] addWriteList failed" + " " + string(resp))
		return
	}

	// 检查from是否在白名单
	args = map[string][]byte{"address": []byte(from)}
	resp, code = ctx.CallContract("identity", "isApprovedUser", args)
	if code != sdk.SUCCESS || string(resp) != trueString {
		ctx.ErrorResult("address[" + from + "] not registered")
		return
	}

	// 检查to是否在白名单
	args = map[string][]byte{"address": []byte(to)}
	resp, code = ctx.CallContract("identity", "isApprovedUser", args)
	if code != sdk.SUCCESS || string(resp) != trueString {
		ctx.ErrorResult("address[" + to + "] not registered")
		return
	}

	// 向from铸造 NFT
	args = map[string][]byte{
		"to":       []byte(from),
		"tokenId":  []byte(tokenId),
		"metadata": []byte(metadata),
	}
	resp, code = ctx.CallContract("erc721", "mint", args)
	if code != sdk.SUCCESS || string(resp) != "mint success" {
		ctx.Log("[buyNow] mint failed: " + string(resp))
		ctx.ErrorResult("[buyNow] mint failed")
		return
	}

	// 设置 from 的资产授权
	args = map[string][]byte{
		"approvalFrom": []byte(from),
	}
	resp, code = ctx.CallContract("erc721", "setApprovalForAll2", args)
	if code != sdk.SUCCESS || string(resp) != "setApprovalForAll2 success" {
		ctx.Log("[buyNow] setApprovalForAll2 failed: " + string(resp))
		ctx.ErrorResult("[buyNow] setApprovalForAll2 failed")
		return
	}

	// erc721 NFT 转移 from -> to
	args = map[string][]byte{
		"from":    []byte(from),
		"to":      []byte(to),
		"tokenId": []byte(tokenId),
	}
	resp, code = ctx.CallContract("erc721", "transferFrom", args)
	if code != sdk.SUCCESS || string(resp) != "transfer success" {
		ctx.ErrorResult("erc721 transferFrom error: " + string(resp))
		return
	}
	spender, _ := ctx.GetSenderPk()
	ctx.EmitEvent("buyNow", from+to+spender)
	ctx.SuccessResult("buyNow success")
	return
}

func main() {}
