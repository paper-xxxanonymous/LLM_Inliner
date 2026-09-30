use contract_sdk_rust::sim_context;
use contract_sdk_rust::sim_context::SimContext;
use contract_sdk_rust::easycodec::*;
use hex;

const PARAM_ADMIN_ADDRESS: &str = "adminAddress";
const PARAM_ADDRESS: &str = "address";
const KEY_ADMIN_ADDRESS: &str = "adminAddress";
const ZERO_ADDR: &str = "0000000000000000000000000000000000000000";
const ZERO_ADDR_WITH_PREFIX: &str = "0x0000000000000000000000000000000000000000";
// 根据公钥计算地址
fn calc_address(pub_key: &str) -> String {
    return pub_key.to_string();
}

//判断发送者是不是管理员
fn is_admin()->bool{
    let ctx = &mut sim_context::get_sim_context();
    let sender_key = ctx.get_sender_pub_key();
    let sender_address=calc_address(&sender_key);
    let r = ctx.get_state("identity", "adminAddress");
    if r.is_err() {
        ctx.error("no admin address");
        return false;
    }
    let admin_addresses = r.unwrap();
    if admin_addresses.len() == 0 {
        ctx.error("no admin address");
        return false;
    }
    let admin_addresses_vec= String::from_utf8(admin_addresses)
        .unwrap_or_default() // 遇到非 UTF-8 数据时提供默认值
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    for address in admin_addresses_vec {
        if sender_address.eq(&address) {
            return true;
        }
    }
    return false;
}



fn is_valid_address(addr: &str) -> bool {
    let addr_len = ZERO_ADDR.len();
    let addr_len_with_prefix = ZERO_ADDR_WITH_PREFIX.len();

    if addr.len() != addr_len && addr.len() != addr_len_with_prefix {
        return false;
    }

    let addr = addr.trim_start_matches("0x"); // 去掉前缀 0x
    return if hex::decode(addr).is_ok() {
        true
    } else {
        false
    }
}
// 合约安装函数
#[no_mangle]
pub extern "C" fn init_contract() {
    let ctx = &mut sim_context::get_sim_context();

    // 获取管理员地址参数
    let mut admin_addresses = ctx.arg_as_utf8_str("adminAddress");
    // 如果传过来的地址列表是空的，默认设置管理员为发送者地址
    if admin_addresses.is_empty(){
        let sender_key = ctx.get_sender_pub_key();
        admin_addresses=calc_address(&sender_key);
    }

    ctx.put_state("identity","adminAddress",admin_addresses.as_bytes());
    ctx.put_state("identity", "userCount","0".as_bytes());//这个userCount没什么作用，只是为了和go的identity合约保持一致
    ctx.emit_event("alterAdminAddress", &vec!["Contract initialized".to_string()]);
    ctx.ok("Init contract success".as_bytes());
}

#[no_mangle]
pub extern "C" fn upgrade() {
    let ctx = &mut sim_context::get_sim_context();
    ctx.ok("Upgrade contract success".as_bytes());
}

// 添加白名单
#[no_mangle]
pub extern "C" fn addWriteList() {
    let ctx = &mut sim_context::get_sim_context();
    let method="addWriteList";
    let addresses=ctx.arg_as_utf8_str(PARAM_ADDRESS);
    // let error_message = format!("[addWriteList] addresses {}",addresses);
    // ctx.error(&error_message);
    // 如果传过来的地址列表是空的
    if addresses.is_empty(){
        let error_message = format!("[{}] address of param should not be empty",method);
        ctx.error(&error_message);
        return;
    }
    let addresses_vec= addresses
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    //取消验证
    // if !is_admin(ctx) {
    //     ctx.error("Permission denied");
    //     return;
    // }

    for address in &addresses_vec {
        if !is_valid_address(&address) {
            let error_message = format!("[{}] invalid address",method);
            ctx.error(&error_message);
            return;
        }
        ctx.put_state("identity", &address, "1".as_bytes());;
    }
    //这句话运行失败不知道为什么
    // ctx.emit_event("addWriteList", &addresses_vec);
    ctx.ok("add write list success".as_bytes());
}

// 移除白名单
#[no_mangle]
pub extern "C" fn removeWriteList() {
    let ctx = &mut sim_context::get_sim_context();
    let addresses=ctx.arg_as_utf8_str(PARAM_ADDRESS);
    let method="removeWriteList";
    // 如果传过来的地址列表是空的
    if addresses.is_empty(){
        let error_message = format!("[{}] address of param should not be empty",method);
        ctx.error(&error_message);
        return;
    }
    let addresses_vec= addresses
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();   //取消验证
    // if !is_admin(ctx) {
    //     ctx.error("Permission denied");
    //     return;
    // }
    for address in &addresses_vec {
        ctx.delete_state("identity", &address);
    }
    // ctx.emit_event("removeWriteList", &addresses_vec);
    ctx.ok("remove write list success".as_bytes());
}

//返回发送者地址
#[no_mangle]
pub extern "C" fn address() {
    let ctx = &mut sim_context::get_sim_context();
    let sender_key = ctx.get_sender_pub_key();
    let sender_address=calc_address(&sender_key);
    ctx.ok(sender_address.as_bytes());
}

//返回发送者地址，实际上是在链上调用自己这个identity合约的address函数
#[no_mangle]
pub extern "C" fn callerAddress() {
    let ctx = &mut sim_context::get_sim_context();
    let mut ec = EasyCodec::new();
    match ctx.call_contract("identity", "address", ec) {
        Ok(vec) => {
            // 如果成功，将返回值作为成功信息返回
            ctx.ok(vec.as_slice());
            return
        }
        Err(resp_code) => {
            // 如果失败，构造错误信息并返回
            let error_message = format!("call_contract failed with code: {}", resp_code);
            ctx.error(&error_message);
            return
        }
    }
}

//判断传过来的地址列表是不是在白名单中
#[no_mangle]
pub extern "C" fn isApprovedUser() {
    let ctx = &mut sim_context::get_sim_context();
    let addresses=ctx.arg_as_utf8_str(PARAM_ADDRESS);
    // 如果传过来的地址列表是空的
    if addresses.is_empty(){
        ctx.error("address of param should not be empty");
        return;
    }
    let addresses_vec= addresses
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();//取消验证

    let mut flag=true;
    for address in &addresses_vec {
        let r = ctx.get_state("identity", &address);
        if r.is_err() {
            flag=false;
        }
        let approved = r.unwrap();
        if approved.len() == 0 {
            flag=false;
        }
    }
    if flag==true{
        ctx.ok("true".as_bytes());
    }else{
        ctx.ok("false".as_bytes());
    }

}
