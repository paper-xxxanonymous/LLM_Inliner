use contract_sdk_rust::sim_context;
use contract_sdk_rust::sim_context::SimContext;
use contract_sdk_rust::easycodec::*;

// 常量定义
const PARAM_TOKEN: &str = "tokenId";
const PARAM_FROM: &str = "from";
const PARAM_TO: &str = "to";
const PARAM_META_DATA: &str = "metadata";
const TRUE_STRING: &str = "true";
// 合约安装函数
#[no_mangle]
pub extern "C" fn init_contract() {
    let ctx = &mut sim_context::get_sim_context();
    ctx.ok("Init contract success".as_bytes());
}
//合约更新函数
#[no_mangle]
pub extern "C" fn upgrade() {
    let ctx = &mut sim_context::get_sim_context();
    ctx.ok("Upgrade contract success".as_bytes());
}


#[no_mangle]
pub extern "C" fn buyNow() {
    let ctx = &mut sim_context::get_sim_context();
    let tokenId_str=ctx.arg_as_utf8_str("tokenId");
    let from=ctx.arg_as_utf8_str("from");
    let to=ctx.arg_as_utf8_str("to");
    let metadata=ctx.arg_as_utf8_str("metadata");

    // 添加白名单
    let mut ec = EasyCodec::new();
    let mut address=format!("{},{}", from,to);
    ec.add_string("address",&address);
    match ctx.call_contract("identity", "addWriteList", ec) {
        Ok(result) => {

            let result_msg = match String::from_utf8(result) {
                Ok(result_msg) => result_msg,  // Correctly return the value
                Err(e) => {
                    ctx.error("[buyNow] addWriteList Failed to parse bytes as UTF-8");
                    return
                }
            };
            if !result_msg.eq("add write list success"){
                let error_message = format!("[buyNow] addWriteList failed {}",address);
                ctx.error(&error_message);
                return
            }
        }
        Err(resp_code) => {
            // 如果失败，构造错误信息并返回
            let error_message = format!("[buyNow] addWriteList failed with code: {}", resp_code);
            ctx.error(&error_message);
            return
        }
    }

    // 查看是否在白名单(注册)
    let mut ec = EasyCodec::new();
    address=format!("{}", from);
    ec.add_string("address",&address);
    match ctx.call_contract("identity", "isApprovedUser", ec) {
        Ok(result) => {
            let result_msg = match String::from_utf8(result) {
                Ok(result_msg) => result_msg,  // Correctly return the value
                Err(e) => {
                    ctx.error("[buyNow] isApprovedUser from Failed to parse bytes as UTF-8");
                    return
                }
            };
            if !result_msg.eq(TRUE_STRING) {
                let error_message = format!("[buyNow] from address{} not registered",from);
                ctx.error(&error_message);
                return
            }
        }
        Err(resp_code) => {
            // 如果失败，构造错误信息并返回
            let error_message = format!("[buyNow] call_contract isApprovedUser failed with code: {}", resp_code);
            ctx.error(&error_message);
            return
        }
    }
    let mut ec = EasyCodec::new();
    address=format!("{}", to);
    ec.add_string("address",&address);
    match ctx.call_contract("identity", "isApprovedUser", ec) {
        Ok(result) => {
            let result_msg = match String::from_utf8(result) {
                Ok(result_msg) => result_msg,  // Correctly return the value
                Err(e) => {
                    ctx.error("[buyNow] isApprovedUser to Failed to parse bytes as UTF-8");
                    return
                }
            };
            if !result_msg.eq(TRUE_STRING) {
                let error_message = format!("[buyNow] to address{} not registered",to);
                ctx.error(&error_message);
                return
            }
        }
        Err(resp_code) => {
            // 如果失败，构造错误信息并返回
            let error_message = format!("[buyNow] call_contract isApprovedUser failed with code: {}", resp_code);
            ctx.error(&error_message);
            return
        }
    }

    //向from发行nft
    let mut ec = EasyCodec::new();
    ec.add_string("to",&from);
    ec.add_string("tokenId",&tokenId_str);
    ec.add_string("metadata",&metadata);
    match ctx.call_contract("erc721", "mint", ec) {
        Ok(result) => {
            let result_msg = match String::from_utf8(result) {
                Ok(result_msg) => result_msg,  // Correctly return the value
                Err(e) => {
                    ctx.error("[buyNow] mint Failed to parse bytes as UTF-8");
                    return
                }
            };
            if !result_msg.eq("mint success") {
                let error_message = format!("[buyNow] mint failed");
                ctx.error(&error_message);
                return
            }
        }
        Err(resp_code) => {
            // 如果失败，构造错误信息并返回
            let error_message = format!("[buyNow] call_contract mint failed with code: {}", resp_code);
            ctx.error(&error_message);
            return
        }
    }


    //向所有调用者授权from的资产管理权限
    let mut ec = EasyCodec::new();
    ec.add_string("approvalFrom",&from);
    match ctx.call_contract("erc721", "setApprovalForAll2", ec) {
        Ok(result) => {
            let result_msg = match String::from_utf8(result) {
                Ok(result_msg) => result_msg,  // Correctly return the value
                Err(e) => {
                    ctx.error("[buyNow] setApprovalForAll2 Failed to parse bytes as UTF-8");
                    return
                }
            };
            if !result_msg.eq("setApprovalForAll2 success") {
                let error_message = format!("[buyNow] setApprovalForAll2 failed");
                ctx.error(&error_message);
                return
            }
        }
        Err(resp_code) => {
            // 如果失败，构造错误信息并返回
            let error_message = format!("[buyNow] call_contract setApprovalForAll2 failed with code: {}", resp_code);
            ctx.error(&error_message);
            return
        }
    }

    //erc721转移
    let mut ec = EasyCodec::new();
    ec.add_string("from",&from);
    ec.add_string("to",&to);
    ec.add_string("tokenId",&tokenId_str);
    match ctx.call_contract("erc721", "transferFrom", ec) {
        Ok(result) => {
            let result_msg = match String::from_utf8(result) {
                Ok(result_msg) => result_msg,  // Correctly return the value
                Err(e) => {
                    ctx.error("[buyNow] transferFrom Failed to parse bytes as UTF-8");
                    return
                }
            };
            if !result_msg.eq("transfer success") {
                let error_message = format!("[buyNow] transferFrom failed");
                ctx.error(&error_message);
                return
            }
        }
        Err(resp_code) => {
            // 如果失败，构造错误信息并返回
            let error_message = format!("[buyNow] call_contract transferFrom failed with code: {}", resp_code);
            ctx.error(&error_message);
            return
        }
    }
    ctx.ok("buyNow success".as_bytes());
}