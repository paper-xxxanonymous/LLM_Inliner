/*
	Copyright (C) BABEC. All rights reserved. Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package main

import (
	"encoding/json"
	"fmt"
	"github.com/TKOTKCh/contract-sdk-go-wasm/sdk"
	"strings"
	"time"
)

// Itinerary 行程数据结构
type Itinerary struct {
	IP       string  `json:"ip"`
	City     string  `json:"city"`
	Region   string  `json:"region"`
	Country  string  `json:"country"`
	Loc      string  `json:"loc"`
	Org      string  `json:"org"`
	Timezone string  `json:"timezone"`
	Asn      Asn     `json:"asn"`
	Company  Company `json:"company"`
	Privacy  Privacy `json:"privacy"`
	Abuse    Abuse   `json:"abuse"`
	Domains  Domains `json:"domains"`
}

type Asn struct {
	Asn    string `json:"asn"`
	Name   string `json:"name"`
	Domain string `json:"domain"`
	Route  string `json:"route"`
	Type   string `json:"type"`
}
type Company struct {
	Name   string `json:"name"`
	Domain string `json:"domain"`
	Type   string `json:"type"`
}
type Privacy struct {
	Vpn     bool   `json:"vpn"`
	Proxy   bool   `json:"proxy"`
	Tor     bool   `json:"tor"`
	Relay   bool   `json:"relay"`
	Hosting bool   `json:"hosting"`
	Service string `json:"service"`
}
type Abuse struct {
	Address string `json:"address"`
	Country string `json:"country"`
	Email   string `json:"email"`
	Name    string `json:"name"`
	Network string `json:"network"`
	Phone   string `json:"phone"`
}
type Domains struct {
	Total   int           `json:"total"`
	Domains []interface{} `json:"domains"`
}

// HistoryValue 针对key的历史记录定义
type HistoryValue struct {
	Field       string      `json:"field"`
	Value       interface{} `json:"value"`
	TxId        string      `json:"txId"`
	Timestamp   string      `json:"timestamp"`
	BlockHeight int         `json:"blockHeight"`
	Key         string      `json:"key"`
}

type Result struct {
	Code int32       `json:"code"`
	Msg  string      `json:"msg"`
	Data interface{} `json:"data"`
}

func ToResult(data interface{}) []byte {
	dataRet := &Result{
		Code: 200,
		Msg:  "",
		Data: data,
	}
	res, err := json.Marshal(dataRet)
	if err != nil {
		res = []byte("json marshal error")
	}
	return res
}

// 安装合约初始化
//
//go:wasmexport init_contract
func InitContract() {
	ctx := sdk.NewSimContext()
	ctx.SuccessResult("Init contract success")
}

// 升级合约
//
//go:wasmexport upgrade
func Upgrade() {
	ctx := sdk.NewSimContext()
	ctx.SuccessResult("Upgrade contract success")
}

// 保存行程数据
//
//go:wasmexport save
func save() {
	ctx := sdk.NewSimContext()

	// 获取参数
	itineraryStr, _ := ctx.ArgString("itinerary")
	phone, _ := ctx.ArgString("phone")

	// 参数校验
	if isBlank(itineraryStr) {
		ctx.ErrorResult("'itinerary' should not be empty!")
	}
	if isBlank(phone) {
		ctx.ErrorResult("'phone' should not be empty!")
	}
	// 解析行程数据
	var itinerary Itinerary
	err1 := json.Unmarshal([]byte(itineraryStr), &itinerary)
	if err1 != nil {
		errMsg := fmt.Sprintf("unmarshal itinerary data failed, %s", err1)
		ctx.ErrorResult(errMsg)
	}

	// 关键字段校验
	if isBlank(itinerary.IP) || isBlank(itinerary.Country) ||
		isBlank(itinerary.City) || isBlank(itinerary.Region) {
		errMsg := fmt.Sprintf("'ip','country','city','region' should not be empty, %s", err1)
		ctx.ErrorResult(errMsg)
	}

	// 存储数据
	err := ctx.PutStateByte(phone, "", []byte(itineraryStr))
	if err != sdk.SUCCESS {
		ctx.ErrorResult("save data failed")
	}

	// 触发事件
	ctx.EmitEvent("save", phone, itineraryStr)
	ctx.SuccessResult(string(ToResult(itinerary)))
}

// 查询历史记录
// 此方法需要依赖 chainmaker.yml 的 history db
// 且需要使用NewHistoryKvIterForKey
//
//go:wasmexport queryHistory
func queryHistory() {
	save()
	ctx := sdk.NewSimContext()
	phone, _ := ctx.ArgString("phone")

	if isBlank(phone) {
		ctx.ErrorResult("phone cannot be empty")
	}
	//缺少NewHistoryKvIterForKey
	iter, err := ctx.NewHistoryKvIterForKey(phone, "")
	if err != sdk.SUCCESS {
		errMsg := fmt.Sprintf("new HistoryKvIter for key=[%s] failed, %s", phone, err)
		ctx.ErrorResult(errMsg)
		return
	}
	var itinerary Itinerary
	recordMap := make(map[string]HistoryValue, 0)
	for iter.HasNext() {
		km, err := iter.Next()

		if err != sdk.SUCCESS {
			errMsg := "iterator failed to get the next element"
			ctx.ErrorResult(errMsg)
			// 避免出现EOF，暂时跳过
			continue
		}

		err1 := json.Unmarshal(km.Value, &itinerary)
		if err1 != nil {
			errMsg := "json parse element error" + "," + err1.Error()
			ctx.ErrorResult(errMsg)
			continue
		}

		hv := &HistoryValue{
			TxId:        km.TxId,
			Timestamp:   km.Timestamp,
			BlockHeight: km.BlockHeight,
			Key:         km.Key,
			Field:       km.Field,
			Value:       itinerary,
		}
		location := itinerary.Country + "-" + itinerary.City + "-" + itinerary.Region
		if record, ok := recordMap[location]; !ok {
			recordMap[location] = *hv
		} else {
			// 只要最新行踪记录
			if record.BlockHeight < km.BlockHeight {
				recordMap[location] = *hv
			}
		}
	}

	closed, err := iter.Close()
	if !closed || err != sdk.SUCCESS {
		errMsg := fmt.Sprintf("iterator close failed")
		ctx.ErrorResult(errMsg)
	}
	ctx.SuccessResult(string(ToResult(recordMap)))
}

//go:wasmexport saveAndQueryHistory
func saveAndQueryHistory() {
	ctx := sdk.NewSimContext()

	// 获取参数
	itineraryStr, _ := ctx.ArgString("itinerary")
	phone, _ := ctx.ArgString("phone")

	// 参数校验
	if isBlank(itineraryStr) {
		ctx.ErrorResult("'itinerary' should not be empty!")
	}
	if isBlank(phone) {
		ctx.ErrorResult("'phone' should not be empty!")
	}
	// 解析行程数据
	var saveItinerary Itinerary
	err1 := json.Unmarshal([]byte(itineraryStr), &saveItinerary)
	if err1 != nil {
		errMsg := fmt.Sprintf("unmarshal itinerary data failed, %s", err1)
		ctx.ErrorResult(errMsg)
	}

	// 关键字段校验
	if isBlank(saveItinerary.IP) || isBlank(saveItinerary.Country) ||
		isBlank(saveItinerary.City) || isBlank(saveItinerary.Region) {
		errMsg := fmt.Sprintf("'ip','country','city','region' should not be empty, %s", err1)
		ctx.ErrorResult(errMsg)
	}

	// 存储数据
	//temp, err := ctx.GetState(phone, "")
	//if err != sdk.SUCCESS || len(temp) == 0 {
	//	err := ctx.PutStateByte(phone, "", []byte(itineraryStr))
	//	if err != sdk.SUCCESS {
	//		ctx.ErrorResult("save data failed")
	//	}
	//}
	err := ctx.PutStateByte(phone, "", []byte(itineraryStr))
	if err != sdk.SUCCESS {
		ctx.ErrorResult("save data failed")
	}
	// 触发事件
	ctx.EmitEvent("save", phone, itineraryStr)

	phone, _ = ctx.ArgString("phone")

	if isBlank(phone) {
		ctx.ErrorResult("phone cannot be empty")
	}
	iter, err := ctx.NewHistoryKvIterForKey(phone, "")
	if err != sdk.SUCCESS {
		errMsg := fmt.Sprintf("new HistoryKvIter for key=[%s] failed, %s", phone, err)
		ctx.ErrorResult(errMsg)
		return
	}
	var itinerary Itinerary
	recordMap := make(map[string]HistoryValue, 0)
	for iter.HasNext() {
		km, err := iter.Next()

		if err != sdk.SUCCESS {
			errMsg := "iterator failed to get the next element"
			ctx.ErrorResult(errMsg)
			// 避免出现EOF，暂时跳过
			continue
		}

		err1 := json.Unmarshal(km.Value, &itinerary)
		if err1 != nil {
			errMsg := "json parse element error" + "," + err1.Error()
			ctx.ErrorResult(errMsg)
			continue
		}

		hv := &HistoryValue{
			TxId:        km.TxId,
			Timestamp:   km.Timestamp,
			BlockHeight: km.BlockHeight,
			Key:         km.Key,
			Field:       km.Field,
			Value:       itinerary,
		}
		location := itinerary.Country + "-" + itinerary.City + "-" + itinerary.Region
		if record, ok := recordMap[location]; !ok {
			recordMap[location] = *hv
		} else {
			// 只要最新行踪记录
			if record.BlockHeight < km.BlockHeight {
				recordMap[location] = *hv
			}
		}
		break
	}

	closed, err := iter.Close()
	if !closed || err != sdk.SUCCESS {
		errMsg := fmt.Sprintf("iterator close failed")
		ctx.ErrorResult(errMsg)
	}
	ctx.SuccessResult(string(ToResult(recordMap)))
}

// 空字符串检查
func isBlank(s string) bool {
	return len(strings.TrimSpace(s)) == 0
}
func Time2Str(aTime time.Time, pattern string) string {
	loc, _ := time.LoadLocation("Asia/Shanghai")
	return aTime.In(loc).Format(pattern)
}
func main() {
	// 新版SDK无需初始化代码
}
