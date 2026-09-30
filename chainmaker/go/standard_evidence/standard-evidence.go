/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package main

import (
	"chainmaker.org/chainmaker/contract-utils/standard"
	"chainmaker.org/chainmaker/contract-utils/str"
	"encoding/json"
	"errors"
	"github.com/TKOTKCh/contract-sdk-go-wasm/sdk"
	"strconv"
)

const (
	paramId                   = "id"
	paramHash                 = "hash"
	paramMetadata             = "metadata"
	paramEvidences            = "evidences"
	paramStandardName         = "standardName"
	methodPreEvidence         = "[evidence]"
	methodPreExistsOfHash     = "[existsOfHash]"
	methodPreExistsOfId       = "[existsOfId]"
	methodPreFindByHash       = "[findByHash]"
	methodPreFindById         = "[findById]"
	methodPreEvidenceBatch    = "[evidenceBatch]"
	methodPreStandards        = "[standards]"
	methodPreSupportsStandard = "[supportsStandard]"

	keyEvidenceHash = "h"
	keyEvidenceId   = "i"
)

//go:wasmexport init_contract
func InitContract() {
	ctx := sdk.NewSimContext()
	ctx.SuccessResult("Init contract success")
}

//go:wasmexport upgrade
func Upgrade() {
	ctx := sdk.NewSimContext()
	ctx.SuccessResult("Upgrade contract success")
}

///////////////////////// 核心业务函数 /////////////////////////

//go:wasmexport Evidence
func evidenceCore() {
	ctx := sdk.NewSimContext()

	// 获取参数
	id, _ := ctx.ArgString(paramId)
	hash, _ := ctx.ArgString(paramHash)
	metadata, _ := ctx.ArgString(paramMetadata)
	err := Evidence(id, hash, metadata)
	if err != nil {
		ctx.ErrorResult(methodPreEvidence + err.Error())
		return
	}

	ctx.SuccessResult("evidence saved")
}

//go:wasmexport EvidenceBatch
func evidenceBatchCore() {
	ctx := sdk.NewSimContext()
	evidencesParam, _ := ctx.ArgString(paramEvidences)

	if str.IsAnyBlank(evidencesParam) {
		ctx.ErrorResult(methodPreEvidenceBatch + "evidences is empty")
		return
	}
	evidences := make([]standard.Evidence, 0)
	err := json.Unmarshal([]byte(evidencesParam), &evidences)
	if err != nil {
		ctx.ErrorResult(methodPreEvidenceBatch + err.Error())
		return
	}

	// 存证
	err = EvidenceBatch(evidences)
	if err != nil {
		ctx.ErrorResult(methodPreEvidenceBatch + err.Error())
		return
	}
	ctx.SuccessResult("ok")
}

//go:wasmexport ExistsOfHash
func existsOfHashCore() {

	ctx := sdk.NewSimContext()
	hash, _ := ctx.ArgString(paramHash)

	if str.IsAnyBlank(hash) {
		ctx.ErrorResult(methodPreExistsOfHash + "hash cannot be empty")
		return
	}

	if exists, err := ExistsOfHash(hash); err != nil {
		ctx.ErrorResult(methodPreExistsOfHash + err.Error())
		return
	} else if exists {
		ctx.SuccessResult(standard.TrueString)
		return
	} else {
		ctx.SuccessResult(standard.FalseString)
		return
	}
}

//go:wasmexport ExistsOfId
func existsOfIdCore() {
	ctx := sdk.NewSimContext()
	id, _ := ctx.ArgString(paramId)

	if str.IsAnyBlank(id) {
		ctx.ErrorResult(methodPreExistsOfId + "id cannot be empty")
		return
	}

	if exists, err := ExistsOfId(id); err != nil {
		ctx.ErrorResult(methodPreExistsOfId + err.Error())
		return
	} else if exists {
		ctx.SuccessResult(standard.TrueString)
		return
	} else {
		ctx.SuccessResult(standard.FalseString)
		return
	}
}

//go:wasmexport FindByHash
func findByHashCore() {

	ctx := sdk.NewSimContext()
	hash, _ := ctx.ArgString(paramHash)

	if str.IsAnyBlank(hash) {
		ctx.ErrorResult("hash cannot be empty")
		return
	}

	evidence, err := FindByHash(hash)
	if err != nil {
		ctx.ErrorResult(methodPreFindByHash + err.Error())
		return
	}

	data, err := json.Marshal(evidence)
	if err != nil {
		ctx.ErrorResult(methodPreFindByHash + err.Error())
		return
	}
	ctx.SuccessResult(string(data))
}

//go:wasmexport evidenceAndFindByHash
func evidenceAndFindByHashCore() {
	ctx := sdk.NewSimContext()
	evidencesParam, _ := ctx.ArgString(paramEvidences)

	if str.IsAnyBlank(evidencesParam) {
		ctx.ErrorResult(methodPreEvidenceBatch + "evidences is empty")
		return
	}
	evidences := make([]standard.Evidence, 0)
	err := json.Unmarshal([]byte(evidencesParam), &evidences)
	if err != nil {
		ctx.ErrorResult(methodPreEvidenceBatch + err.Error())
		return
	}

	// 存证
	err = EvidenceBatch(evidences)
	if err != nil {
		ctx.ErrorResult(methodPreEvidenceBatch + err.Error())
		return
	}
	ctx.SuccessResult("ok")

	hash, _ := ctx.ArgString(paramHash)

	if str.IsAnyBlank(hash) {
		ctx.ErrorResult("hash cannot be empty")
		return
	}

	evidence, err := FindByHash(hash)
	if err != nil {
		ctx.ErrorResult(methodPreFindByHash + err.Error())
		return
	}

	data, err := json.Marshal(evidence)
	if err != nil {
		ctx.ErrorResult(methodPreFindByHash + err.Error())
		return
	}
	ctx.SuccessResult(string(data))
}

//go:wasmexport testgasNewctx
func newctx() {
	ctx := sdk.NewSimContext()
	evidencesParam, _ := ctx.ArgString(paramEvidences)
	_ = evidencesParam
}

//go:wasmexport testgasUnmarshal
func testgasUnmarshal() {
	ctx := sdk.NewSimContext()
	evidencesParam, _ := ctx.ArgString(paramEvidences)

	if str.IsAnyBlank(evidencesParam) {
		ctx.ErrorResult(methodPreEvidenceBatch + "evidences is empty")
		return
	}
	evidences := make([]standard.Evidence, 0)
	err := json.Unmarshal([]byte(evidencesParam), &evidences)
	if err != nil {
		ctx.ErrorResult(methodPreEvidenceBatch + err.Error())
		return
	}
}

//go:wasmexport testgasEvidence
func testgasEvidence() {
	ctx := sdk.NewSimContext()
	evidencesParam, _ := ctx.ArgString(paramEvidences)

	if str.IsAnyBlank(evidencesParam) {
		ctx.ErrorResult(methodPreEvidenceBatch + "evidences is empty")
		return
	}
	evidences := make([]standard.Evidence, 0)
	err := json.Unmarshal([]byte(evidencesParam), &evidences)
	if err != nil {
		ctx.ErrorResult(methodPreEvidenceBatch + err.Error())
		return
	}

	// 存证
	err = EvidenceBatch(evidences)
	if err != nil {
		ctx.ErrorResult(methodPreEvidenceBatch + err.Error())
		return
	}
	ctx.SuccessResult("ok")
}

//go:wasmexport testgasMake
func testgasMake() {
	ctx := sdk.NewSimContext()
	evidencesParam, _ := ctx.ArgString(paramEvidences)
	evidences := make([]standard.Evidence, 0)
	_ = evidencesParam
	_ = evidences
	ctx.SuccessResult("ok")
}

//go:wasmexport testgasMakeUnmarshal
func testgasMakeUnmarshal() {
	ctx := sdk.NewSimContext()
	evidencesParam, _ := ctx.ArgString(paramEvidences)
	evidences := make([]standard.Evidence, 0)
	err := json.Unmarshal([]byte(evidencesParam), &evidences)
	_ = err
	ctx.SuccessResult("ok")
}

////go:wasmexport testgasJSONMarhal
//func testgasJSONMarhal() {
//	ctx := sdk.NewSimContext()
//	p := Person{
//		Name:    "Alice",
//		Age:     25,
//		IsAdmin: true,
//	}
//
//	// 将结构体序列化为 JSON 字节切片
//	jsonData, err := json.Marshal(p)
//	if err != nil {
//		ctx.ErrorResult("序列化失败")
//		return
//	}
//	ctx.SuccessResult(string(jsonData))
//}

//go:wasmexport testgasWarmUp
func testgasWarmUp() {
	type tstruct struct {
	}
	jsonStr := `{}`
	var p tstruct
	// 将 JSON 字符串解析到结构体
	json.Unmarshal([]byte(jsonStr), &p)
	json.Marshal(p)
	strconv.ParseInt("1", 10, 64)
}

//go:wasmexport FindById
func findByIdCore() {
	ctx := sdk.NewSimContext()

	id, _ := ctx.ArgString(paramId)
	if str.IsAnyBlank(id) {
		ctx.ErrorResult(methodPreFindById + "id cannot be empty")
		return
	}

	evidence, err := FindById(id)
	if err != nil {
		ctx.ErrorResult(methodPreFindById + err.Error())
		return
	}

	data, err := json.Marshal(evidence)
	if err != nil {
		ctx.ErrorResult(methodPreFindById + err.Error())
		return
	}
	ctx.SuccessResult(string(data))
	return
}

//go:wasmexport Standards
func standardsCore() {
	ctx := sdk.NewSimContext()
	data, err := json.Marshal(Standards())
	if err != nil {
		ctx.ErrorResult(methodPreStandards + err.Error())
		return
	}
	ctx.SuccessResult(string(data))
	return
}

//go:wasmexport SupportStandard
func supportStandardCore() {
	ctx := sdk.NewSimContext()

	standardName, _ := ctx.ArgString(paramStandardName)
	if str.IsAnyBlank(standardName) {
		ctx.ErrorResult(methodPreSupportsStandard + "standardName cannot be empty")
		return
	}

	if SupportStandard(standardName) {
		ctx.SuccessResult(standard.TrueString)
		return
	} else {
		//注意这里SuccessResult不可以在else外面，因为successResult会覆盖，最终的successResult是最后一个执行的语句，或者直接return
		//ErrorResult不会覆盖，但是会累加？
		ctx.SuccessResult(standard.FalseString)
		return
	}
}

// 这个函数不暴露，辅助evidenceCore
func Evidence(id string, hash string, metadata string) error {
	ctx := sdk.NewSimContext()
	// 校验是否存在
	//existsHash, err := ctx.GetStateByte(keyEvidenceHash, hash)
	//if err != sdk.SUCCESS {
	//	return errors.New("get evidence hash failed")
	//}
	//if len(existsHash) > 0 {
	//	return errors.New("hash already exists: " + hash)
	//}
	//
	//existsId, err := ctx.GetStateByte(keyEvidenceId, id)
	//if err != sdk.SUCCESS {
	//	return errors.New("get evidence id failed")
	//}
	//if len(existsId) > 0 {
	//	return errors.New("id already exists: " + id)
	//}

	// 获取区块信息, 构建存证对象
	txId, _ := ctx.GetTxId()
	blockHeightStr, _ := ctx.GetBlockHeight()
	blockHeight, _ := strconv.ParseInt(blockHeightStr, 10, 0)
	evidence := standard.Evidence{
		Id:          id,
		TxId:        txId,
		Hash:        hash,
		BlockHeight: int(blockHeight),
		Metadata:    metadata,
	}

	data, err1 := json.Marshal(&evidence)
	if err1 != nil {
		return err1
	}

	err := ctx.PutStateByte(keyEvidenceHash, hash, data)
	if err != sdk.SUCCESS {
		return errors.New("put evidence hash failed")
	}
	err = ctx.PutStateByte(keyEvidenceId, id, []byte(hash))
	if err != sdk.SUCCESS {
		return errors.New("put evidence id failed")
	}

	return nil
}

// 这个函数不暴露 批量存证
func EvidenceBatch(evidences []standard.Evidence) error {
	for i := range evidences {
		err := Evidence(evidences[i].Id, evidences[i].Hash, evidences[i].Metadata)
		if err != nil {
			return err
		}
	}
	return nil
}

// ExistsOfHash 哈希是否存在
func ExistsOfHash(hash string) (bool, error) {
	ctx := sdk.NewSimContext()
	existsHash, err := ctx.GetStateByte(keyEvidenceHash, hash)
	if err != sdk.SUCCESS {
		return len(existsHash) > 0, nil
	} else {
		return len(existsHash) > 0, errors.New("hash isn't exists")
	}
}

// ExistsOfId ID是否存在
func ExistsOfId(id string) (bool, error) {
	ctx := sdk.NewSimContext()
	existsId, err := ctx.GetStateByte(keyEvidenceId, id)
	if err != sdk.SUCCESS {
		return len(existsId) > 0, nil
	} else {
		return len(existsId) > 0, errors.New("id isn't exists")
	}

}

// FindByHash 根据哈希查找
func FindByHash(hash string) (*standard.Evidence, error) {
	ctx := sdk.NewSimContext()
	data, err := ctx.GetStateByte(keyEvidenceHash, hash)
	if err != sdk.SUCCESS {
		return nil, errors.New("get evidence failed")
	}
	evi := &standard.Evidence{}
	err1 := json.Unmarshal(data, evi)
	return evi, err1
}

// FindById 根据id查找
func FindById(id string) (*standard.Evidence, error) {
	ctx := sdk.NewSimContext()
	hash, err := ctx.GetState(keyEvidenceId, id)
	if err != sdk.SUCCESS {
		return nil, errors.New("get evidence failed")
	}
	evidenceByte, err := ctx.GetStateByte(keyEvidenceHash, hash)
	if err != sdk.SUCCESS {
		return nil, errors.New("get evidence failed")
	}

	evi := &standard.Evidence{}
	err1 := json.Unmarshal(evidenceByte, evi)
	return evi, err1
}

// Standards  获取当前合约支持的标准协议列表
func Standards() []string {
	return []string{standard.ContractStandardNameCMEVI, standard.ContractStandardNameCMBC}
}

// SupportStandard  获取当前合约是否支持某合约标准协议
func SupportStandard(standardName string) bool {
	return standardName == standard.ContractStandardNameCMEVI || standardName == standard.ContractStandardNameCMBC
}
func main() {

}
