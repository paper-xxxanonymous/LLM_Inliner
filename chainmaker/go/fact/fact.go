/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package main

import (
	"encoding/json"
	"fmt"
	"github.com/TKOTKCh/contract-sdk-go-wasm/sdk"
	"strconv"
)

// 安装合约时会执行此方法，必须
//
//go:wasmexport init_contract
func initContract() {
	// 此处可写安装合约的初始化逻辑

}

// 升级合约时会执行此方法，必须
//
//go:wasmexport upgrade
func upgrade() {
	// 此处可写升级合约的逻辑

}

// 存证对象
type Fact struct {
	fileHash string
	fileName string
	time     int32 // second
	ec       *sdk.EasyCodec
}

// 新建存证对象
func NewFact(fileHash string, fileName string, time int32) *Fact {
	fact := &Fact{
		fileHash: fileHash,
		fileName: fileName,
		time:     time,
	}
	return fact
}

// 获取序列化对象
func (f *Fact) getEasyCodec() *sdk.EasyCodec {
	if f.ec == nil {
		f.ec = sdk.NewEasyCodec()
		f.ec.AddString("fileHash", f.fileHash)
		f.ec.AddString("fileName", f.fileName)
		f.ec.AddInt32("time", f.time)
	}
	return f.ec
}

// 序列化为json字符串
func (f *Fact) toJson() string {
	return f.getEasyCodec().ToJson()
}

// 序列化为cmec编码
func (f *Fact) marshal() []byte {
	return f.getEasyCodec().Marshal()
}

// 反序列化cmec为存证对象
func unmarshalToFact(data []byte) *Fact {
	ec := sdk.NewEasyCodecWithBytes(data)
	fileHash, _ := ec.GetString("fileHash")
	fileName, _ := ec.GetString("fileName")
	time, _ := ec.GetInt32("time")

	fact := &Fact{
		fileHash: fileHash,
		fileName: fileName,
		time:     time,
		ec:       ec,
	}
	return fact
}

// 对外暴露 save 方法，供用户由 SDK 调用
//
//go:wasmexport save
func save() {
	// 获取上下文
	ctx := sdk.NewSimContext()

	// 获取参数
	fileHash, err1 := ctx.ArgString("file_hash")
	fileName, err2 := ctx.ArgString("file_name")
	timeStr, err3 := ctx.ArgString("time")

	if err1 != sdk.SUCCESS || err2 != sdk.SUCCESS || err3 != sdk.SUCCESS {
		ctx.Log("get arg fail.")
		ctx.ErrorResult("get arg fail.")
		return
	}

	time, err := strconv.ParseInt(timeStr, 10, 32)
	if err != nil {
		ctx.ErrorResult(err.Error())
		ctx.Log(err.Error())
		return
	}

	// 构建结构体
	fact := NewFact(fileHash, fileName, int32(time))

	// 序列化：两种方式
	jsonStr := fact.toJson()
	bytesData := fact.marshal()

	//发送事件
	ctx.EmitEvent("topic_vx", fact.fileHash, fact.fileName)

	// 存储数据
	ctx.PutState("fact_json", fact.fileHash, jsonStr)
	ctx.PutStateByte("fact_bytes", fact.fileHash, bytesData)

	// 记录日志
	ctx.Log("【save】 fileHash=" + fact.fileHash)
	ctx.Log("【save】 fileName=" + fact.fileName)
	// 返回结果
	ctx.SuccessResult(fact.fileName + fact.fileHash)
}

// 对外暴露 find_by_file_hash 方法，供用户由 SDK 调用
//
//go:wasmexport findByFileHash
func findByFileHash() {
	ctx := sdk.NewSimContext()
	// 获取参数
	fileHash, _ := ctx.ArgString("file_hash")
	// 查询Json
	if result, resultCode := ctx.GetStateByte("fact_json", fileHash); resultCode != sdk.SUCCESS {
		// 返回结果
		ctx.ErrorResult("failed to call get_state, only 64 letters and numbers are allowed. got key:" + "fact" + ", field:" + fileHash)
	} else {
		// 返回结果
		ctx.SuccessResultByte(result)
		// 记录日志
		ctx.Log("get val:" + string(result))
	}

	// 查询EcBytes
	if result, resultCode := ctx.GetStateByte("fact_bytes", fileHash); resultCode == sdk.SUCCESS {
		// 反序列化
		fact := unmarshalToFact(result)
		// 返回结果
		ctx.SuccessResult(fact.toJson())
		// 记录日志
		ctx.Log("get val:" + fact.toJson())
		ctx.Log("【find_by_file_hash】 fileHash=" + fact.fileHash)
		ctx.Log("【find_by_file_hash】 fileName=" + fact.fileName)
	}
}

//go:wasmexport saveAndFindByFileHash
func saveAndFindByFileHash() {
	// 获取上下文
	ctx := sdk.NewSimContext()

	// 获取参数
	fileHash, err1 := ctx.ArgString("file_hash")
	fileName, err2 := ctx.ArgString("file_name")
	timeStr, err3 := ctx.ArgString("time")

	if err1 != sdk.SUCCESS || err2 != sdk.SUCCESS || err3 != sdk.SUCCESS {
		ctx.Log("get arg fail.")
		ctx.ErrorResult("get arg fail.")
		return
	}

	time, err := strconv.ParseInt(timeStr, 10, 32)
	if err != nil {
		ctx.ErrorResult(err.Error())
		ctx.Log(err.Error())
		return
	}

	// 构建结构体
	fact := NewFact(fileHash, fileName, int32(time))

	// 序列化：两种方式

	factBytes, err := json.Marshal(fact)
	if err != nil {
		ctx.ErrorResult(fmt.Sprintf("marshal fact failed, err: %s", err))
		return
	}

	//发送事件
	ctx.EmitEvent("topic_vx", fact.fileHash, fact.fileName)

	// 存储数据
	ctx.PutStateByte("fact_bytes", fact.fileHash, factBytes)

	// 记录日志
	ctx.Log("【save】 fileHash=" + fact.fileHash)
	ctx.Log("【save】 fileName=" + fact.fileName)

	// 获取参数
	fileHash, _ = ctx.ArgString("file_hash")

	// 查询EcBytes
	result, resultCode := ctx.GetStateByte("fact_bytes", fileHash)
	if resultCode != sdk.SUCCESS {
		ctx.ErrorResult("failed to call get_state")
		return
	}

	// 反序列化
	if err = json.Unmarshal(result, &fact); err != nil {
		ctx.ErrorResult(fmt.Sprintf("unmarshal fact failed, err: %s", err))
		return
	}

	// 记录日志
	ctx.Log("get val:" + fact.toJson())
	ctx.Log("【find_by_file_hash】 fileHash=" + fact.fileHash)
	ctx.Log("【find_by_file_hash】 fileName=" + fact.fileName)
	// 返回结果
	ctx.SuccessResult(fact.toJson())

}

func main() {

}
