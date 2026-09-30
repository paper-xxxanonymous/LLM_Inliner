/*
	Copyright (C) BABEC. All rights reserved. Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package main

import (
	"chainmaker.org/chainmaker/contract-utils/address"
	"chainmaker.org/chainmaker/contract-utils/safemath"
	"encoding/json"
	"fmt"
	"github.com/TKOTKCh/contract-sdk-go-wasm/sdk"
	"strconv"
	"strings"
)

type tokenLatestTxInfo struct {
	TxId        string
	BlockHeight uint64
	From        string
	To          string
}

type accountTokens struct {
	Account string
	Tokens  []string
}

const (
	erc721InfoMapName      = "erc721"
	balanceInfoMapName     = "balanceInfo"
	accountMapName         = "accountInfo"
	tokenOwnerMapName      = "tokenIds"
	tokenInfoMapName       = "tokenInfo"
	tokenApproveMapName    = "tokenApprove"
	operatorApproveMapName = "operatorApprove"
	trueString             = "1"
	falseString            = "0"
)

// 安装合约初始化函数
//
//go:wasmexport init_contract
func InitContract() {
	ctx := sdk.NewSimContext()
	err := updateErc721Info()
	if err != nil {
		ctx.ErrorResult("Init contract failed")
		return
	}
	ctx.SuccessResult("Init contract success")
	return
}

// 升级合约函数
//
//go:wasmexport upgrade
func Upgrade() {
	ctx := sdk.NewSimContext()
	err := updateErc721Info()
	if err != nil {
		ctx.ErrorResult("Upgrade contract failed")
		return
	}
	ctx.SuccessResult("Upgrade contract success")
	return
}

// 更新ERC721信息的内部函数
func updateErc721Info() error {
	ctx := sdk.NewSimContext()
	// 获取参数
	name, _ := ctx.ArgString("name")
	symbol, _ := ctx.ArgString("symbol")
	tokenURI, _ := ctx.ArgString("tokenURI")

	// 更新名称
	if len(name) > 0 {
		if err := ctx.PutState(erc721InfoMapName, "name", name); err != sdk.SUCCESS {
			return fmt.Errorf("set name of erc721Info failed")
		}
	}

	// 更新符号
	if len(symbol) > 0 {
		if err := ctx.PutState(erc721InfoMapName, "symbol", symbol); err != sdk.SUCCESS {
			return fmt.Errorf("set symbol of erc721Info failed")
		}
	}

	// 更新tokenURI
	if len(tokenURI) > 0 {
		if err := ctx.PutState(erc721InfoMapName, "tokenURI", tokenURI); err != sdk.SUCCESS {
			return fmt.Errorf("set tokenURI of erc721Info failed")
		}
	}

	// 更新管理员
	admin, err := ctx.GetSenderPk()
	if err != sdk.SUCCESS {
		return fmt.Errorf("get sender failed")
	}
	err = ctx.PutState(erc721InfoMapName, "admin", admin)
	if err != sdk.SUCCESS {
		return fmt.Errorf("set admin of erc721Info failed")
	}
	return nil
}
func getBalance(account string) (balance *safemath.SafeUint256, err1 error) {
	ctx := sdk.NewSimContext()
	balanceBytes, err := ctx.GetStateByte(balanceInfoMapName, account)
	if err != sdk.SUCCESS {
		//balance, _ := safemath.ParseSafeUint256(string(0))
		//return balance, nil
		return nil, fmt.Errorf("get balance failed, err")
	}
	balance, ok := safemath.ParseSafeUint256(string(balanceBytes))
	if !ok {
		//balance, ok = safemath.ParseSafeUint256(string(0))
		return nil, fmt.Errorf("balance bytes invalid")
	}

	return balance, nil
}

// 查询账户余额
//
//go:wasmexport balanceOf
func balanceOf() {
	ctx := sdk.NewSimContext()
	account, _ := ctx.ArgString("account")

	if !address.IsValidAddress(account) {
		ctx.ErrorResult("ERC721: balanceOf from the invalid address")
		return
	}
	if address.IsZeroAddress(account) {
		ctx.ErrorResult("ERC721: address zero is not a valid owner")
		return
	}

	balanceCount, err := getBalance(account)
	if err != nil {
		ctx.ErrorResult("Get balance failed")
		return
	}
	ctx.SuccessResult(balanceCount.ToString())
	return
}

// 查询token所有者
//
//go:wasmexport ownerOf
func ownerOf() {
	ctx := sdk.NewSimContext()
	tokenIdStr, _ := ctx.ArgString("tokenId")
	tokenId, ok := safemath.ParseSafeUint256(tokenIdStr)
	if !ok {
		ctx.ErrorResult("Parse tokenId failed")
		return
	}
	owner, err := ctx.GetState(tokenOwnerMapName, tokenId.ToString())
	if err != sdk.SUCCESS || len(owner) == 0 {
		ctx.ErrorResult("get owner failed")
		return
	}
	ctx.SuccessResult(owner)
	return
}

// 设置token授权
//
//go:wasmexport approve
func approve() {
	ctx := sdk.NewSimContext()
	to, _ := ctx.ArgString("to")
	tokenIdStr, _ := ctx.ArgString("tokenId")
	tokenId, ok := safemath.ParseSafeUint256(tokenIdStr)
	if !ok {
		ctx.ErrorResult("Parse tokenId failed")
		return
	}
	// 获取当前所有者
	owner, err := ctx.GetState(tokenOwnerMapName, tokenId.ToString())
	if err != sdk.SUCCESS {
		ctx.ErrorResult("get owner failed")
		return
	}
	if owner == to {
		ctx.ErrorResult("approval to current owner")
		return
	}

	// 验证调用者权限
	sender, _ := ctx.GetSenderPk()
	if owner != sender {
		if !isApprovedForAll(owner, sender) {
			ctx.ErrorResult("ERC721: approve caller is not token owner or approved for all")
			return
		}
	}

	// 保存授权信息
	if err := ctx.PutState(tokenApproveMapName, tokenId.ToString(), to); err != sdk.SUCCESS {
		ctx.ErrorResult("set owner failed")
		return
	}

	ctx.EmitEvent("approve", owner, to, tokenId.ToString())
	ctx.SuccessResult("approve success")
	return
}

// 获取token授权信息
//
//go:wasmexport getApproved
func getApproved() {
	ctx := sdk.NewSimContext()
	tokenIdStr, _ := ctx.ArgString("tokenId")
	tokenId, ok := safemath.ParseSafeUint256(tokenIdStr)
	if !ok {
		ctx.ErrorResult("Parse tokenId failed")
		return
	}
	approvedTo, err := ctx.GetState(tokenApproveMapName, tokenId.ToString())
	if err != sdk.SUCCESS {
		ctx.ErrorResult("get approved failed")
		return
	}
	ctx.SuccessResult(approvedTo)
	return
}

// 设置全局操作员权限
// 这个函数不暴露
func setApprovalForAll(operator string, approved bool) {
	ctx := sdk.NewSimContext()

	sender, _ := ctx.GetSenderPk()
	if sender == operator {
		ctx.ErrorResult("ERC721: approve to caller")
		return
	}

	var approvedStr string
	if approved {
		approvedStr = trueString
	} else {
		approvedStr = falseString
	}

	key := sender + "_" + operator
	if err := ctx.PutState(operatorApproveMapName, key, approvedStr); err != sdk.SUCCESS {
		ctx.ErrorResult("set operator approve failed")
		return
	}
	ctx.EmitEvent("ApprovalForAll", sender, operator, approvedStr)
	ctx.SuccessResult("setApprovalForAll success")
	return
}

// 设置全局操作员权限
// 函数是我自己改写的，授权调用者（msg.sender）对于第三方（operator）所有资产的管理权，approved为true表示有权，false表示无权
//
//go:wasmexport setApprovalForAll2
func setApprovalForAll2() {
	ctx := sdk.NewSimContext()
	approvalFrom, _ := ctx.ArgString("approvalFrom")
	operator := approvalFrom
	sender, _ := ctx.GetSenderPk()
	if sender == operator {
		ctx.ErrorResult("ERC721: approve to caller")
		return
	}

	approvedStr := trueString
	key := operator + "_" + sender
	if err := ctx.PutState(operatorApproveMapName, key, approvedStr); err != sdk.SUCCESS {
		ctx.ErrorResult("set operator approve failed")
		return
	}
	ctx.EmitEvent("ApprovalForAll2", operator, sender, approvedStr)
	ctx.SuccessResult("setApprovalForAll2 success")
	return
}

// 查询全局权限

func isApprovedForAll(owner, sender string) bool {
	ctx := sdk.NewSimContext()

	val, err := ctx.GetState(operatorApproveMapName, owner+"_"+sender)
	if err != sdk.SUCCESS {
		ctx.ErrorResult("get approved val from approve info failed")
		return false
	}

	if string(val) == trueString {
		ctx.SuccessResult(trueString)
		return true
	} else {
		ctx.SuccessResult(falseString)
		return false
	}

}

// 转账功能
//
//go:wasmexport transferFrom
func transferFrom() {
	ctx := sdk.NewSimContext()
	from, _ := ctx.ArgString("from")
	to, _ := ctx.ArgString("to")
	tokenIdStr, _ := ctx.ArgString("tokenId")
	tokenId, ok := safemath.ParseSafeUint256(tokenIdStr)
	if !ok {
		ctx.ErrorResult("Parse tokenId failed")
		return
	}

	// 权限验证
	sender, _ := ctx.GetSenderPk()
	if !isApprovedOrOwner(sender, tokenId) {
		//ctx.ErrorResult("ERC721: caller is not token owner or approved")
		//return
	}

	// 执行转账
	if transfer(from, to, tokenId) {
		ctx.EmitEvent("transfer", from, to, tokenId.ToString())
		ctx.SuccessResult("transfer success")
		return
	}
}

func transfer(from, to string, tokenId *safemath.SafeUint256) bool {
	ctx := sdk.NewSimContext()
	// 获取当前所有者
	owner, err := ctx.GetState(tokenOwnerMapName, tokenId.ToString())
	if err != sdk.SUCCESS {
		//ctx.ErrorResult("get owner failed")
		return false
	}
	if owner != from {
		//ctx.ErrorResult("ERC721: transfer from incorrect owner")
		return false
	}
	if !address.IsValidAddress(to) {
		//ctx.ErrorResult("ERC20: transfer to the invalid address")
		return false
	}
	if address.IsZeroAddress(to) {
		//ctx.ErrorResult("ERC20: transfer to the zero address")
		return false
	}
	err = ctx.DeleteState(tokenApproveMapName, tokenId.ToString())
	if err != sdk.SUCCESS {
		//ctx.ErrorResult(fmt.Sprintf("delete token approve failed, err"))
		return false
	}

	// update "from" balance count
	err1 := decreaseTokenCountByOne(from)
	if err1 != nil {
		//ctx.ErrorResult(err1.Error())
		return false
	}

	// update "to" balance count
	err1 = increaseTokenCountByOne(to)
	if err1 != nil {
		//ctx.ErrorResult(err1.Error())
		return false
	}

	// update token owner
	err1 = setTokenOwner(to, tokenId)
	if err1 != nil {
		//ctx.ErrorResult(err1.Error())
		return false
	}

	err1 = setAccountToken(from, to, tokenId)
	if err1 != nil {
		//ctx.ErrorResult(err1.Error())
		return false
	}

	err1 = setTokenLatestTxInfo(tokenId, from, to)
	if err1 != nil {
		//ctx.ErrorResult(err1.Error())
		return false
	}
	return true
}
func isApprovedOrOwner(sender string, tokenId *safemath.SafeUint256) bool {
	ctx := sdk.NewSimContext()
	// 获取当前所有者
	owner, err := ctx.GetState(tokenOwnerMapName, tokenId.ToString())
	if err != sdk.SUCCESS {
		ctx.ErrorResult("get owner failed")
		return false
	}
	if owner == sender {
		return true
	}
	if isApprovedForAll(owner, sender) {
		return true
	}
	approvedTo, err := ctx.GetState(tokenApproveMapName, tokenId.ToString())
	return approvedTo == sender
}

func setTokenLatestTxInfo(tokenId *safemath.SafeUint256, from, to string) error {
	ctx := sdk.NewSimContext()
	txId, err := ctx.GetTxId()
	if err != sdk.SUCCESS {
		return fmt.Errorf("get tx id failed, err")
	}
	blockHeightStr, err := ctx.GetBlockHeight()
	if err != sdk.SUCCESS {
		return fmt.Errorf("get block height failed, err")
	}
	//这个contracts-sdk-tinygo没有timestamp
	blockHeight, _ := strconv.ParseUint(blockHeightStr, 10, 64)
	tkTxInfo := &tokenLatestTxInfo{
		TxId:        txId,
		BlockHeight: blockHeight,
		From:        from,
		To:          to,
	}
	latestInfoBytes, _ := json.Marshal(tkTxInfo)
	//latestInfoBytes := []byte("")
	//_ = tkTxInfo
	ctx.Log(fmt.Sprintf("setTokenLatestTxInfo to %s", string(latestInfoBytes)))
	err = ctx.PutStateByte(tokenInfoMapName, tokenId.ToString()+"_latestTxInfo", latestInfoBytes)
	if err != sdk.SUCCESS {
		return fmt.Errorf("set latestTxInfo of token failed, err")
	}
	return nil
}

//go:wasmexport testgasJSONUnmarshal
func testgasJSONUnmarshal() {
	type tstruct struct {
	}
	jsonStr := `{}`
	var p tstruct
	json.Unmarshal([]byte(jsonStr), &p)
}

//go:wasmexport testgasJSONMarshal
func testgasJSONMarshal() {
	testgasJsonMarshal_stub("0000000000000000000000000000000000000000", "8acfaca5eeec9f6f7c23c4ffac969b86f27799b0")
	testgasJsonMarshal_stub2("0000000000000000000000000000000000000000", "8acfaca5eeec9f6f7c23c4ffac969b86f27799b0")
	testgasJsonMarshal_stub3("0000000000000000000000000000000000000000", "8acfaca5eeec9f6f7c23c4ffac969b86f27799b0")
}

func testgasJsonMarshal_stub(from, to string) {
	ctx := sdk.NewSimContext()
	tkTxInfo := &tokenLatestTxInfo{
		TxId:        "TX_ID",
		BlockHeight: 111,
		From:        from,
		To:          to,
	}
	latestInfoBytes, _ := json.Marshal(tkTxInfo)
	_ = string(latestInfoBytes)
	ctx.Log(fmt.Sprintf("setTokenLatestTxInfo to %s", string(latestInfoBytes)))
	ctx.PutStateByte(tokenInfoMapName, "_latestTxInfo", latestInfoBytes)
}
func testgasJsonMarshal_stub3(from, to string) {
	ctx := sdk.NewSimContext()
	tkTxInfo := &tokenLatestTxInfo{
		TxId:        "TX_ID",
		BlockHeight: 111,
		From:        from,
		To:          to,
	}
	//latestInfoBytes, _ := json.Marshal(tkTxInfo)
	latestInfoBytes := []byte("")
	_ = string(latestInfoBytes)
	_ = tkTxInfo
	ctx.Log(fmt.Sprintf("setTokenLatestTxInfo to %s", string(latestInfoBytes)))
	ctx.PutStateByte(tokenInfoMapName, "_latestTxInfo", latestInfoBytes)
}
func testgasJsonMarshal_stub2(from, to string) {
	ctx := sdk.NewSimContext()
	tkTxInfo := &tokenLatestTxInfo{
		TxId:        "TX_ID",
		BlockHeight: 111,
		From:        "0000000000000000000000000000000000000000",
		To:          "8acfaca5eeec9f6f7c23c4ffac969b86f27799b0",
	}
	latestInfoBytes, _ := json.Marshal(tkTxInfo)
	_ = string(latestInfoBytes)
	ctx.Log(fmt.Sprintf("setTokenLatestTxInfo to %s", string(latestInfoBytes)))
	ctx.PutStateByte(tokenInfoMapName, "_latestTxInfo", latestInfoBytes)
}
func increaseTokenCountByOne(account string) error {
	ctx := sdk.NewSimContext()
	originTokenCount, err := getBalance(account)
	if err != nil {
		return fmt.Errorf("get token count failed, err")
	}
	newTokenCount, ok := safemath.SafeAdd(originTokenCount, safemath.SafeUintOne)
	if !ok {
		return fmt.Errorf("balance count of from is overflow")
	}
	err1 := ctx.PutStateByte(balanceInfoMapName, account, []byte(newTokenCount.ToString()))
	if err1 != sdk.SUCCESS {
		return fmt.Errorf("increaseTokenCountByOne failed")
	}
	return nil
}

func decreaseTokenCountByOne(account string) error {
	ctx := sdk.NewSimContext()
	originTokenCount, err := getBalance(account)
	if err != nil {
		return fmt.Errorf("get token count failed, err")
	}
	newTokenCount, ok := safemath.SafeSub(originTokenCount, safemath.SafeUintOne)
	if !ok {
		return fmt.Errorf("balance count of from is overflow")
	}
	err1 := ctx.PutStateByte(balanceInfoMapName, account, []byte(newTokenCount.ToString()))
	if err1 != sdk.SUCCESS {
		return fmt.Errorf("decreaseTokenCountByOne failed")
	}
	return nil
}

func setTokenOwner(to string, tokenId *safemath.SafeUint256) error {
	ctx := sdk.NewSimContext()
	err := ctx.PutState(tokenOwnerMapName, tokenId.ToString(), to)
	if err != sdk.SUCCESS {
		return fmt.Errorf("set token owner failed, err")
	}
	return nil
}
func setAccountToken(from, to string, tokenId *safemath.SafeUint256) error {
	ctx := sdk.NewSimContext()
	err := ctx.PutState(accountMapName, to+"_"+tokenId.ToString(), trueString)
	if err != sdk.SUCCESS {
		return fmt.Errorf("setAccountToken failed, err")
	}
	if address.IsZeroAddress(from) {
		return nil
	}
	err = ctx.DeleteState(accountMapName, from+"_"+tokenId.ToString())
	if err != sdk.SUCCESS {
		return fmt.Errorf("setAccountToken failed, err")
	}
	return nil
}

// 铸造Token
//
//go:wasmexport mint
func mint() {
	ctx := sdk.NewSimContext()
	to, _ := ctx.ArgString("to")
	metadata, _ := ctx.ArgString("metadata")
	tokenIdStr, _ := ctx.ArgString("tokenId")
	originTokenId := tokenIdStr
	tokenId, ok := safemath.ParseSafeUint256(tokenIdStr)
	if !ok {
		ctx.ErrorResult("Parse tokenId failed")
		return
	}
	if !address.IsValidAddress(to) {
		ctx.ErrorResult("mint to invalid address")
		return
	}
	if address.IsZeroAddress(to) {
		ctx.ErrorResult("ERC721: mint to the zero address")
		return
	}

	minted, err1 := minted(tokenId)
	if err1 != nil {
		ctx.ErrorResult(err1.Error())
		return
	}
	if minted {
		ctx.ErrorResult(fmt.Sprintf("%s %v duplicated token", originTokenId, tokenId))
		return
	}
	sender, err := ctx.GetSenderPk()
	if err != sdk.SUCCESS {
		ctx.ErrorResult(fmt.Sprintf("get origin failed, err:%s", sender))
		return
	}

	_ = metadata
	admin, err := ctx.GetState(erc721InfoMapName, "admin")
	_ = admin
	if err != sdk.SUCCESS {
		ctx.ErrorResult(fmt.Sprintf("get admin from erc721Info failed, err:%s", err))
		return
	}

	//删除这个限制
	//if sender != string(admin) {
	//	ctx.ErrorResult("only admin can mint tokens")
	//return
	//}

	err1 = increaseTokenCountByOne(to)
	if err1 != nil {
		ctx.ErrorResult(err1.Error())
		return
	}
	err1 = setTokenOwner(to, tokenId)
	if err1 != nil {
		ctx.ErrorResult(err1.Error())
		return
	}

	err1 = setAccountToken(address.ZeroAddr, to, tokenId)
	if err1 != nil {
		ctx.ErrorResult(err1.Error())
		return
	}

	err1 = setMetadata(tokenId, metadata)
	if err1 != nil {
		ctx.ErrorResult(err1.Error())
		return
	}

	err1 = setTokenLatestTxInfo(tokenId, address.ZeroAddr, to)
	if err1 != nil {
		ctx.ErrorResult(err1.Error())
		return
	}
	ctx.SuccessResult("mint success")
	return
}

func setMetadata(tokenId *safemath.SafeUint256, metadata string) error {
	ctx := sdk.NewSimContext()
	if len(metadata) > 0 {
		err := ctx.PutState(tokenInfoMapName, tokenId.ToString()+"_metadata", metadata)
		if err != sdk.SUCCESS {
			return fmt.Errorf("set metadata of erc721Info failed, err:%s", err)
		}
	}
	return nil
}

func minted(tokenId *safemath.SafeUint256) (bool, error) {
	ctx := sdk.NewSimContext()
	owner, err := ctx.GetState(tokenOwnerMapName, tokenId.ToString())
	if err != sdk.SUCCESS {
		ctx.ErrorResult("get owner failed")
		return false, nil
	}

	return len(owner) > 0 && !address.IsZeroAddress(owner), nil
}

// 查询元数据
//
//go:wasmexport tokenURI
func tokenURI() {
	ctx := sdk.NewSimContext()
	tokenIdStr, _ := ctx.ArgString("tokenId")
	tokenId, ok := safemath.ParseSafeUint256(tokenIdStr)
	if !ok {
		ctx.ErrorResult("Parse tokenId failed")
		return
	}
	baseURI, _ := ctx.GetState(erc721InfoMapName, "tokenURI")
	if len(baseURI) == 0 {
		ctx.SuccessResult("")
		return
	}
	ctx.SuccessResult(baseURI + "/" + tokenId.ToString())
	return
}

//go:wasmexport tokenMetadata
func tokenMetadata() {
	ctx := sdk.NewSimContext()
	tokenIdStr, _ := ctx.ArgString("tokenId")
	tokenId, ok := safemath.ParseSafeUint256(tokenIdStr)
	if !ok {
		ctx.ErrorResult("Parse tokenId failed")
		return
	}
	metadata, err := ctx.GetState(tokenInfoMapName, tokenId.ToString()+"_metadata")
	if err != sdk.SUCCESS || len(metadata) == 0 {
		ctx.ErrorResult("get metadata of erc721Info failed")
		return
	}
	ctx.SuccessResult(fmt.Sprintf("tokenMetadata is %s", metadata))
	return
}

//go:wasmexport name
func name() {
	ctx := sdk.NewSimContext()

	name, err := ctx.GetState(erc721InfoMapName, "name")
	if err != sdk.SUCCESS || len(name) == 0 {
		ctx.ErrorResult("get name from erc721Info failed, no name")
		return
	}
	ctx.SuccessResult("[name] name=" + string(name))
	return
}

//go:wasmexport symbol
func symbol() {
	ctx := sdk.NewSimContext()

	symbol, err := ctx.GetState(erc721InfoMapName, "symbol")
	if err != sdk.SUCCESS || len(symbol) == 0 {
		ctx.ErrorResult("get symbol from erc721Info failed, no symbol")
		return
	}
	ctx.SuccessResult("[symbol] symbol=" + string(symbol))
	return
}

//go:wasmexport accountTokens
func getAccountTokens() {

	ctx := sdk.NewSimContext()
	account, _ := ctx.ArgString("account")
	if !address.IsValidAddress(account) {
		ctx.ErrorResult("invalid account")
		return
	}
	rs, _ := ctx.NewIteratorPrefixWithKeyField(accountMapName, account)
	ats := &accountTokens{
		Account: account,
		Tokens:  make([]string, 0),
	}
	for {

		if !rs.HasNext() {
			break
		}
		_, field, _, err := rs.Next()
		if err != sdk.SUCCESS {
			ctx.ErrorResult(fmt.Sprintf("iterator next failed, err: %s", err))
			return
		}
		itemId := strings.Split(field, "_")[1]
		if len(itemId) == 0 {
			ctx.ErrorResult("invalid itemId")
			return
		}
		ats.Tokens = append(ats.Tokens, itemId)
	}
	var atsBytes []byte
	atsBytes, err1 := json.Marshal(ats)
	if err1 != nil {
		ctx.ErrorResult(err1.Error())
	}
	ctx.SuccessResult(string(atsBytes))

}

//go:wasmexport tokenLatestTxInfo
func getTokenLatestTxInfo() {
	ctx := sdk.NewSimContext()
	tokenIdStr, _ := ctx.ArgString("tokenId")
	tokenId, ok := safemath.ParseSafeUint256(tokenIdStr)
	if !ok {
		ctx.ErrorResult("Parse tokenId failed")
		return
	}
	latestTxInfo, err := ctx.GetState(tokenInfoMapName, tokenId.ToString()+"_latestTxInfo")
	if err != sdk.SUCCESS || len(latestTxInfo) == 0 {
		ctx.ErrorResult("get latest tx info failed, err")
		return
	}
	ctx.SuccessResult(fmt.Sprintf("tokenLatestTxInfo is %s", latestTxInfo))
	return
}

//go:wasmexport getGasRemaining
func getGasRemaining() {
	ctx := sdk.NewSimContext()

	ctx.SuccessResult("666")
}
func main() {}
